//! Theme manager for loading and switching themes.

use crate::{Theme, ThemeOverrides, ThemeMeta};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Theme loading/validation error.
#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("Failed to parse theme: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// Warning from theme validation.
#[derive(Debug, Clone)]
pub enum ThemeWarning {
    /// A token was missing and a default was used
    MissingToken {
        path: String,
        default_used: String,
    },
    /// A deprecated token was used
    DeprecatedToken {
        path: String,
        replacement: String,
    },
}

/// Result of theme validation.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Non-fatal warnings
    pub warnings: Vec<ThemeWarning>,
    /// Fatal errors (theme won't load)
    pub errors: Vec<ThemeError>,
}

impl ValidationResult {
    /// Check if validation succeeded (no errors).
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Manages theme loading, switching, and hot reload.
pub struct ThemeManager {
    /// Built-in themes
    builtin: HashMap<String, Theme>,
    /// User-loaded themes
    user: HashMap<String, Theme>,
    /// Current theme name
    current: String,
    /// Runtime overrides
    overrides: ThemeOverrides,
    /// Search paths for user themes
    search_paths: Vec<PathBuf>,
    /// File watcher for hot reload
    #[cfg(feature = "hot-reload")]
    watcher: Option<notify::RecommendedWatcher>,
}

impl ThemeManager {
    /// Create a new theme manager with built-in themes.
    pub fn new() -> Self {
        let builtin = crate::presets::builtin_themes();
        let current = "default-dark".to_string();

        Self {
            builtin,
            user: HashMap::new(),
            current,
            overrides: ThemeOverrides::default(),
            search_paths: Vec::new(),
            #[cfg(feature = "hot-reload")]
            watcher: None,
        }
    }

    /// Set search paths for user themes.
    pub fn with_search_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.search_paths = paths;
        self
    }

    /// Load user themes from search paths.
    pub fn load_user_themes(&mut self) -> ValidationResult {
        let mut result = ValidationResult::default();

        for search_path in &self.search_paths.clone() {
            if !search_path.exists() {
                continue;
            }

            let entries = match std::fs::read_dir(search_path) {
                Ok(e) => e,
                Err(e) => {
                    result.errors.push(ThemeError::IoError(e));
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "toml") {
                    match self.load_theme_file(&path) {
                        Ok((name, theme)) => {
                            self.user.insert(name, theme);
                        }
                        Err(e) => {
                            result.errors.push(e);
                        }
                    }
                }
            }
        }

        result
    }

    /// Load a specific theme file.
    fn load_theme_file(&self, path: &Path) -> Result<(String, Theme), ThemeError> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&content)?;
        let name = theme.name.clone();

        // Handle inheritance
        let theme = if let Some(ref parent_name) = theme.extends {
            self.resolve_inheritance(theme, parent_name)?
        } else {
            theme
        };

        Ok((name, theme))
    }

    /// Resolve theme inheritance.
    fn resolve_inheritance(&self, child: Theme, parent_name: &str) -> Result<Theme, ThemeError> {
        let parent = self
            .get(parent_name)
            .ok_or_else(|| ThemeError::NotFound(format!("Parent theme not found: {}", parent_name)))?;

        // In a full implementation, merge child onto parent
        // For now, return child as-is
        Ok(child)
    }

    /// Get a theme by name.
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.user.get(name).or_else(|| self.builtin.get(name))
    }

    /// Get the current theme (with overrides applied).
    pub fn current(&self) -> Theme {
        let base = self
            .get(&self.current)
            .cloned()
            .unwrap_or_else(|| crate::presets::default_dark());

        self.apply_overrides(base)
    }

    /// Set the current theme.
    pub fn set_current(&mut self, name: &str) -> Result<(), ThemeError> {
        if self.get(name).is_none() {
            return Err(ThemeError::NotFound(name.to_string()));
        }
        self.current = name.to_string();
        Ok(())
    }

    /// Apply runtime overrides.
    pub fn apply_overrides(&self, mut theme: Theme) -> Theme {
        // Apply color overrides
        for (key, token) in &self.overrides.colors {
            // Would apply to the appropriate color in the palette
            // This is simplified
            let _ = (key, token);
        }

        // Apply animation overrides
        if let Some(ref animation) = self.overrides.animation {
            theme.animation = animation.clone();
        }

        // Apply variant overrides
        if let Some(variant) = self.overrides.variant {
            theme.variant = variant;
        }

        theme
    }

    /// Set runtime overrides.
    pub fn set_overrides(&mut self, overrides: ThemeOverrides) {
        self.overrides = overrides;
    }

    /// List all available themes.
    pub fn list_themes(&self) -> Vec<ThemeMeta> {
        let mut themes = Vec::new();

        for (name, theme) in &self.builtin {
            themes.push(ThemeMeta {
                name: name.clone(),
                display_name: theme.name.clone(),
                description: None,
                is_dark: name.contains("dark"),
                is_builtin: true,
                path: None,
            });
        }

        for (name, theme) in &self.user {
            themes.push(ThemeMeta {
                name: name.clone(),
                display_name: theme.name.clone(),
                description: None,
                is_dark: name.contains("dark"),
                is_builtin: false,
                path: None,
            });
        }

        themes.sort_by(|a, b| a.name.cmp(&b.name));
        themes
    }

    /// Enable hot reload for theme files.
    #[cfg(feature = "hot-reload")]
    pub fn enable_hot_reload<F>(&mut self, on_change: F) -> Result<(), notify::Error>
    where
        F: Fn(&Theme) + Send + 'static,
    {
        use notify::{RecommendedWatcher, RecursiveMode, Watcher};

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

        for path in &self.search_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::NonRecursive)?;
            }
        }

        self.watcher = Some(watcher);

        std::thread::spawn(move || {
            for res in rx {
                match res {
                    Ok(event) => {
                        // Reload theme and call callback
                    }
                    Err(e) => {
                        eprintln!("Watch error: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = ThemeManager::new();
        assert!(!manager.builtin.is_empty());
    }

    #[test]
    fn test_get_theme() {
        let manager = ThemeManager::new();
        assert!(manager.get("default-dark").is_some());
    }

    #[test]
    fn test_set_current() {
        let mut manager = ThemeManager::new();
        assert!(manager.set_current("default-dark").is_ok());
        assert!(manager.set_current("nonexistent").is_err());
    }

    #[test]
    fn test_list_themes() {
        let manager = ThemeManager::new();
        let themes = manager.list_themes();
        assert!(!themes.is_empty());
    }
}
