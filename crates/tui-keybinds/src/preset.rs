//! Built-in keymap presets.

use crate::binding::{KeyBinding, KeySequence};
use crate::context::Context;
use crate::keymap::{ActionId, Keymap};
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

/// Built-in keymap presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeymapPreset {
    /// Modern, intuitive keybindings
    #[default]
    Default,
    /// Vim-like keybindings
    Vim,
    /// Emacs-like keybindings
    Emacs,
}

impl KeymapPreset {
    /// Load the preset keymap.
    pub fn load(&self) -> Keymap {
        match self {
            Self::Default => default_keymap(),
            Self::Vim => vim_keymap(),
            Self::Emacs => emacs_keymap(),
        }
    }
}

/// Default modern keymap.
fn default_keymap() -> Keymap {
    let mut keymap = Keymap::new();

    // Global bindings
    keymap.bind(
        KeySequence::single(KeyBinding::ctrl('q')),
        "quit",
    );
    keymap.bind(
        KeySequence::single(KeyBinding::ctrl('s')),
        "save",
    );
    keymap.bind(
        KeySequence::single(KeyBinding::ctrl('z')),
        "undo",
    );
    keymap.bind(
        KeySequence::single(KeyBinding::new(KeyCode::Char('z'), crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::SHIFT)),
        "redo",
    );
    keymap.bind(
        KeySequence::single(KeyBinding::ctrl('p')),
        "command_palette",
    );
    keymap.bind(
        KeySequence::single(KeyBinding::ctrl('f')),
        "search",
    );
    keymap.bind(
        KeySequence::single(KeyBinding::key(KeyCode::F(1))),
        "help",
    );
    keymap.bind(
        KeySequence::single(KeyBinding::key(KeyCode::Char('?'))),
        "help",
    );

    // Navigation
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Up)),
        "move_up",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Down)),
        "move_down",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Left)),
        "move_left",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Right)),
        "move_right",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Home)),
        "move_start",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::End)),
        "move_end",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::PageUp)),
        "page_up",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::PageDown)),
        "page_down",
    );

    // Selection
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Enter)),
        "select",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Esc)),
        "cancel",
    );

    // Dialog context
    keymap.bind_in_context(
        Context::Dialog,
        KeySequence::single(KeyBinding::key(KeyCode::Enter)),
        "confirm",
    );
    keymap.bind_in_context(
        Context::Dialog,
        KeySequence::single(KeyBinding::key(KeyCode::Esc)),
        "cancel",
    );
    keymap.bind_in_context(
        Context::Dialog,
        KeySequence::single(KeyBinding::key(KeyCode::Tab)),
        "next_field",
    );
    keymap.bind_in_context(
        Context::Dialog,
        KeySequence::single(KeyBinding::new(KeyCode::Tab, crossterm::event::KeyModifiers::SHIFT)),
        "prev_field",
    );

    keymap
}

