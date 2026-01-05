//! # tui-theme
//!
//! Full-featured theming engine for the TUI Suite applications.
//!
//! ## Features
//!
//! - Semantic color tokens with state-level granularity
//! - Multiple color depth support (24-bit, 256, 16)
//! - Theme inheritance and layered overrides
//! - Built-in presets (Catppuccin, Nord, Solarized, etc.)
//! - High contrast and colorblind accessibility modes
//! - Hot reload for theme development
//! - Optional syntax highlighting support

mod animation;
mod colors;
mod manager;
mod presets;
mod spacing;
mod styles;
mod variant;

pub use animation::{AnimationConfig, EasingFunction};
pub use colors::{Color, ColorDepth, ColorPalette, ColorSet, ColorToken};
pub use manager::{ThemeManager, ValidationResult, ThemeWarning, ThemeError};
pub use presets::builtin_themes;
pub use spacing::{Spacing, SPACING_XS, SPACING_SM, SPACING_MD, SPACING_LG, SPACING_XL};
pub use styles::{
    BorderStyle, BorderType, DialogStyles, FormStyles, PaletteStyles, Sides,
    StateStyles, StatusBarStyles, StyleMap, TabStyles, TableStyles, TreeStyles,
    WidgetStyle,
};
pub use variant::ThemeVariant;

use ratatui::style::Style as RatatuiStyle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A complete theme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Optional parent theme to inherit from
    #[serde(default)]
    pub extends: Option<String>,
    /// Color palettes for different terminal capabilities
    pub colors: ColorPalette,
    /// Widget-specific styles
    #[serde(default)]
    pub styles: StyleMap,
    /// Theme variant (compact, comfortable, spacious)
    #[serde(default)]
    pub variant: ThemeVariant,
    /// Animation configuration
    #[serde(default)]
    pub animation: AnimationConfig,
    /// Optional syntax highlighting colors
    #[serde(default)]
    pub syntax: Option<SyntaxColors>,
}

impl Theme {
    /// Create a new theme with default values.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            extends: None,
            colors: ColorPalette::default(),
            styles: StyleMap::default(),
            variant: ThemeVariant::default(),
            animation: AnimationConfig::default(),
            syntax: None,
        }
    }

    /// Create a high-contrast version of this theme.
    pub fn to_high_contrast(&self, config: HighContrastConfig) -> Theme {
        let mut theme = self.clone();
        theme.name = format!("{} (High Contrast)", self.name);
        // Apply contrast boost to all color tokens
        theme.colors = theme.colors.with_increased_contrast(config);
        theme
    }

    /// Create a colorblind-friendly version of this theme.
    pub fn to_colorblind(&self, mode: ColorblindMode) -> Theme {
        let mut theme = self.clone();
        theme.name = format!("{} ({})", self.name, mode.label());
        // Apply daltonization to all colors
        theme.colors = theme.colors.with_colorblind_filter(mode);
        theme
    }

    /// Get the active color set based on terminal capabilities.
    pub fn color_set(&self, depth: ColorDepth) -> &ColorSet {
        match depth {
            ColorDepth::TrueColor => &self.colors.true_color,
            ColorDepth::Color256 => &self.colors.color_256,
            ColorDepth::Color16 => &self.colors.color_16,
        }
    }

    /// Convert a color token to a Ratatui style.
    pub fn to_ratatui_style(&self, token: &ColorToken) -> RatatuiStyle {
        let mut style = RatatuiStyle::default();
        if let Some(fg) = token.color.to_ratatui() {
            style = style.fg(fg);
        }
        style = style.add_modifier(token.modifiers);
        style
    }
}

impl Default for Theme {
    fn default() -> Self {
        presets::default_dark()
    }
}

/// High contrast configuration.
#[derive(Debug, Clone, Copy)]
pub struct HighContrastConfig {
    /// Minimum contrast ratio (WCAG AA = 4.5, AAA = 7.0)
    pub min_contrast_ratio: f32,
    /// How much to boost contrast
    pub boost_factor: f32,
}

impl Default for HighContrastConfig {
    fn default() -> Self {
        Self {
            min_contrast_ratio: 4.5, // WCAG AA
            boost_factor: 1.5,
        }
    }
}

