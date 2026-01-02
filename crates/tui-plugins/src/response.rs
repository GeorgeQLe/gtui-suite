//! Plugin responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from a plugin to an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    /// The action to take.
    pub action: ResponseAction,
    /// Whether the event was fully handled (prevent further processing).
    #[serde(default)]
    pub handled: bool,
    /// Optional payload data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// Actions a plugin can request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseAction {
    /// No action needed.
    None,

    /// Show a notification.
    Notify {
        /// Message to display.
        message: String,
        /// Notification level.
        #[serde(default)]
        level: NotifyLevel,
        /// Duration in milliseconds (0 = until dismissed).
        #[serde(default)]
        duration_ms: u64,
    },

    /// Show a prompt dialog.
    Prompt {
        /// Prompt title.
        title: String,
        /// Prompt message.
        #[serde(default)]
        message: Option<String>,
        /// Input type.
        #[serde(default)]
        input_type: PromptInputType,
        /// Default value.
        #[serde(default)]
        default_value: Option<String>,
        /// Callback command to run with result.
        callback: String,
    },

    /// Run an application command.
    RunCommand {
        /// Command name.
        name: String,
        /// Command arguments.
        #[serde(default)]
        args: HashMap<String, serde_json::Value>,
    },

    /// Set clipboard content.
    SetClipboard {
        /// Text to copy.
        text: String,
    },

    /// Open a file.
    OpenFile {
        /// File path.
        path: String,
        /// Position to jump to.
        #[serde(default)]
        position: Option<(usize, usize)>,
    },

    /// Insert text at cursor.
    InsertText {
        /// Text to insert.
        text: String,
    },

    /// Replace selection.
    ReplaceSelection {
        /// Replacement text.
        text: String,
    },

    /// Log a message.
    Log {
        /// Log level.
        level: LogLevel,
        /// Message.
        message: String,
    },

    /// Set a timer.
    SetTimer {
        /// Timer ID.
        id: String,
        /// Interval in milliseconds.
        interval_ms: u64,
        /// Whether to repeat.
        #[serde(default)]
        repeat: bool,
    },

    /// Cancel a timer.
    CancelTimer {
        /// Timer ID.
        id: String,
    },

    /// Request data from host.
    RequestData {
        /// Data type requested.
        data_type: String,
        /// Callback command.
        callback: String,
    },

    /// Return data to host.
    ReturnData {
        /// Returned data.
        data: serde_json::Value,
    },

    /// Custom action.
    Custom {
        /// Action name.
        name: String,
        /// Action data.
        data: serde_json::Value,
    },
}

/// Notification level.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyLevel {
    /// Informational.
    #[default]
    Info,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Error.
    Error,
}

/// Prompt input type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptInputType {
    /// Single line text.
    #[default]
    Text,
    /// Password (hidden).
    Password,
    /// Multi-line text.
    TextArea,
    /// Yes/No confirmation.
    Confirm,
    /// Select from options.
    Select {
        /// Available options.
        options: Vec<String>,
    },
}

/// Log level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl PluginResponse {
    /// Create a response with no action.
    pub fn none() -> Self {
        Self {
            action: ResponseAction::None,
            handled: false,
            payload: None,
        }
    }

    /// Create a notification response.
    pub fn notify(message: impl Into<String>) -> Self {
        Self {
            action: ResponseAction::Notify {
                message: message.into(),
                level: NotifyLevel::Info,
                duration_ms: 3000,
            },
            handled: false,
            payload: None,
        }
    }

    /// Create a notification with level.
    pub fn notify_with_level(message: impl Into<String>, level: NotifyLevel) -> Self {
        Self {
            action: ResponseAction::Notify {
                message: message.into(),
                level,
                duration_ms: 3000,
            },
            handled: false,
            payload: None,
        }
    }

    /// Create an error notification.
    pub fn error(message: impl Into<String>) -> Self {
        Self::notify_with_level(message, NotifyLevel::Error)
    }

    /// Create a success notification.
    pub fn success(message: impl Into<String>) -> Self {
        Self::notify_with_level(message, NotifyLevel::Success)
    }

    /// Create a log response.
    pub fn log(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            action: ResponseAction::Log {
                level,
                message: message.into(),
            },
            handled: false,
            payload: None,
        }
    }

    /// Create a command response.
    pub fn run_command(name: impl Into<String>) -> Self {
        Self {
            action: ResponseAction::RunCommand {
                name: name.into(),
                args: HashMap::new(),
            },
            handled: true,
            payload: None,
        }
    }

    /// Create a command response with args.
    pub fn run_command_with_args(
        name: impl Into<String>,
        args: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            action: ResponseAction::RunCommand {
                name: name.into(),
                args,
            },
            handled: true,
            payload: None,
        }
    }

    /// Mark the event as handled.
    pub fn handled(mut self) -> Self {
        self.handled = true;
        self
    }

    /// Add payload data.
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Convert to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Parse from JSON.
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_response() {
        let response = PluginResponse::notify("Hello!");
        assert!(!response.handled);

        if let ResponseAction::Notify { message, level, .. } = response.action {
            assert_eq!(message, "Hello!");
            assert!(matches!(level, NotifyLevel::Info));
        } else {
            panic!("Expected notify action");
        }
    }

    #[test]
    fn test_handled() {
        let response = PluginResponse::notify("Test").handled();
        assert!(response.handled);
    }

    #[test]
    fn test_run_command() {
        let response = PluginResponse::run_command("save");
        assert!(response.handled);

        if let ResponseAction::RunCommand { name, .. } = response.action {
            assert_eq!(name, "save");
        } else {
            panic!("Expected run_command action");
        }
    }

    #[test]
    fn test_serialization() {
        let response = PluginResponse::notify("Test");
        let json = response.to_json();
        let restored = PluginResponse::from_json(json).unwrap();

        if let ResponseAction::Notify { message, .. } = restored.action {
            assert_eq!(message, "Test");
        } else {
            panic!("Expected notify action");
        }
    }
}
