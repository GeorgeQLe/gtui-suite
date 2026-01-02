//! Shell configuration.

use crate::notification::NotificationConfig;
use crate::ShellVariant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Shell configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Shell variant.
    #[serde(default)]
    pub variant: ShellVariant,
    /// Prefix key binding.
    #[serde(default = "default_prefix_key")]
    pub prefix_key: String,
    /// Prefix key timeout in milliseconds.
    #[serde(default = "default_prefix_timeout")]
    pub prefix_timeout_ms: u64,
    /// Session configuration.
    #[serde(default)]
    pub session: SessionConfig,
    /// Notification configuration.
    #[serde(default)]
    pub notifications: NotificationConfig,
    /// Workspace definitions.
    #[serde(default)]
    pub workspaces: HashMap<String, Vec<String>>,
    /// Startup configuration.
    #[serde(default)]
    pub startup: StartupConfig,
    /// Crash handling configuration.
    #[serde(default)]
    pub crash: CrashConfig,
}

fn default_prefix_key() -> String {
    "ctrl+space".to_string()
}

fn default_prefix_timeout() -> u64 {
    500
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            variant: ShellVariant::default(),
            prefix_key: default_prefix_key(),
            prefix_timeout_ms: default_prefix_timeout(),
            session: SessionConfig::default(),
            notifications: NotificationConfig::default(),
            workspaces: HashMap::new(),
            startup: StartupConfig::default(),
            crash: CrashConfig::default(),
        }
    }
}

impl ShellConfig {
    /// Load configuration from file.
    pub fn load(path: &PathBuf) -> Result<Self, crate::ShellError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from default location.
    pub fn load_default() -> Result<Self, crate::ShellError> {
        if let Some(path) = Self::default_path() {
            if path.exists() {
                return Self::load(&path);
            }
        }
        Ok(Self::default())
    }

    /// Get default config path.
    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "tui-shell")
            .map(|d| d.config_dir().join("config.toml"))
    }

    /// Save configuration to file.
    pub fn save(&self, path: &PathBuf) -> Result<(), crate::ShellError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::ShellError::Config(e.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Session persistence configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Whether session persistence is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Whether to auto-save session.
    #[serde(default = "default_true")]
    pub auto_save: bool,
    /// Auto-save interval in seconds.
    #[serde(default = "default_save_interval")]
    pub save_interval_secs: u64,
    /// Whether to restore session on start.
    #[serde(default = "default_true")]
    pub restore_on_start: bool,
}

fn default_true() -> bool {
    true
}

fn default_save_interval() -> u64 {
    300
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_save: true,
            save_interval_secs: 300,
            restore_on_start: true,
        }
    }
}

/// Startup configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StartupConfig {
    /// Apps to launch on startup.
    #[serde(default)]
    pub apps: Vec<String>,
    /// Initial workspace.
    #[serde(default)]
    pub workspace: Option<String>,
}

/// Crash handling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashConfig {
    /// Whether to show crash dialog.
    #[serde(default = "default_true")]
    pub show_dialog: bool,
    /// Whether to auto-restart by default.
    #[serde(default)]
    pub auto_restart_default: bool,
    /// Initial backoff in milliseconds.
    #[serde(default = "default_backoff_initial")]
    pub backoff_initial_ms: u64,
    /// Maximum backoff in milliseconds.
    #[serde(default = "default_backoff_max")]
    pub backoff_max_ms: u64,
}

fn default_backoff_initial() -> u64 {
    1000
}

fn default_backoff_max() -> u64 {
    30000
}

impl Default for CrashConfig {
    fn default() -> Self {
        Self {
            show_dialog: true,
            auto_restart_default: false,
            backoff_initial_ms: 1000,
            backoff_max_ms: 30000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ShellConfig::default();
        assert_eq!(config.variant, ShellVariant::Tiled);
        assert_eq!(config.prefix_key, "ctrl+space");
    }

    #[test]
    fn test_config_serialization() {
        let config = ShellConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("variant"));

        let restored: ShellConfig = toml::from_str(&toml).unwrap();
        assert_eq!(restored.variant, config.variant);
    }

    #[test]
    fn test_session_config() {
        let config = SessionConfig::default();
        assert!(config.enabled);
        assert!(config.auto_save);
    }
}
