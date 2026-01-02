//! Conflict detection for keybindings.

use crate::binding::KeySequence;
use crate::context::Context;
use crate::keymap::ActionId;
use serde::{Deserialize, Serialize};

/// Severity of a keybinding conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictSeverity {
    /// Same key in different contexts (may be intentional)
    Warning,
    /// Same key in same context (config rejected)
    Error,
}

/// A keybinding conflict.
#[derive(Debug, Clone)]
pub struct Conflict {
    /// The conflicting key sequence
    pub key: KeySequence,
    /// Actions and their sources that conflict
    pub actions: Vec<(String, ActionId)>, // (source name, action)
    /// The context where the conflict occurs
    pub context: Option<Context>,
    /// Conflict severity
    pub severity: ConflictSeverity,
}

impl Conflict {
    /// Create a new conflict.
    pub fn new(
        key: KeySequence,
        context: Option<Context>,
        severity: ConflictSeverity,
    ) -> Self {
        Self {
            key,
            actions: Vec::new(),
            context,
            severity,
        }
    }

    /// Add an action to the conflict.
    pub fn add_action(&mut self, source: impl Into<String>, action: ActionId) {
        self.actions.push((source.into(), action));
    }
}

/// Report of all conflicts found in a configuration.
#[derive(Debug, Default)]
pub struct ConflictReport {
    /// All detected conflicts
    pub conflicts: Vec<Conflict>,
}

impl ConflictReport {
    /// Create an empty conflict report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a conflict to the report.
    pub fn add(&mut self, conflict: Conflict) {
        self.conflicts.push(conflict);
    }

    /// Check if there are any conflicts.
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Check if there are any error-level conflicts.
    pub fn has_errors(&self) -> bool {
        self.conflicts.iter().any(|c| c.severity == ConflictSeverity::Error)
    }

    /// Get only error-level conflicts.
    pub fn errors(&self) -> impl Iterator<Item = &Conflict> {
        self.conflicts.iter().filter(|c| c.severity == ConflictSeverity::Error)
    }

    /// Get only warning-level conflicts.
    pub fn warnings(&self) -> impl Iterator<Item = &Conflict> {
        self.conflicts.iter().filter(|c| c.severity == ConflictSeverity::Warning)
    }
}

impl std::fmt::Display for ConflictReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.conflicts.is_empty() {
            return write!(f, "No conflicts detected");
        }

        for conflict in &self.conflicts {
            let severity = match conflict.severity {
                ConflictSeverity::Error => "ERROR",
                ConflictSeverity::Warning => "WARNING",
            };

            let context = conflict
                .context
                .as_ref()
                .map(|c| format!(" in context '{}'", c))
                .unwrap_or_default();

            writeln!(
                f,
                "[{}] Key '{}'{} has multiple bindings:",
                severity, conflict.key, context
            )?;

            for (source, action) in &conflict.actions {
                writeln!(f, "  - '{}' from {}", action.0, source)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::KeyBinding;
    use crossterm::event::KeyCode;

    #[test]
    fn test_conflict_creation() {
        let key = KeySequence::single(KeyBinding::ctrl('s'));
        let mut conflict = Conflict::new(key, None, ConflictSeverity::Error);

        conflict.add_action("default", ActionId::new("save"));
        conflict.add_action("user", ActionId::new("sync"));

        assert_eq!(conflict.actions.len(), 2);
        assert_eq!(conflict.severity, ConflictSeverity::Error);
    }

    #[test]
    fn test_conflict_report() {
        let mut report = ConflictReport::new();
        assert!(!report.has_conflicts());

        let key = KeySequence::single(KeyBinding::ctrl('s'));
        let mut conflict = Conflict::new(key, None, ConflictSeverity::Error);
        conflict.add_action("a", ActionId::new("action1"));
        conflict.add_action("b", ActionId::new("action2"));
        report.add(conflict);

        assert!(report.has_conflicts());
        assert!(report.has_errors());
        assert_eq!(report.errors().count(), 1);
    }

    #[test]
    fn test_report_display() {
        let mut report = ConflictReport::new();

        let key = KeySequence::single(KeyBinding::ctrl('s'));
        let mut conflict = Conflict::new(key, Some(Context::Normal), ConflictSeverity::Error);
        conflict.add_action("default", ActionId::new("save"));
        conflict.add_action("user", ActionId::new("sync"));
        report.add(conflict);

        let display = report.to_string();
        assert!(display.contains("ERROR"));
        assert!(display.contains("Ctrl+S"));
        assert!(display.contains("save"));
        assert!(display.contains("sync"));
    }
}
