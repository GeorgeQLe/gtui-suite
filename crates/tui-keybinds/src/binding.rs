//! Key binding types.

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A single key binding (key + modifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    /// The key code
    pub key: KeyCode,
    /// Modifier keys (Ctrl, Alt, Shift)
    #[serde(default = "default_modifiers")]
    pub modifiers: KeyModifiers,
}

fn default_modifiers() -> KeyModifiers {
    KeyModifiers::NONE
}

impl KeyBinding {
    /// Create a new key binding.
    pub fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create a key binding with no modifiers.
    pub fn key(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::NONE,
        }
    }

    /// Create a Ctrl+key binding.
    pub fn ctrl(c: char) -> Self {
        Self {
            key: KeyCode::Char(c.to_ascii_lowercase()),
            modifiers: KeyModifiers::CONTROL,
        }
    }

    /// Create an Alt+key binding.
    pub fn alt(c: char) -> Self {
        Self {
            key: KeyCode::Char(c.to_ascii_lowercase()),
            modifiers: KeyModifiers::ALT,
        }
    }

    /// Check if this matches a crossterm key event.
    pub fn matches(&self, event: &crossterm::event::KeyEvent) -> bool {
        self.key == event.code && self.modifiers == event.modifiers
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }

        let key_str = match self.key {
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Escape".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", self.key),
        };

        parts.push(&key_str);

        if f.alternate() {
            // Symbolic format
            write!(f, "{}", parts.join(""))
        } else {
            write!(f, "{}", parts.join("+"))
        }
    }
}

/// A sequence of key bindings (for chords like 'gg' or 'C-x C-s').
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct KeySequence {
    /// The keys in this sequence
    pub keys: Vec<KeyBinding>,
}

impl KeySequence {
    /// Create an empty key sequence.
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Create a sequence from a single key.
    pub fn single(key: KeyBinding) -> Self {
        Self { keys: vec![key] }
    }

    /// Create a sequence from multiple keys.
    pub fn from_keys(keys: Vec<KeyBinding>) -> Self {
        Self { keys }
    }

    /// Add a key to the sequence.
    pub fn push(&mut self, key: KeyBinding) {
        self.keys.push(key);
    }

    /// Check if this sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Get the length of this sequence.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Check if this sequence starts with another sequence.
    pub fn starts_with(&self, prefix: &KeySequence) -> bool {
        if prefix.len() > self.len() {
            return false;
        }
        self.keys.iter().zip(prefix.keys.iter()).all(|(a, b)| a == b)
    }

    /// Check if this is a prefix of another sequence.
    pub fn is_prefix_of(&self, other: &KeySequence) -> bool {
        other.starts_with(self)
    }
}

impl fmt::Display for KeySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.keys.iter().map(|k| k.to_string()).collect();
        write!(f, "{}", parts.join(" "))
    }
}

impl From<KeyBinding> for KeySequence {
    fn from(key: KeyBinding) -> Self {
        Self::single(key)
    }
}

impl From<Vec<KeyBinding>> for KeySequence {
    fn from(keys: Vec<KeyBinding>) -> Self {
        Self { keys }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding_display() {
        let binding = KeyBinding::ctrl('s');
        assert_eq!(binding.to_string(), "Ctrl+S");

        let binding = KeyBinding::key(KeyCode::Enter);
        assert_eq!(binding.to_string(), "Enter");

        let binding = KeyBinding::new(KeyCode::Char('x'), KeyModifiers::ALT | KeyModifiers::SHIFT);
        assert_eq!(binding.to_string(), "Alt+Shift+X");
    }

    #[test]
    fn test_key_sequence() {
        let mut seq = KeySequence::new();
        seq.push(KeyBinding::key(KeyCode::Char('g')));
        seq.push(KeyBinding::key(KeyCode::Char('g')));

        assert_eq!(seq.len(), 2);
        assert_eq!(seq.to_string(), "G G");
    }

    #[test]
    fn test_sequence_prefix() {
        let full = KeySequence::from_keys(vec![
            KeyBinding::key(KeyCode::Char('g')),
            KeyBinding::key(KeyCode::Char('g')),
        ]);

        let prefix = KeySequence::single(KeyBinding::key(KeyCode::Char('g')));

        assert!(full.starts_with(&prefix));
        assert!(prefix.is_prefix_of(&full));
    }
}
