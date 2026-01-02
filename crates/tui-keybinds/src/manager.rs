//! Keybind manager.

use crate::binding::{KeyBinding, KeySequence};
use crate::conflict::{Conflict, ConflictReport, ConflictSeverity};
use crate::context::{ConditionState, Context};
use crate::display::KeyDisplayConfig;
use crate::keymap::{ActionId, Keymap};
use crate::macros::MacroManager;
use crate::preset::KeymapPreset;
use crate::scheme::KeyScheme;

use crossterm::event::KeyEvent;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;
use thiserror::Error;

/// Error loading keybind configuration.
#[derive(Debug, Error)]
pub enum ConflictError {
    #[error("keybinding conflicts detected:\n{0}")]
    Conflicts(ConflictReport),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Keybind manager.
pub struct KeybindManager {
    /// Key input scheme
    scheme: KeyScheme,
    /// Base keymap from preset
    preset_keymap: Keymap,
    /// User keymap overrides
    user_keymap: Keymap,
    /// Macros
    pub macros: MacroManager,
    /// Current context
    context: Context,
    /// Display configuration
    display: KeyDisplayConfig,
    /// Current key sequence being built
    pending_keys: Vec<KeyBinding>,
    /// When the last key was pressed
    last_key_time: Option<Instant>,
    /// Keys that were explicitly unbound
    unbound_keys: HashSet<KeySequence>,
}

impl KeybindManager {
    /// Create a new keybind manager.
    pub fn new(scheme: KeyScheme, preset: Option<KeymapPreset>) -> Self {
        let preset_keymap = preset.map(|p| p.load()).unwrap_or_default();

        Self {
            scheme,
            preset_keymap,
            user_keymap: Keymap::new(),
            macros: MacroManager::new(),
            context: Context::Normal,
            display: KeyDisplayConfig::default(),
            pending_keys: Vec::new(),
            last_key_time: None,
            unbound_keys: HashSet::new(),
        }
    }

    /// Set the key scheme.
    pub fn set_scheme(&mut self, scheme: KeyScheme) {
        self.scheme = scheme;
        self.clear_pending();
    }

    /// Set the display configuration.
    pub fn set_display(&mut self, config: KeyDisplayConfig) {
        self.display = config;
    }

    /// Set the current context.
    pub fn set_context(&mut self, context: Context) {
        self.context = context;
        self.clear_pending();
    }

    /// Get the current context.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Add a user keybinding.
    pub fn bind(&mut self, keys: impl Into<KeySequence>, action: impl Into<ActionId>) {
        self.user_keymap.bind(keys, action);
    }

    /// Add a context-specific user keybinding.
    pub fn bind_in_context(
        &mut self,
        context: Context,
        keys: impl Into<KeySequence>,
        action: impl Into<ActionId>,
    ) {
        self.user_keymap.bind_in_context(context, keys, action);
    }

    /// Unbind a key sequence.
    pub fn unbind(&mut self, keys: impl Into<KeySequence>) {
        let keys = keys.into();
        self.unbound_keys.insert(keys.clone());
        self.user_keymap.unbind(&keys);
    }

    /// Handle a key event.
    ///
    /// Returns the action to execute, if any.
    pub fn handle_key(&mut self, event: KeyEvent) -> Option<ActionId> {
        let binding = KeyBinding::new(event.code, event.modifiers);

        // Check timeout
        if let Some(last_time) = self.last_key_time {
            let timeout_ms = self.scheme.timeout_ms();
            if timeout_ms > 0 && last_time.elapsed().as_millis() > timeout_ms as u128 {
                self.clear_pending();
            }
        }

        self.pending_keys.push(binding);
        self.last_key_time = Some(Instant::now());

        let sequence = KeySequence::from_keys(self.pending_keys.clone());

        // Check if this is explicitly unbound
        if self.unbound_keys.contains(&sequence) {
            self.clear_pending();
            return None;
        }

        // Try to get an action
        if let Some(action) = self.get_action(&sequence) {
            self.clear_pending();
            return Some(action.clone());
        }

        // Check if this could be a prefix of a longer sequence
        if self.scheme.supports_sequences() && self.is_prefix(&sequence) {
            // Wait for more keys
            return None;
        }

        // No match and not a prefix - clear and try single key
        if self.pending_keys.len() > 1 {
            self.pending_keys.clear();
            self.pending_keys.push(KeyBinding::new(event.code, event.modifiers));
            let sequence = KeySequence::from_keys(self.pending_keys.clone());

            if let Some(action) = self.get_action(&sequence) {
                self.clear_pending();
                return Some(action.clone());
            }
        }

        self.clear_pending();
        None
    }

