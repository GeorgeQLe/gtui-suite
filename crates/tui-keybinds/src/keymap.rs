//! Keymap and action definitions.

use crate::binding::KeySequence;
use crate::context::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A named action that can be triggered by a keybinding.
pub trait Action: Send + Sync {
    /// Unique identifier for this action.
    fn id(&self) -> &str;

    /// Human-readable label.
    fn label(&self) -> &str;

    /// Description for help screen.
    fn description(&self) -> &str;

    /// Optional group for help screen organization.
    fn group(&self) -> Option<&str> {
        None
    }
}

/// A simple string-based action for configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionId(pub String);

impl ActionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for ActionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ActionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A keymap containing bindings for actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Keymap {
    /// Global bindings (apply in all contexts)
    #[serde(default)]
    pub global: HashMap<KeySequence, ActionId>,

    /// Context-specific bindings
    #[serde(default)]
    pub contexts: HashMap<Context, HashMap<KeySequence, ActionId>>,
}

impl Keymap {
    /// Create a new empty keymap.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a global binding.
    pub fn bind(&mut self, keys: impl Into<KeySequence>, action: impl Into<ActionId>) {
        self.global.insert(keys.into(), action.into());
    }

    /// Add a context-specific binding.
    pub fn bind_in_context(
        &mut self,
        context: Context,
        keys: impl Into<KeySequence>,
        action: impl Into<ActionId>,
    ) {
        self.contexts
            .entry(context)
            .or_insert_with(HashMap::new)
            .insert(keys.into(), action.into());
    }

    /// Get the action for a key sequence in a given context.
    pub fn get_action(&self, keys: &KeySequence, context: &Context) -> Option<&ActionId> {
        // Context-specific bindings take precedence
        if let Some(context_map) = self.contexts.get(context) {
            if let Some(action) = context_map.get(keys) {
                return Some(action);
            }
        }

        // Fall back to global bindings
        self.global.get(keys)
    }

    /// Get all bindings for a specific context (including global).
    pub fn bindings_for_context(&self, context: &Context) -> impl Iterator<Item = (&KeySequence, &ActionId)> {
        let context_bindings = self
            .contexts
            .get(context)
            .map(|m| m.iter())
            .into_iter()
            .flatten();

        self.global.iter().chain(context_bindings)
    }

    /// Find the key sequence(s) bound to an action.
    pub fn get_bindings_for_action(
        &self,
        action_id: &str,
        context: &Context,
    ) -> Vec<&KeySequence> {
        let mut bindings = Vec::new();

        // Check context-specific
        if let Some(context_map) = self.contexts.get(context) {
            for (keys, action) in context_map {
                if action.0 == action_id {
                    bindings.push(keys);
                }
            }
        }

        // Check global
        for (keys, action) in &self.global {
            if action.0 == action_id {
                bindings.push(keys);
            }
        }

        bindings
    }

    /// Merge another keymap into this one (other takes precedence).
    pub fn merge(&mut self, other: Keymap) {
        for (keys, action) in other.global {
            self.global.insert(keys, action);
        }

        for (context, bindings) in other.contexts {
            let entry = self.contexts.entry(context).or_insert_with(HashMap::new);
            for (keys, action) in bindings {
                entry.insert(keys, action);
            }
        }
    }

    /// Remove bindings for specific keys.
    pub fn unbind(&mut self, keys: &KeySequence) {
        self.global.remove(keys);
        for context_map in self.contexts.values_mut() {
            context_map.remove(keys);
        }
    }
}

/// A group of related actions with preset bindings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionGroup {
    /// Group name
    pub name: String,
    /// Action IDs in this group
    pub actions: Vec<String>,
    /// Named presets for this group
    pub presets: HashMap<String, HashMap<String, KeySequence>>,
}

impl ActionGroup {
    /// Create a new action group.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            actions: Vec::new(),
            presets: HashMap::new(),
        }
    }

    /// Add an action to this group.
    pub fn action(mut self, id: impl Into<String>) -> Self {
        self.actions.push(id.into());
        self
    }

    /// Add a preset to this group.
    pub fn preset(
        mut self,
        name: impl Into<String>,
        bindings: HashMap<String, KeySequence>,
    ) -> Self {
        self.presets.insert(name.into(), bindings);
        self
    }

    /// Get bindings for a preset.
    pub fn get_preset(&self, name: &str) -> Option<&HashMap<String, KeySequence>> {
        self.presets.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::KeyBinding;
    use crossterm::event::KeyCode;

    #[test]
    fn test_keymap_global() {
        let mut keymap = Keymap::new();
        keymap.bind(
            KeySequence::single(KeyBinding::ctrl('s')),
            "save",
        );

        let action = keymap.get_action(
            &KeySequence::single(KeyBinding::ctrl('s')),
            &Context::Normal,
        );
        assert_eq!(action.map(|a| a.0.as_str()), Some("save"));
    }

    #[test]
    fn test_keymap_context() {
        let mut keymap = Keymap::new();

        // Global binding
        keymap.bind(
            KeySequence::single(KeyBinding::key(KeyCode::Char('j'))),
            "move_down",
        );

        // Context-specific override
        keymap.bind_in_context(
            Context::Insert,
            KeySequence::single(KeyBinding::key(KeyCode::Char('j'))),
            "insert_j",
        );

        // In normal mode, get global
        let action = keymap.get_action(
            &KeySequence::single(KeyBinding::key(KeyCode::Char('j'))),
            &Context::Normal,
        );
        assert_eq!(action.map(|a| a.0.as_str()), Some("move_down"));

        // In insert mode, get override
        let action = keymap.get_action(
            &KeySequence::single(KeyBinding::key(KeyCode::Char('j'))),
            &Context::Insert,
        );
        assert_eq!(action.map(|a| a.0.as_str()), Some("insert_j"));
    }

    #[test]
    fn test_keymap_merge() {
        let mut base = Keymap::new();
        base.bind(
            KeySequence::single(KeyBinding::ctrl('a')),
            "action_a",
        );

        let mut override_map = Keymap::new();
        override_map.bind(
            KeySequence::single(KeyBinding::ctrl('a')),
            "action_a_override",
        );
        override_map.bind(
            KeySequence::single(KeyBinding::ctrl('b')),
            "action_b",
        );

        base.merge(override_map);

        let action = base.get_action(
            &KeySequence::single(KeyBinding::ctrl('a')),
            &Context::Normal,
        );
        assert_eq!(action.map(|a| a.0.as_str()), Some("action_a_override"));

        let action = base.get_action(
            &KeySequence::single(KeyBinding::ctrl('b')),
            &Context::Normal,
        );
        assert_eq!(action.map(|a| a.0.as_str()), Some("action_b"));
    }
}