/// Vim-like keymap.
fn vim_keymap() -> Keymap {
    let mut keymap = Keymap::new();

    // Global
    keymap.bind(
        KeySequence::single(KeyBinding::ctrl('p')),
        "command_palette",
    );

    // Normal mode navigation
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('h'))),
        "move_left",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('j'))),
        "move_down",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('k'))),
        "move_up",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('l'))),
        "move_right",
    );

    // Vim motions
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::from_keys(vec![
            KeyBinding::key(KeyCode::Char('g')),
            KeyBinding::key(KeyCode::Char('g')),
        ]),
        "go_to_start",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::new(KeyCode::Char('g'), crossterm::event::KeyModifiers::SHIFT)),
        "go_to_end",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('u')),
        "page_up",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('d')),
        "page_down",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('w'))),
        "word_forward",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('b'))),
        "word_backward",
    );

    // Actions
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('i'))),
        "enter_insert",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('a'))),
        "append",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('o'))),
        "open_below",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::new(KeyCode::Char('o'), crossterm::event::KeyModifiers::SHIFT)),
        "open_above",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('d'))),
        "delete",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('y'))),
        "yank",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('p'))),
        "paste",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('u'))),
        "undo",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('r')),
        "redo",
    );

    // Visual mode
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('v'))),
        "enter_visual",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::new(KeyCode::Char('v'), crossterm::event::KeyModifiers::SHIFT)),
        "enter_visual_line",
    );

    // Search
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('/'))),
        "search",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('n'))),
        "search_next",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::new(KeyCode::Char('n'), crossterm::event::KeyModifiers::SHIFT)),
        "search_prev",
    );

    // Command mode
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char(':'))),
        "enter_command",
    );

    // Quit
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::key(KeyCode::Char('q'))),
        "quit",
    );

    // Insert mode
    keymap.bind_in_context(
        Context::Insert,
        KeySequence::single(KeyBinding::key(KeyCode::Esc)),
        "exit_insert",
    );

    // Visual mode
    keymap.bind_in_context(
        Context::Visual,
        KeySequence::single(KeyBinding::key(KeyCode::Esc)),
        "exit_visual",
    );
    keymap.bind_in_context(
        Context::Visual,
        KeySequence::single(KeyBinding::key(KeyCode::Char('d'))),
        "delete_selection",
    );
    keymap.bind_in_context(
        Context::Visual,
        KeySequence::single(KeyBinding::key(KeyCode::Char('y'))),
        "yank_selection",
    );

    keymap
}

/// Emacs-like keymap.
fn emacs_keymap() -> Keymap {
    let mut keymap = Keymap::new();

    // Global
    keymap.bind(
        KeySequence::from_keys(vec![
            KeyBinding::ctrl('x'),
            KeyBinding::ctrl('c'),
        ]),
        "quit",
    );
    keymap.bind(
        KeySequence::from_keys(vec![
            KeyBinding::ctrl('x'),
            KeyBinding::ctrl('s'),
        ]),
        "save",
    );
    keymap.bind(
        KeySequence::from_keys(vec![
            KeyBinding::ctrl('x'),
            KeyBinding::ctrl('f'),
        ]),
        "find_file",
    );

    // Navigation
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('p')),
        "move_up",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('n')),
        "move_down",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('b')),
        "move_left",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('f')),
        "move_right",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('a')),
        "move_start_of_line",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('e')),
        "move_end_of_line",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::alt('v')),
        "page_up",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('v')),
        "page_down",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::alt('<')),
        "go_to_start",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::alt('>')),
        "go_to_end",
    );

    // Editing
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('d')),
        "delete_char",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('k')),
        "kill_line",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('y')),
        "yank",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('/')),
        "undo",
    );

    // Search
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('s')),
        "isearch_forward",
    );
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('r')),
        "isearch_backward",
    );

    // Cancel
    keymap.bind_in_context(
        Context::Normal,
        KeySequence::single(KeyBinding::ctrl('g')),
        "cancel",
    );

    keymap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preset() {
        let keymap = KeymapPreset::Default.load();
        assert!(!keymap.global.is_empty());
    }

    #[test]
    fn test_vim_preset() {
        let keymap = KeymapPreset::Vim.load();

        // Check hjkl navigation
        let j = KeySequence::single(KeyBinding::key(KeyCode::Char('j')));
        let action = keymap.get_action(&j, &Context::Normal);
        assert_eq!(action.map(|a| a.0.as_str()), Some("move_down"));
    }

    #[test]
    fn test_emacs_preset() {
        let keymap = KeymapPreset::Emacs.load();

        // Check C-x C-c quit
        let quit = KeySequence::from_keys(vec![
            KeyBinding::ctrl('x'),
            KeyBinding::ctrl('c'),
        ]);
        let action = keymap.get_action(&quit, &Context::Normal);
        assert_eq!(action.map(|a| a.0.as_str()), Some("quit"));
    }
}