    /// Get the action for a key sequence.
    fn get_action(&self, sequence: &KeySequence) -> Option<&ActionId> {
        // User keymap takes precedence
        if let Some(action) = self.user_keymap.get_action(sequence, &self.context) {
            return Some(action);
        }

        // Fall back to preset
        self.preset_keymap.get_action(sequence, &self.context)
    }

    /// Check if a sequence could be a prefix of a bound sequence.
    fn is_prefix(&self, prefix: &KeySequence) -> bool {
        // Check user keymap
        for (seq, _) in self.user_keymap.bindings_for_context(&self.context) {
            if seq.starts_with(prefix) && seq.len() > prefix.len() {
                return true;
            }
        }

        // Check preset keymap
        for (seq, _) in self.preset_keymap.bindings_for_context(&self.context) {
            if seq.starts_with(prefix) && seq.len() > prefix.len() {
                return true;
            }
        }

        false
    }

    /// Clear pending key sequence.
    fn clear_pending(&mut self) {
        self.pending_keys.clear();
        self.last_key_time = None;
    }

    /// Get the key sequence(s) bound to an action.
    pub fn get_binding_for(&self, action_id: &str) -> Vec<KeySequence> {
        let mut bindings = Vec::new();

        // Check user keymap
        for seq in self.user_keymap.get_bindings_for_action(action_id, &self.context) {
            bindings.push(seq.clone());
        }

        // Check preset (only if not overridden)
        for seq in self.preset_keymap.get_bindings_for_action(action_id, &self.context) {
            if !bindings.iter().any(|b| b == seq) {
                bindings.push(seq.clone());
            }
        }

        bindings
    }

    /// Format a key sequence for display.
    pub fn format_sequence(&self, sequence: &KeySequence) -> String {
        sequence
            .keys
            .iter()
            .map(|k| self.format_key(k))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format a single key binding for display.
    pub fn format_key(&self, key: &KeyBinding) -> String {
        let mut parts = Vec::new();

        if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            parts.push(self.display.format_modifier("ctrl").to_string());
        }
        if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
            parts.push(self.display.format_modifier("alt").to_string());
        }
        if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            parts.push(self.display.format_modifier("shift").to_string());
        }

        let key_str = match key.key {
            crossterm::event::KeyCode::Char(c) => c.to_uppercase().to_string(),
            crossterm::event::KeyCode::Enter => self.display.format_key("enter"),
            crossterm::event::KeyCode::Esc => self.display.format_key("escape"),
            crossterm::event::KeyCode::Tab => self.display.format_key("tab"),
            crossterm::event::KeyCode::Backspace => self.display.format_key("backspace"),
            crossterm::event::KeyCode::Delete => self.display.format_key("delete"),
            crossterm::event::KeyCode::Up => self.display.format_key("up"),
            crossterm::event::KeyCode::Down => self.display.format_key("down"),
            crossterm::event::KeyCode::Left => self.display.format_key("left"),
            crossterm::event::KeyCode::Right => self.display.format_key("right"),
            crossterm::event::KeyCode::Home => self.display.format_key("home"),
            crossterm::event::KeyCode::End => self.display.format_key("end"),
            crossterm::event::KeyCode::PageUp => self.display.format_key("pageup"),
            crossterm::event::KeyCode::PageDown => self.display.format_key("pagedown"),
            crossterm::event::KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", key.key),
        };

