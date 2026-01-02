//! # tui-keybinds
//!
//! Keybinding configuration and management for the TUI Suite.
//!
//! ## Features
//!
//! - Multiple key schemes (simple modifiers, leader key, full chords)
//! - Context-aware bindings with expression syntax
//! - Macro recording and playback
//! - Conflict detection
//! - Built-in vim, emacs, and default presets

mod binding;
mod conflict;
mod context;
mod display;
mod keymap;
mod macros;
mod manager;
mod parser;
mod preset;
mod scheme;

pub use binding::{KeyBinding, KeySequence};
pub use conflict::{Conflict, ConflictReport, ConflictSeverity};
pub use context::{Context, ContextCondition};
pub use display::{KeyDisplayConfig, KeyDisplayFormat};
pub use keymap::{Action, ActionGroup, Keymap};
pub use macros::{Macro, MacroManager};
pub use manager::{ConflictError, KeybindManager};
pub use parser::parse_key;
pub use preset::KeymapPreset;
pub use scheme::KeyScheme;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Helper to create a key binding from modifiers and key.
pub fn key(code: KeyCode) -> KeyBinding {
    KeyBinding {
        key: code,
        modifiers: KeyModifiers::NONE,
    }
}

/// Helper to create a Ctrl+key binding.
pub fn ctrl(c: char) -> KeyBinding {
    KeyBinding {
        key: KeyCode::Char(c.to_ascii_lowercase()),
        modifiers: KeyModifiers::CONTROL,
    }
}

/// Helper to create an Alt+key binding.
pub fn alt(c: char) -> KeyBinding {
    KeyBinding {
        key: KeyCode::Char(c.to_ascii_lowercase()),
        modifiers: KeyModifiers::ALT,
    }
}

/// Helper to create a Shift+key binding.
pub fn shift(code: KeyCode) -> KeyBinding {
    KeyBinding {
        key: code,
        modifiers: KeyModifiers::SHIFT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_helpers() {
        let k = key(KeyCode::Enter);
        assert_eq!(k.key, KeyCode::Enter);
        assert_eq!(k.modifiers, KeyModifiers::NONE);

        let c = ctrl('s');
        assert!(c.modifiers.contains(KeyModifiers::CONTROL));

        let a = alt('x');
        assert!(a.modifiers.contains(KeyModifiers::ALT));
    }
}
