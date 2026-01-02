//! Key notation parser.

use crate::binding::KeyBinding;
use crossterm::event::{KeyCode, KeyModifiers};

/// Parse a key notation string into a KeyBinding.
///
/// Supported formats:
/// - `"ctrl+s"`, `"C-s"`, `"<C-s>"`, `"Ctrl+S"` - Ctrl+S
/// - `"alt+x"`, `"A-x"`, `"M-x"` - Alt+X (M for Meta, common in emacs)
/// - `"shift+tab"`, `"S-tab"` - Shift+Tab
/// - `"enter"`, `"escape"`, `"tab"`, `"space"` - Special keys
/// - `"up"`, `"down"`, `"left"`, `"right"` - Arrow keys
/// - `"f1"` through `"f12"` - Function keys
pub fn parse_key(s: &str) -> Result<KeyBinding, ParseError> {
    let s = s.trim();

    // Handle angle bracket notation <C-x>
    let s = s.strip_prefix('<').and_then(|s| s.strip_suffix('>')).unwrap_or(s);

    let mut modifiers = KeyModifiers::NONE;
    let mut parts: Vec<&str> = if s.contains('+') {
        s.split('+').collect()
    } else if s.contains('-') && s.len() > 2 {
        // Handle vim/emacs notation like C-x, M-x
        s.split('-').collect()
    } else {
        vec![s]
    };

    if parts.is_empty() {
        return Err(ParseError::Empty);
    }

    // Process modifiers (all but last part)
    while parts.len() > 1 {
        let modifier = parts.remove(0).to_lowercase();
        match modifier.as_str() {
            "ctrl" | "control" | "c" => modifiers |= KeyModifiers::CONTROL,
            "alt" | "option" | "a" | "m" => modifiers |= KeyModifiers::ALT, // M for Meta
            "shift" | "s" => modifiers |= KeyModifiers::SHIFT,
            "super" | "cmd" | "command" | "win" => {
                // Super/Cmd not directly supported by crossterm
                // Could be handled specially by apps
            }
            _ => return Err(ParseError::UnknownModifier(modifier)),
        }
    }

    // Parse the key (last part)
    let key_str = parts[0].to_lowercase();
    let key = match key_str.as_str() {
        // Special keys
        "enter" | "return" | "cr" => KeyCode::Enter,
        "escape" | "esc" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backspace" | "bs" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "insert" | "ins" => KeyCode::Insert,
        "space" => KeyCode::Char(' '),

        // Arrow keys
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,

        // Navigation
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdn" | "pgdown" => KeyCode::PageDown,

        // Function keys
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),

        // Single character
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            // Shift modifier for uppercase handled implicitly
            KeyCode::Char(c)
        }

        _ => return Err(ParseError::UnknownKey(key_str)),
    };

    Ok(KeyBinding { key, modifiers })
}

/// Error parsing a key notation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Empty input
    Empty,
    /// Unknown modifier
    UnknownModifier(String),
    /// Unknown key
    UnknownKey(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty key notation"),
            Self::UnknownModifier(m) => write!(f, "unknown modifier: {}", m),
            Self::UnknownKey(k) => write!(f, "unknown key: {}", k),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let key = parse_key("a").unwrap();
        assert_eq!(key.key, KeyCode::Char('a'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_ctrl() {
        let key = parse_key("ctrl+s").unwrap();
        assert_eq!(key.key, KeyCode::Char('s'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));

        let key = parse_key("C-s").unwrap();
        assert_eq!(key.key, KeyCode::Char('s'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));

        let key = parse_key("<C-s>").unwrap();
        assert_eq!(key.key, KeyCode::Char('s'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_alt() {
        let key = parse_key("alt+x").unwrap();
        assert!(key.modifiers.contains(KeyModifiers::ALT));

        let key = parse_key("M-x").unwrap(); // Emacs Meta
        assert!(key.modifiers.contains(KeyModifiers::ALT));
    }

    #[test]
    fn test_parse_combined() {
        let key = parse_key("ctrl+shift+s").unwrap();
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert!(key.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_parse_special() {
        assert_eq!(parse_key("enter").unwrap().key, KeyCode::Enter);
        assert_eq!(parse_key("escape").unwrap().key, KeyCode::Esc);
        assert_eq!(parse_key("tab").unwrap().key, KeyCode::Tab);
        assert_eq!(parse_key("space").unwrap().key, KeyCode::Char(' '));
    }

    #[test]
    fn test_parse_arrows() {
        assert_eq!(parse_key("up").unwrap().key, KeyCode::Up);
        assert_eq!(parse_key("down").unwrap().key, KeyCode::Down);
        assert_eq!(parse_key("left").unwrap().key, KeyCode::Left);
        assert_eq!(parse_key("right").unwrap().key, KeyCode::Right);
    }

    #[test]
    fn test_parse_function_keys() {
        assert_eq!(parse_key("f1").unwrap().key, KeyCode::F(1));
        assert_eq!(parse_key("f12").unwrap().key, KeyCode::F(12));
    }

    #[test]
    fn test_parse_case_insensitive() {
        let key1 = parse_key("Ctrl+S").unwrap();
        let key2 = parse_key("ctrl+s").unwrap();
        assert_eq!(key1.key, key2.key);
        assert_eq!(key1.modifiers, key2.modifiers);
    }

    #[test]
    fn test_parse_error() {
        assert!(parse_key("").is_err());
        assert!(parse_key("unknownmod+x").is_err());
    }
}