impl HighContrastConfig {
    /// WCAG AAA level contrast.
    pub fn wcag_aaa() -> Self {
        Self {
            min_contrast_ratio: 7.0,
            boost_factor: 2.0,
        }
    }
}

/// Colorblind simulation/correction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorblindMode {
    /// Red-green colorblindness (most common, ~6% of males)
    Deuteranopia,
    /// Red-green colorblindness
    Protanopia,
    /// Blue-yellow colorblindness
    Tritanopia,
}

impl ColorblindMode {
    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Deuteranopia => "Deuteranopia",
            Self::Protanopia => "Protanopia",
            Self::Tritanopia => "Tritanopia",
        }
    }
}

/// Syntax highlighting colors for code display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: Option<ColorToken>,
    pub string: Option<ColorToken>,
    pub comment: Option<ColorToken>,
    pub function: Option<ColorToken>,
    pub type_name: Option<ColorToken>,
    pub number: Option<ColorToken>,
    pub operator: Option<ColorToken>,
    pub punctuation: Option<ColorToken>,
    pub variable: Option<ColorToken>,
    pub constant: Option<ColorToken>,
}

/// Trait for types that provide syntax highlighting colors.
pub trait SyntaxTheme {
    fn keyword(&self) -> ColorToken;
    fn string(&self) -> ColorToken;
    fn comment(&self) -> ColorToken;
    fn function(&self) -> ColorToken;
    fn type_name(&self) -> ColorToken;
    fn number(&self) -> ColorToken;
    fn operator(&self) -> ColorToken;
    fn punctuation(&self) -> ColorToken;
    fn variable(&self) -> ColorToken;
    fn constant(&self) -> ColorToken;
}

/// Theme metadata for listing available themes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_dark: bool,
    pub is_builtin: bool,
    pub path: Option<PathBuf>,
}

/// Theme overrides that can be applied at runtime.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeOverrides {
    /// Override specific color tokens
    #[serde(default)]
    pub colors: HashMap<String, ColorToken>,
    /// Override animation settings
    #[serde(default)]
    pub animation: Option<AnimationConfig>,
    /// Override variant
    #[serde(default)]
    pub variant: Option<ThemeVariant>,
}

impl ThemeOverrides {
    /// Create empty overrides.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override a color.
    pub fn color(mut self, name: impl Into<String>, token: ColorToken) -> Self {
        self.colors.insert(name.into(), token);
        self
    }

    /// Override animation config.
    pub fn animation(mut self, config: AnimationConfig) -> Self {
        self.animation = Some(config);
        self
    }

    /// Override variant.
    pub fn variant(mut self, variant: ThemeVariant) -> Self {
        self.variant = Some(variant);
        self
    }
}

/// Get the default theme search paths.
pub fn default_search_paths(app_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // User app-specific themes
    if let Some(config_dir) = directories::ProjectDirs::from("", "", app_name) {
        let mut theme_dir = config_dir.config_dir().to_path_buf();
        theme_dir.push("themes");
        paths.push(theme_dir);
    }

    // User shared themes
    if let Some(config_dir) = directories::ProjectDirs::from("", "", "tui-suite") {
        let mut theme_dir = config_dir.config_dir().to_path_buf();
        theme_dir.push("themes");
        paths.push(theme_dir);
    }

    // System-wide themes
    paths.push(PathBuf::from("/usr/share/tui-suite/themes"));

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::new("test-theme");
        assert_eq!(theme.name, "test-theme");
        assert!(theme.extends.is_none());
    }

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert!(!theme.name.is_empty());
    }

    #[test]
    fn test_theme_overrides() {
        let overrides = ThemeOverrides::new()
            .color("accent", ColorToken::new(Color::hex("#ff0000")))
            .variant(ThemeVariant::Compact);

        assert!(overrides.colors.contains_key("accent"));
        assert_eq!(overrides.variant, Some(ThemeVariant::Compact));
    }

    #[test]
    fn test_colorblind_modes() {
        assert_eq!(ColorblindMode::Deuteranopia.label(), "Deuteranopia");
        assert_eq!(ColorblindMode::Protanopia.label(), "Protanopia");
        assert_eq!(ColorblindMode::Tritanopia.label(), "Tritanopia");
    }
}