        parts.push(key_str);
        parts.join("+")
    }

    /// Check for conflicts between keymaps.
    pub fn check_conflicts(&self) -> ConflictReport {
        let mut report = ConflictReport::new();
        let mut seen: HashMap<(KeySequence, Option<Context>), Vec<(String, ActionId)>> =
            HashMap::new();

        // Collect all bindings
        for (seq, action) in &self.preset_keymap.global {
            seen.entry((seq.clone(), None))
                .or_default()
                .push(("preset".to_string(), action.clone()));
        }

        for (ctx, bindings) in &self.preset_keymap.contexts {
            for (seq, action) in bindings {
                seen.entry((seq.clone(), Some(ctx.clone())))
                    .or_default()
                    .push(("preset".to_string(), action.clone()));
            }
        }

        for (seq, action) in &self.user_keymap.global {
            seen.entry((seq.clone(), None))
                .or_default()
                .push(("user".to_string(), action.clone()));
        }

        for (ctx, bindings) in &self.user_keymap.contexts {
            for (seq, action) in bindings {
                seen.entry((seq.clone(), Some(ctx.clone())))
                    .or_default()
                    .push(("user".to_string(), action.clone()));
            }
        }

        // Report conflicts
        for ((seq, ctx), actions) in seen {
            if actions.len() > 1 {
                let severity = if actions.iter().all(|(src, _)| src == "user") {
                    ConflictSeverity::Error
                } else {
                    ConflictSeverity::Warning // User override of preset is ok
                };

                // Only report as error if same action from same source
                if actions.iter().map(|(src, _)| src).collect::<HashSet<_>>().len() == 1 {
                    let mut conflict = Conflict::new(seq, ctx, severity);
                    for (src, action) in actions {
                        conflict.add_action(src, action);
                    }
                    report.add(conflict);
                }
            }
        }

        report
    }
}

impl Default for KeybindManager {
    fn default() -> Self {
        Self::new(KeyScheme::default(), Some(KeymapPreset::Default))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_manager_creation() {
        let manager = KeybindManager::default();
        assert!(matches!(manager.context(), Context::Normal));
    }

    #[test]
    fn test_handle_key() {
        let mut manager = KeybindManager::default();

        // Ctrl+Q should quit in default preset
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let action = manager.handle_key(event);
        assert_eq!(action.map(|a| a.0), Some("quit".to_string()));
    }

    #[test]
    fn test_user_override() {
        let mut manager = KeybindManager::default();

        // Override Ctrl+S
        manager.bind(
            KeySequence::single(KeyBinding::ctrl('s')),
            "custom_save",
        );

        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = manager.handle_key(event);
        assert_eq!(action.map(|a| a.0), Some("custom_save".to_string()));
    }

    #[test]
    fn test_context_switching() {
        let mut manager = KeybindManager::new(KeyScheme::default(), Some(KeymapPreset::Vim));

        // In normal mode, 'j' is move_down
        manager.set_context(Context::Normal);
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = manager.handle_key(event);
        assert_eq!(action.map(|a| a.0), Some("move_down".to_string()));

        // In insert mode, 'j' might be different
        manager.set_context(Context::Insert);
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let action = manager.handle_key(event);
        // No special binding in insert mode
        assert!(action.is_none());
    }

    #[test]
    fn test_unbind() {
        let mut manager = KeybindManager::default();

        // First verify Ctrl+S works
        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = manager.handle_key(event);
        assert!(action.is_some());

        // Unbind it
        manager.unbind(KeySequence::single(KeyBinding::ctrl('s')));

        // Now it should return None
        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = manager.handle_key(event);
        assert!(action.is_none());
    }
}
