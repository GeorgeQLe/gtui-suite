//! Built-in theme presets.

use crate::animation::AnimationConfig;
use crate::colors::{Color, ColorPalette, ColorSet, ColorToken};
use crate::styles::StyleMap;
use crate::variant::ThemeVariant;
use crate::Theme;
use std::collections::HashMap;

/// Get all built-in themes.
pub fn builtin_themes() -> HashMap<String, Theme> {
    let mut themes = HashMap::new();

    themes.insert("default-dark".to_string(), default_dark());
    themes.insert("default-light".to_string(), default_light());
    themes.insert("catppuccin-mocha".to_string(), catppuccin_mocha());
    themes.insert("catppuccin-latte".to_string(), catppuccin_latte());
    themes.insert("nord".to_string(), nord());
    themes.insert("solarized-dark".to_string(), solarized_dark());
    themes.insert("solarized-light".to_string(), solarized_light());
    themes.insert("gruvbox-dark".to_string(), gruvbox_dark());
    themes.insert("dracula".to_string(), dracula());
    themes.insert("one-dark".to_string(), one_dark());

    // High contrast variants
    themes.insert("high-contrast-dark".to_string(), high_contrast_dark());
    themes.insert("high-contrast-light".to_string(), high_contrast_light());

    themes
}

/// Default dark theme.
pub fn default_dark() -> Theme {
    Theme {
        name: "Default Dark".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: dark_color_set(),
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// Default light theme.
pub fn default_light() -> Theme {
    Theme {
        name: "Default Light".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: light_color_set(),
            color_256: light_color_set_256(),
            color_16: light_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

fn dark_color_set() -> ColorSet {
    ColorSet {
        bg_primary: ColorToken::new(Color::hex("#1a1b26")),
        bg_secondary: ColorToken::new(Color::hex("#24283b")),
        bg_tertiary: ColorToken::new(Color::hex("#414868")),
        bg_hover: ColorToken::new(Color::hex("#33467c")),
        bg_focused: ColorToken::new(Color::hex("#3d59a1")),
        bg_pressed: ColorToken::new(Color::hex("#2a2e3f")),
        bg_disabled: ColorToken::new(Color::hex("#1a1b26")),

        fg_primary: ColorToken::new(Color::hex("#c0caf5")),
        fg_secondary: ColorToken::new(Color::hex("#a9b1d6")),
        fg_muted: ColorToken::new(Color::hex("#565f89")),
        fg_disabled: ColorToken::new(Color::hex("#414868")),

        accent: ColorToken::new(Color::hex("#7aa2f7")),
        accent_secondary: ColorToken::new(Color::hex("#bb9af7")),
        accent_hover: ColorToken::new(Color::hex("#89b4fa")),
        accent_focused: ColorToken::new(Color::hex("#7dcfff")),

        success: ColorToken::new(Color::hex("#9ece6a")),
        warning: ColorToken::new(Color::hex("#e0af68")),
        error: ColorToken::new(Color::hex("#f7768e")),
        info: ColorToken::new(Color::hex("#7aa2f7")),

        border: ColorToken::new(Color::hex("#414868")),
        border_focused: ColorToken::new(Color::hex("#7aa2f7")),
        border_error: ColorToken::new(Color::hex("#f7768e")),

        flash_success: ColorToken::new(Color::hex("#9ece6a")),
        flash_error: ColorToken::new(Color::hex("#f7768e")),
    }
}

fn dark_color_set_256() -> ColorSet {
    ColorSet {
        bg_primary: ColorToken::new(Color::index(234)),
        bg_secondary: ColorToken::new(Color::index(235)),
        bg_tertiary: ColorToken::new(Color::index(237)),
        bg_hover: ColorToken::new(Color::index(238)),
        bg_focused: ColorToken::new(Color::index(24)),
        bg_pressed: ColorToken::new(Color::index(236)),
        bg_disabled: ColorToken::new(Color::index(234)),

        fg_primary: ColorToken::new(Color::index(253)),
        fg_secondary: ColorToken::new(Color::index(250)),
        fg_muted: ColorToken::new(Color::index(245)),
        fg_disabled: ColorToken::new(Color::index(240)),

        accent: ColorToken::new(Color::index(75)),
        accent_secondary: ColorToken::new(Color::index(141)),
        accent_hover: ColorToken::new(Color::index(117)),
        accent_focused: ColorToken::new(Color::index(81)),

        success: ColorToken::new(Color::index(107)),
        warning: ColorToken::new(Color::index(179)),
        error: ColorToken::new(Color::index(203)),
        info: ColorToken::new(Color::index(75)),

        border: ColorToken::new(Color::index(240)),
        border_focused: ColorToken::new(Color::index(75)),
        border_error: ColorToken::new(Color::index(203)),

        flash_success: ColorToken::new(Color::index(107)),
        flash_error: ColorToken::new(Color::index(203)),
    }
}

fn dark_color_set_16() -> ColorSet {
    ColorSet {
        bg_primary: ColorToken::new(Color::named("black")),
        bg_secondary: ColorToken::new(Color::named("black")),
        bg_tertiary: ColorToken::new(Color::named("darkgray")),
        bg_hover: ColorToken::new(Color::named("darkgray")),
        bg_focused: ColorToken::new(Color::named("blue")),
        bg_pressed: ColorToken::new(Color::named("black")),
        bg_disabled: ColorToken::new(Color::named("black")),

        fg_primary: ColorToken::new(Color::named("white")),
        fg_secondary: ColorToken::new(Color::named("white")),
        fg_muted: ColorToken::new(Color::named("gray")),
        fg_disabled: ColorToken::new(Color::named("darkgray")),

        accent: ColorToken::new(Color::named("blue")),
        accent_secondary: ColorToken::new(Color::named("magenta")),
        accent_hover: ColorToken::new(Color::named("lightblue")),
        accent_focused: ColorToken::new(Color::named("cyan")),

        success: ColorToken::new(Color::named("green")),
        warning: ColorToken::new(Color::named("yellow")),
        error: ColorToken::new(Color::named("red")),
        info: ColorToken::new(Color::named("blue")),

        border: ColorToken::new(Color::named("gray")),
        border_focused: ColorToken::new(Color::named("blue")),
        border_error: ColorToken::new(Color::named("red")),

        flash_success: ColorToken::new(Color::named("green")),
        flash_error: ColorToken::new(Color::named("red")),
    }
}

fn light_color_set() -> ColorSet {
    ColorSet {
        bg_primary: ColorToken::new(Color::hex("#f5f5f5")),
        bg_secondary: ColorToken::new(Color::hex("#e8e8e8")),
        bg_tertiary: ColorToken::new(Color::hex("#d0d0d0")),
        bg_hover: ColorToken::new(Color::hex("#c8d3f5")),
        bg_focused: ColorToken::new(Color::hex("#b4c6e7")),
        bg_pressed: ColorToken::new(Color::hex("#d8d8d8")),
        bg_disabled: ColorToken::new(Color::hex("#f0f0f0")),

        fg_primary: ColorToken::new(Color::hex("#1a1b26")),
        fg_secondary: ColorToken::new(Color::hex("#343b58")),
        fg_muted: ColorToken::new(Color::hex("#6c7086")),
        fg_disabled: ColorToken::new(Color::hex("#9ca0b0")),

        accent: ColorToken::new(Color::hex("#2e7de9")),
        accent_secondary: ColorToken::new(Color::hex("#9854f1")),
        accent_hover: ColorToken::new(Color::hex("#4d9cf5")),
        accent_focused: ColorToken::new(Color::hex("#007197")),

        success: ColorToken::new(Color::hex("#587539")),
        warning: ColorToken::new(Color::hex("#8c6c3e")),
        error: ColorToken::new(Color::hex("#f52a65")),
        info: ColorToken::new(Color::hex("#2e7de9")),

        border: ColorToken::new(Color::hex("#c0c0c0")),
        border_focused: ColorToken::new(Color::hex("#2e7de9")),
        border_error: ColorToken::new(Color::hex("#f52a65")),

        flash_success: ColorToken::new(Color::hex("#587539")),
        flash_error: ColorToken::new(Color::hex("#f52a65")),
    }
}

fn light_color_set_256() -> ColorSet {
    ColorSet {
        bg_primary: ColorToken::new(Color::index(255)),
        bg_secondary: ColorToken::new(Color::index(254)),
        bg_tertiary: ColorToken::new(Color::index(252)),
        bg_hover: ColorToken::new(Color::index(153)),
        bg_focused: ColorToken::new(Color::index(111)),
        bg_pressed: ColorToken::new(Color::index(253)),
        bg_disabled: ColorToken::new(Color::index(255)),

        fg_primary: ColorToken::new(Color::index(234)),
        fg_secondary: ColorToken::new(Color::index(238)),
        fg_muted: ColorToken::new(Color::index(244)),
        fg_disabled: ColorToken::new(Color::index(250)),

        accent: ColorToken::new(Color::index(33)),
        accent_secondary: ColorToken::new(Color::index(129)),
        accent_hover: ColorToken::new(Color::index(39)),
        accent_focused: ColorToken::new(Color::index(31)),

        success: ColorToken::new(Color::index(64)),
        warning: ColorToken::new(Color::index(136)),
        error: ColorToken::new(Color::index(197)),
        info: ColorToken::new(Color::index(33)),

        border: ColorToken::new(Color::index(250)),
        border_focused: ColorToken::new(Color::index(33)),
        border_error: ColorToken::new(Color::index(197)),

        flash_success: ColorToken::new(Color::index(64)),
        flash_error: ColorToken::new(Color::index(197)),
    }
}

fn light_color_set_16() -> ColorSet {
    ColorSet {
        bg_primary: ColorToken::new(Color::named("white")),
        bg_secondary: ColorToken::new(Color::named("white")),
        bg_tertiary: ColorToken::new(Color::named("gray")),
        bg_hover: ColorToken::new(Color::named("lightblue")),
        bg_focused: ColorToken::new(Color::named("blue")),
        bg_pressed: ColorToken::new(Color::named("gray")),
        bg_disabled: ColorToken::new(Color::named("white")),

        fg_primary: ColorToken::new(Color::named("black")),
        fg_secondary: ColorToken::new(Color::named("black")),
        fg_muted: ColorToken::new(Color::named("darkgray")),
        fg_disabled: ColorToken::new(Color::named("gray")),

        accent: ColorToken::new(Color::named("blue")),
        accent_secondary: ColorToken::new(Color::named("magenta")),
        accent_hover: ColorToken::new(Color::named("lightblue")),
        accent_focused: ColorToken::new(Color::named("cyan")),

        success: ColorToken::new(Color::named("green")),
        warning: ColorToken::new(Color::named("yellow")),
        error: ColorToken::new(Color::named("red")),
        info: ColorToken::new(Color::named("blue")),

        border: ColorToken::new(Color::named("gray")),
        border_focused: ColorToken::new(Color::named("blue")),
        border_error: ColorToken::new(Color::named("red")),

        flash_success: ColorToken::new(Color::named("green")),
        flash_error: ColorToken::new(Color::named("red")),
    }
}

/// Catppuccin Mocha theme.
pub fn catppuccin_mocha() -> Theme {
    Theme {
        name: "Catppuccin Mocha".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#1e1e2e")),
                bg_secondary: ColorToken::new(Color::hex("#313244")),
                bg_tertiary: ColorToken::new(Color::hex("#45475a")),
                bg_hover: ColorToken::new(Color::hex("#585b70")),
                bg_focused: ColorToken::new(Color::hex("#89b4fa")),
                bg_pressed: ColorToken::new(Color::hex("#313244")),
                bg_disabled: ColorToken::new(Color::hex("#1e1e2e")),

                fg_primary: ColorToken::new(Color::hex("#cdd6f4")),
                fg_secondary: ColorToken::new(Color::hex("#bac2de")),
                fg_muted: ColorToken::new(Color::hex("#6c7086")),
                fg_disabled: ColorToken::new(Color::hex("#45475a")),

                accent: ColorToken::new(Color::hex("#89b4fa")),
                accent_secondary: ColorToken::new(Color::hex("#cba6f7")),
                accent_hover: ColorToken::new(Color::hex("#b4befe")),
                accent_focused: ColorToken::new(Color::hex("#74c7ec")),

                success: ColorToken::new(Color::hex("#a6e3a1")),
                warning: ColorToken::new(Color::hex("#f9e2af")),
                error: ColorToken::new(Color::hex("#f38ba8")),
                info: ColorToken::new(Color::hex("#89b4fa")),

                border: ColorToken::new(Color::hex("#45475a")),
                border_focused: ColorToken::new(Color::hex("#89b4fa")),
                border_error: ColorToken::new(Color::hex("#f38ba8")),

                flash_success: ColorToken::new(Color::hex("#a6e3a1")),
                flash_error: ColorToken::new(Color::hex("#f38ba8")),
            },
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// Catppuccin Latte theme.
pub fn catppuccin_latte() -> Theme {
    Theme {
        name: "Catppuccin Latte".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#eff1f5")),
                bg_secondary: ColorToken::new(Color::hex("#e6e9ef")),
                bg_tertiary: ColorToken::new(Color::hex("#ccd0da")),
                bg_hover: ColorToken::new(Color::hex("#bcc0cc")),
                bg_focused: ColorToken::new(Color::hex("#1e66f5")),
                bg_pressed: ColorToken::new(Color::hex("#dce0e8")),
                bg_disabled: ColorToken::new(Color::hex("#eff1f5")),

                fg_primary: ColorToken::new(Color::hex("#4c4f69")),
                fg_secondary: ColorToken::new(Color::hex("#5c5f77")),
                fg_muted: ColorToken::new(Color::hex("#8c8fa1")),
                fg_disabled: ColorToken::new(Color::hex("#9ca0b0")),

                accent: ColorToken::new(Color::hex("#1e66f5")),
                accent_secondary: ColorToken::new(Color::hex("#8839ef")),
                accent_hover: ColorToken::new(Color::hex("#7287fd")),
                accent_focused: ColorToken::new(Color::hex("#04a5e5")),

                success: ColorToken::new(Color::hex("#40a02b")),
                warning: ColorToken::new(Color::hex("#df8e1d")),
                error: ColorToken::new(Color::hex("#d20f39")),
                info: ColorToken::new(Color::hex("#1e66f5")),

                border: ColorToken::new(Color::hex("#ccd0da")),
                border_focused: ColorToken::new(Color::hex("#1e66f5")),
                border_error: ColorToken::new(Color::hex("#d20f39")),

                flash_success: ColorToken::new(Color::hex("#40a02b")),
                flash_error: ColorToken::new(Color::hex("#d20f39")),
            },
            color_256: light_color_set_256(),
            color_16: light_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// Nord theme.
pub fn nord() -> Theme {
    Theme {
        name: "Nord".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#2e3440")),
                bg_secondary: ColorToken::new(Color::hex("#3b4252")),
                bg_tertiary: ColorToken::new(Color::hex("#434c5e")),
                bg_hover: ColorToken::new(Color::hex("#4c566a")),
                bg_focused: ColorToken::new(Color::hex("#5e81ac")),
                bg_pressed: ColorToken::new(Color::hex("#3b4252")),
                bg_disabled: ColorToken::new(Color::hex("#2e3440")),

                fg_primary: ColorToken::new(Color::hex("#eceff4")),
                fg_secondary: ColorToken::new(Color::hex("#e5e9f0")),
                fg_muted: ColorToken::new(Color::hex("#d8dee9")),
                fg_disabled: ColorToken::new(Color::hex("#4c566a")),

                accent: ColorToken::new(Color::hex("#88c0d0")),
                accent_secondary: ColorToken::new(Color::hex("#81a1c1")),
                accent_hover: ColorToken::new(Color::hex("#8fbcbb")),
                accent_focused: ColorToken::new(Color::hex("#5e81ac")),

                success: ColorToken::new(Color::hex("#a3be8c")),
                warning: ColorToken::new(Color::hex("#ebcb8b")),
                error: ColorToken::new(Color::hex("#bf616a")),
                info: ColorToken::new(Color::hex("#88c0d0")),

                border: ColorToken::new(Color::hex("#4c566a")),
                border_focused: ColorToken::new(Color::hex("#88c0d0")),
                border_error: ColorToken::new(Color::hex("#bf616a")),

                flash_success: ColorToken::new(Color::hex("#a3be8c")),
                flash_error: ColorToken::new(Color::hex("#bf616a")),
            },
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// Solarized Dark theme.
pub fn solarized_dark() -> Theme {
    Theme {
        name: "Solarized Dark".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#002b36")),
                bg_secondary: ColorToken::new(Color::hex("#073642")),
                bg_tertiary: ColorToken::new(Color::hex("#586e75")),
                bg_hover: ColorToken::new(Color::hex("#657b83")),
                bg_focused: ColorToken::new(Color::hex("#268bd2")),
                bg_pressed: ColorToken::new(Color::hex("#073642")),
                bg_disabled: ColorToken::new(Color::hex("#002b36")),

                fg_primary: ColorToken::new(Color::hex("#839496")),
                fg_secondary: ColorToken::new(Color::hex("#93a1a1")),
                fg_muted: ColorToken::new(Color::hex("#657b83")),
                fg_disabled: ColorToken::new(Color::hex("#586e75")),

                accent: ColorToken::new(Color::hex("#268bd2")),
                accent_secondary: ColorToken::new(Color::hex("#6c71c4")),
                accent_hover: ColorToken::new(Color::hex("#2aa198")),
                accent_focused: ColorToken::new(Color::hex("#859900")),

                success: ColorToken::new(Color::hex("#859900")),
                warning: ColorToken::new(Color::hex("#b58900")),
                error: ColorToken::new(Color::hex("#dc322f")),
                info: ColorToken::new(Color::hex("#268bd2")),

                border: ColorToken::new(Color::hex("#586e75")),
                border_focused: ColorToken::new(Color::hex("#268bd2")),
                border_error: ColorToken::new(Color::hex("#dc322f")),

                flash_success: ColorToken::new(Color::hex("#859900")),
                flash_error: ColorToken::new(Color::hex("#dc322f")),
            },
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// Solarized Light theme.
pub fn solarized_light() -> Theme {
    Theme {
        name: "Solarized Light".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#fdf6e3")),
                bg_secondary: ColorToken::new(Color::hex("#eee8d5")),
                bg_tertiary: ColorToken::new(Color::hex("#93a1a1")),
                bg_hover: ColorToken::new(Color::hex("#839496")),
                bg_focused: ColorToken::new(Color::hex("#268bd2")),
                bg_pressed: ColorToken::new(Color::hex("#eee8d5")),
                bg_disabled: ColorToken::new(Color::hex("#fdf6e3")),

                fg_primary: ColorToken::new(Color::hex("#657b83")),
                fg_secondary: ColorToken::new(Color::hex("#586e75")),
                fg_muted: ColorToken::new(Color::hex("#93a1a1")),
                fg_disabled: ColorToken::new(Color::hex("#93a1a1")),

                accent: ColorToken::new(Color::hex("#268bd2")),
                accent_secondary: ColorToken::new(Color::hex("#6c71c4")),
                accent_hover: ColorToken::new(Color::hex("#2aa198")),
                accent_focused: ColorToken::new(Color::hex("#859900")),

                success: ColorToken::new(Color::hex("#859900")),
                warning: ColorToken::new(Color::hex("#b58900")),
                error: ColorToken::new(Color::hex("#dc322f")),
                info: ColorToken::new(Color::hex("#268bd2")),

                border: ColorToken::new(Color::hex("#93a1a1")),
                border_focused: ColorToken::new(Color::hex("#268bd2")),
                border_error: ColorToken::new(Color::hex("#dc322f")),

                flash_success: ColorToken::new(Color::hex("#859900")),
                flash_error: ColorToken::new(Color::hex("#dc322f")),
            },
            color_256: light_color_set_256(),
            color_16: light_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// Gruvbox Dark theme.
pub fn gruvbox_dark() -> Theme {
    Theme {
        name: "Gruvbox Dark".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#282828")),
                bg_secondary: ColorToken::new(Color::hex("#3c3836")),
                bg_tertiary: ColorToken::new(Color::hex("#504945")),
                bg_hover: ColorToken::new(Color::hex("#665c54")),
                bg_focused: ColorToken::new(Color::hex("#83a598")),
                bg_pressed: ColorToken::new(Color::hex("#3c3836")),
                bg_disabled: ColorToken::new(Color::hex("#282828")),

                fg_primary: ColorToken::new(Color::hex("#ebdbb2")),
                fg_secondary: ColorToken::new(Color::hex("#d5c4a1")),
                fg_muted: ColorToken::new(Color::hex("#a89984")),
                fg_disabled: ColorToken::new(Color::hex("#665c54")),

                accent: ColorToken::new(Color::hex("#83a598")),
                accent_secondary: ColorToken::new(Color::hex("#d3869b")),
                accent_hover: ColorToken::new(Color::hex("#8ec07c")),
                accent_focused: ColorToken::new(Color::hex("#fabd2f")),

                success: ColorToken::new(Color::hex("#b8bb26")),
                warning: ColorToken::new(Color::hex("#fabd2f")),
                error: ColorToken::new(Color::hex("#fb4934")),
                info: ColorToken::new(Color::hex("#83a598")),

                border: ColorToken::new(Color::hex("#504945")),
                border_focused: ColorToken::new(Color::hex("#83a598")),
                border_error: ColorToken::new(Color::hex("#fb4934")),

                flash_success: ColorToken::new(Color::hex("#b8bb26")),
                flash_error: ColorToken::new(Color::hex("#fb4934")),
            },
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// Dracula theme.
pub fn dracula() -> Theme {
    Theme {
        name: "Dracula".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#282a36")),
                bg_secondary: ColorToken::new(Color::hex("#44475a")),
                bg_tertiary: ColorToken::new(Color::hex("#6272a4")),
                bg_hover: ColorToken::new(Color::hex("#44475a")),
                bg_focused: ColorToken::new(Color::hex("#bd93f9")),
                bg_pressed: ColorToken::new(Color::hex("#44475a")),
                bg_disabled: ColorToken::new(Color::hex("#282a36")),

                fg_primary: ColorToken::new(Color::hex("#f8f8f2")),
                fg_secondary: ColorToken::new(Color::hex("#f8f8f2")),
                fg_muted: ColorToken::new(Color::hex("#6272a4")),
                fg_disabled: ColorToken::new(Color::hex("#44475a")),

                accent: ColorToken::new(Color::hex("#bd93f9")),
                accent_secondary: ColorToken::new(Color::hex("#ff79c6")),
                accent_hover: ColorToken::new(Color::hex("#ff79c6")),
                accent_focused: ColorToken::new(Color::hex("#8be9fd")),

                success: ColorToken::new(Color::hex("#50fa7b")),
                warning: ColorToken::new(Color::hex("#f1fa8c")),
                error: ColorToken::new(Color::hex("#ff5555")),
                info: ColorToken::new(Color::hex("#8be9fd")),

                border: ColorToken::new(Color::hex("#44475a")),
                border_focused: ColorToken::new(Color::hex("#bd93f9")),
                border_error: ColorToken::new(Color::hex("#ff5555")),

                flash_success: ColorToken::new(Color::hex("#50fa7b")),
                flash_error: ColorToken::new(Color::hex("#ff5555")),
            },
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// One Dark theme.
pub fn one_dark() -> Theme {
    Theme {
        name: "One Dark".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#282c34")),
                bg_secondary: ColorToken::new(Color::hex("#21252b")),
                bg_tertiary: ColorToken::new(Color::hex("#2c313c")),
                bg_hover: ColorToken::new(Color::hex("#3e4451")),
                bg_focused: ColorToken::new(Color::hex("#61afef")),
                bg_pressed: ColorToken::new(Color::hex("#21252b")),
                bg_disabled: ColorToken::new(Color::hex("#282c34")),

                fg_primary: ColorToken::new(Color::hex("#abb2bf")),
                fg_secondary: ColorToken::new(Color::hex("#828997")),
                fg_muted: ColorToken::new(Color::hex("#5c6370")),
                fg_disabled: ColorToken::new(Color::hex("#3e4451")),

                accent: ColorToken::new(Color::hex("#61afef")),
                accent_secondary: ColorToken::new(Color::hex("#c678dd")),
                accent_hover: ColorToken::new(Color::hex("#56b6c2")),
                accent_focused: ColorToken::new(Color::hex("#98c379")),

                success: ColorToken::new(Color::hex("#98c379")),
                warning: ColorToken::new(Color::hex("#e5c07b")),
                error: ColorToken::new(Color::hex("#e06c75")),
                info: ColorToken::new(Color::hex("#61afef")),

                border: ColorToken::new(Color::hex("#3e4451")),
                border_focused: ColorToken::new(Color::hex("#61afef")),
                border_error: ColorToken::new(Color::hex("#e06c75")),

                flash_success: ColorToken::new(Color::hex("#98c379")),
                flash_error: ColorToken::new(Color::hex("#e06c75")),
            },
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// High contrast dark theme.
pub fn high_contrast_dark() -> Theme {
    Theme {
        name: "High Contrast Dark".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#000000")),
                bg_secondary: ColorToken::new(Color::hex("#1a1a1a")),
                bg_tertiary: ColorToken::new(Color::hex("#333333")),
                bg_hover: ColorToken::new(Color::hex("#404040")),
                bg_focused: ColorToken::new(Color::hex("#0066cc")),
                bg_pressed: ColorToken::new(Color::hex("#1a1a1a")),
                bg_disabled: ColorToken::new(Color::hex("#000000")),

                fg_primary: ColorToken::new(Color::hex("#ffffff")),
                fg_secondary: ColorToken::new(Color::hex("#e0e0e0")),
                fg_muted: ColorToken::new(Color::hex("#a0a0a0")),
                fg_disabled: ColorToken::new(Color::hex("#606060")),

                accent: ColorToken::new(Color::hex("#00ccff")),
                accent_secondary: ColorToken::new(Color::hex("#ff66ff")),
                accent_hover: ColorToken::new(Color::hex("#66ffff")),
                accent_focused: ColorToken::new(Color::hex("#ffff00")),

                success: ColorToken::new(Color::hex("#00ff00")),
                warning: ColorToken::new(Color::hex("#ffff00")),
                error: ColorToken::new(Color::hex("#ff0000")),
                info: ColorToken::new(Color::hex("#00ccff")),

                border: ColorToken::new(Color::hex("#ffffff")),
                border_focused: ColorToken::new(Color::hex("#00ccff")),
                border_error: ColorToken::new(Color::hex("#ff0000")),

                flash_success: ColorToken::new(Color::hex("#00ff00")),
                flash_error: ColorToken::new(Color::hex("#ff0000")),
            },
            color_256: dark_color_set_256(),
            color_16: dark_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

/// High contrast light theme.
pub fn high_contrast_light() -> Theme {
    Theme {
        name: "High Contrast Light".to_string(),
        extends: None,
        colors: ColorPalette {
            true_color: ColorSet {
                bg_primary: ColorToken::new(Color::hex("#ffffff")),
                bg_secondary: ColorToken::new(Color::hex("#e6e6e6")),
                bg_tertiary: ColorToken::new(Color::hex("#cccccc")),
                bg_hover: ColorToken::new(Color::hex("#b3b3b3")),
                bg_focused: ColorToken::new(Color::hex("#0066cc")),
                bg_pressed: ColorToken::new(Color::hex("#e6e6e6")),
                bg_disabled: ColorToken::new(Color::hex("#ffffff")),

                fg_primary: ColorToken::new(Color::hex("#000000")),
                fg_secondary: ColorToken::new(Color::hex("#1a1a1a")),
                fg_muted: ColorToken::new(Color::hex("#404040")),
                fg_disabled: ColorToken::new(Color::hex("#808080")),

                accent: ColorToken::new(Color::hex("#0000cc")),
                accent_secondary: ColorToken::new(Color::hex("#660066")),
                accent_hover: ColorToken::new(Color::hex("#0000ff")),
                accent_focused: ColorToken::new(Color::hex("#006600")),

                success: ColorToken::new(Color::hex("#006600")),
                warning: ColorToken::new(Color::hex("#996600")),
                error: ColorToken::new(Color::hex("#cc0000")),
                info: ColorToken::new(Color::hex("#0000cc")),

                border: ColorToken::new(Color::hex("#000000")),
                border_focused: ColorToken::new(Color::hex("#0000cc")),
                border_error: ColorToken::new(Color::hex("#cc0000")),

                flash_success: ColorToken::new(Color::hex("#006600")),
                flash_error: ColorToken::new(Color::hex("#cc0000")),
            },
            color_256: light_color_set_256(),
            color_16: light_color_set_16(),
        },
        styles: StyleMap::default(),
        variant: ThemeVariant::Comfortable,
        animation: AnimationConfig::default(),
        syntax: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_themes() {
        let themes = builtin_themes();
        assert!(!themes.is_empty());
        assert!(themes.contains_key("default-dark"));
        assert!(themes.contains_key("catppuccin-mocha"));
        assert!(themes.contains_key("nord"));
    }

    #[test]
    fn test_default_dark() {
        let theme = default_dark();
        assert_eq!(theme.name, "Default Dark");
    }

    #[test]
    fn test_all_themes_have_palettes() {
        let themes = builtin_themes();
        for (name, theme) in themes {
            // Just verify they don't panic when accessed
            let _ = &theme.colors.true_color.accent;
            let _ = &theme.colors.color_256.accent;
            let _ = &theme.colors.color_16.accent;
        }
    }
}
