//! Key display configuration.

use serde::{Deserialize, Serialize};

/// Format for displaying key bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyDisplayFormat {
    /// Unicode symbols: ⌘S, ⌃P, ⇧Tab
    Symbolic,
    /// Text labels: Ctrl+S, Alt+P, Shift+Tab
    #[default]
    Text,
}

/// Configuration for key display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDisplayConfig {
    /// Display format
    pub format: KeyDisplayFormat,
    /// Fall back to text if symbolic fails
    pub fallback_to_text: bool,
}

impl Default for KeyDisplayConfig {
    fn default() -> Self {
        Self {
            format: KeyDisplayFormat::Text,
            fallback_to_text: true,
        }
    }
}

impl KeyDisplayConfig {
    /// Create a symbolic display config.
    pub fn symbolic() -> Self {
        Self {
            format: KeyDisplayFormat::Symbolic,
            fallback_to_text: true,
        }
    }

    /// Create a text display config.
    pub fn text() -> Self {
        Self {
            format: KeyDisplayFormat::Text,
            fallback_to_text: false,
        }
    }

    /// Format a modifier key.
    pub fn format_modifier(&self, name: &str) -> &'static str {
        match (self.format, name.to_lowercase().as_str()) {
            (KeyDisplayFormat::Symbolic, "ctrl" | "control") => "\u{2303}",
            (KeyDisplayFormat::Symbolic, "alt" | "option") => "\u{2325}",
            (KeyDisplayFormat::Symbolic, "shift") => "\u{21e7}",
            (KeyDisplayFormat::Symbolic, "super" | "cmd" | "command") => "\u{2318}",
            (_, "ctrl" | "control") => "Ctrl",
            (_, "alt" | "option") => "Alt",
            (_, "shift") => "Shift",
            (_, "super" | "cmd" | "command") => "Super",
            _ => name,
        }
    }

    /// Format a key name.
    pub fn format_key(&self, key: &str) -> String {
        match self.format {
            KeyDisplayFormat::Symbolic => match key.to_lowercase().as_str() {
                "enter" | "return" => "\u{23ce}".to_string(),
                "escape" | "esc" => "\u{238b}".to_string(),
                "tab" => "\u{21e5}".to_string(),
                "backspace" => "\u{232b}".to_string(),
                "delete" => "\u{2326}".to_string(),
                "space" => "\u{2423}".to_string(),
                "up" => "\u{2191}".to_string(),
                "down" => "\u{2193}".to_string(),
                "left" => "\u{2190}".to_string(),
                "right" => "\u{2192}".to_string(),
                "home" => "\u{21f1}".to_string(),
                "end" => "\u{21f2}".to_string(),
                "pageup" => "\u{21de}".to_string(),
                "pagedown" => "\u{21df}".to_string(),
                _ => key.to_uppercase(),
            },
            KeyDisplayFormat::Text => match key.to_lowercase().as_str() {
                "enter" | "return" => "Enter".to_string(),
                "escape" | "esc" => "Escape".to_string(),
                "tab" => "Tab".to_string(),
                "backspace" => "Backspace".to_string(),
                "delete" => "Delete".to_string(),
                "space" => "Space".to_string(),
                "up" => "Up".to_string(),
                "down" => "Down".to_string(),
                "left" => "Left".to_string(),
                "right" => "Right".to_string(),
                "home" => "Home".to_string(),
                "end" => "End".to_string(),
                "pageup" => "PageUp".to_string(),
                "pagedown" => "PageDown".to_string(),
                _ => key.to_uppercase(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_format() {
        let config = KeyDisplayConfig::text();

        assert_eq!(config.format_modifier("ctrl"), "Ctrl");
        assert_eq!(config.format_key("enter"), "Enter");
        assert_eq!(config.format_key("a"), "A");
    }

    #[test]
    fn test_symbolic_format() {
        let config = KeyDisplayConfig::symbolic();

        assert_eq!(config.format_modifier("ctrl"), "\u{2303}");
        assert_eq!(config.format_key("enter"), "\u{23ce}");
        assert_eq!(config.format_key("up"), "\u{2191}");
    }
}
