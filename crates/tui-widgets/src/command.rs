//! Command trait for CommandPalette integration.
//!
//! Apps implement the `Command` trait for their actions, and the shell
//! wraps app commands for a unified launcher experience.

use std::collections::HashMap;
use thiserror::Error;

/// Error type for command execution.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Invalid parameter value
    #[error("invalid parameter '{name}': {reason}")]
    InvalidParameter { name: String, reason: String },

    /// Missing required parameter
    #[error("missing required parameter: {0}")]
    MissingParameter(String),

    /// Command execution failed
    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    /// Command was cancelled
    #[error("command cancelled")]
    Cancelled,

    /// Command not found
    #[error("command not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Shared trait for CommandPalette integration.
///
/// Apps implement this trait for their actions, enabling them to be
/// discovered and executed through the command palette.
///
/// # Example
///
/// ```ignore
/// struct GoToLineCommand;
///
/// impl Command for GoToLineCommand {
///     fn id(&self) -> &str {
///         "editor.goto_line"
///     }
///
///     fn label(&self) -> &str {
///         "Go to Line"
///     }
///
///     fn description(&self) -> Option<&str> {
///         Some("Jump to a specific line number")
///     }
///
///     fn keywords(&self) -> &[&str] {
///         &["jump", "navigate", "line"]
///     }
///
///     fn execute(&self, params: HashMap<String, String>) -> Result<(), CommandError> {
///         let line = params.get("line")
///             .ok_or_else(|| CommandError::MissingParameter("line".into()))?
///             .parse::<usize>()
///             .map_err(|_| CommandError::InvalidParameter {
///                 name: "line".into(),
///                 reason: "must be a number".into(),
///             })?;
///         // Jump to line...
///         Ok(())
///     }
/// }
/// ```
pub trait Command: Send + Sync {
    /// Unique identifier for this command.
    ///
    /// Should be namespaced (e.g., "editor.goto_line", "file.save").
    fn id(&self) -> &str;

    /// Human-readable label shown in the command palette.
    fn label(&self) -> &str;

    /// Optional description for additional context.
    fn description(&self) -> Option<&str> {
        None
    }

    /// Keywords for fuzzy search matching.
    ///
    /// These are searched in addition to the label.
    fn keywords(&self) -> &[&str] {
        &[]
    }

    /// Execute the command with the given parameters.
    ///
    /// Parameters are collected via the command palette's multi-step wizard
    /// if the command has parameter definitions.
    fn execute(&self, params: HashMap<String, String>) -> Result<(), CommandError>;

    /// Optional keyboard shortcut hint.
    ///
    /// This is for display only; actual keybinding is managed by tui-keybinds.
    fn shortcut_hint(&self) -> Option<&str> {
        None
    }

    /// Category for grouping in the command palette.
    fn category(&self) -> Option<&str> {
        None
    }

    /// Whether this command is currently enabled.
    ///
    /// Disabled commands are shown grayed out in the palette.
    fn is_enabled(&self) -> bool {
        true
    }

    /// Whether this command should be hidden from the palette.
    ///
    /// Hidden commands can still be executed programmatically.
    fn is_hidden(&self) -> bool {
        false
    }
}

/// A boxed command for type erasure.
pub type BoxedCommand = Box<dyn Command>;

/// Command registry for managing available commands.
#[derive(Default)]
pub struct CommandRegistry {
    commands: Vec<BoxedCommand>,
}

impl CommandRegistry {
    /// Create a new empty command registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command.
    pub fn register(&mut self, command: impl Command + 'static) {
        self.commands.push(Box::new(command));
    }

    /// Get all registered commands.
    pub fn commands(&self) -> &[BoxedCommand] {
        &self.commands
    }

    /// Find a command by ID.
    pub fn find(&self, id: &str) -> Option<&dyn Command> {
        self.commands.iter().find(|c| c.id() == id).map(|c| c.as_ref())
    }

    /// Execute a command by ID.
    pub fn execute(
        &self,
        id: &str,
        params: HashMap<String, String>,
    ) -> Result<(), CommandError> {
        let command = self.find(id).ok_or_else(|| CommandError::NotFound(id.into()))?;
        command.execute(params)
    }

    /// Get visible commands (not hidden).
    pub fn visible_commands(&self) -> impl Iterator<Item = &dyn Command> {
        self.commands.iter().filter(|c| !c.is_hidden()).map(|c| c.as_ref())
    }

    /// Get enabled commands only.
    pub fn enabled_commands(&self) -> impl Iterator<Item = &dyn Command> {
        self.commands.iter().filter(|c| c.is_enabled()).map(|c| c.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        id: String,
        label: String,
    }

    impl Command for TestCommand {
        fn id(&self) -> &str {
            &self.id
        }

        fn label(&self) -> &str {
            &self.label
        }

        fn execute(&self, _params: HashMap<String, String>) -> Result<(), CommandError> {
            Ok(())
        }
    }

    #[test]
    fn test_command_registry() {
        let mut registry = CommandRegistry::new();

        registry.register(TestCommand {
            id: "test.command1".into(),
            label: "Test Command 1".into(),
        });

        registry.register(TestCommand {
            id: "test.command2".into(),
            label: "Test Command 2".into(),
        });

        assert_eq!(registry.commands().len(), 2);
        assert!(registry.find("test.command1").is_some());
        assert!(registry.find("test.nonexistent").is_none());
    }

    #[test]
    fn test_execute_command() {
        let mut registry = CommandRegistry::new();

        registry.register(TestCommand {
            id: "test.cmd".into(),
            label: "Test".into(),
        });

        let result = registry.execute("test.cmd", HashMap::new());
        assert!(result.is_ok());

        let result = registry.execute("nonexistent", HashMap::new());
        assert!(matches!(result, Err(CommandError::NotFound(_))));
    }
}
