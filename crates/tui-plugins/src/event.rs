//! Plugin events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Events sent from host to plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginEvent {
    /// Application lifecycle event.
    Lifecycle(LifecycleEvent),

    /// Key press event.
    Key(KeyEventData),

    /// Command invoked.
    Command(CommandEvent),

    /// Selection changed.
    SelectionChanged(SelectionEvent),

    /// File opened.
    FileOpened(FileEvent),

    /// File saved.
    FileSaved(FileEvent),

    /// Theme changed.
    ThemeChanged(ThemeEvent),

    /// Timer tick.
    Timer(TimerEvent),

    /// Custom event.
    Custom {
        /// Event name.
        name: String,
        /// Event payload as JSON.
        payload: serde_json::Value,
    },
}

/// Application lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEvent {
    /// Application starting.
    Starting,
    /// Application ready.
    Ready,
    /// Application shutting down.
    ShuttingDown,
    /// Plugin being enabled.
    Enabled,
    /// Plugin being disabled.
    Disabled,
}

/// Key event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEventData {
    /// Key code.
    pub code: String,
    /// Modifier keys.
    pub modifiers: Vec<String>,
    /// Raw key event (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl KeyEventData {
    /// Create a new key event.
    pub fn new(code: impl Into<String>, modifiers: Vec<String>) -> Self {
        Self {
            code: code.into(),
            modifiers,
            raw: None,
        }
    }

    /// Check if Ctrl is pressed.
    pub fn has_ctrl(&self) -> bool {
        self.modifiers.iter().any(|m| m.to_lowercase() == "ctrl")
    }

    /// Check if Alt is pressed.
    pub fn has_alt(&self) -> bool {
        self.modifiers.iter().any(|m| m.to_lowercase() == "alt")
    }

    /// Check if Shift is pressed.
    pub fn has_shift(&self) -> bool {
        self.modifiers.iter().any(|m| m.to_lowercase() == "shift")
    }
}

/// Command event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    /// Command name/ID.
    pub name: String,
    /// Command arguments.
    #[serde(default)]
    pub args: HashMap<String, serde_json::Value>,
}

impl CommandEvent {
    /// Create a new command event.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: HashMap::new(),
        }
    }

    /// Create with arguments.
    pub fn with_args(name: impl Into<String>, args: HashMap<String, serde_json::Value>) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }

    /// Get a string argument.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.args.get(key).and_then(|v| v.as_str())
    }

    /// Get an integer argument.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.args.get(key).and_then(|v| v.as_i64())
    }

    /// Get a boolean argument.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.args.get(key).and_then(|v| v.as_bool())
    }
}

/// Selection event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionEvent {
    /// Selected text or item ID.
    pub selection: String,
    /// Selection type.
    pub selection_type: SelectionType,
    /// Start position (for text selections).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<Position>,
    /// End position (for text selections).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<Position>,
}

/// Type of selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionType {
    /// Text selection.
    Text,
    /// Line selection.
    Line,
    /// Block/rectangle selection.
    Block,
    /// Item selection (list/table).
    Item,
    /// Multiple items.
    Multi,
}

/// Position in a document.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed).
    pub line: usize,
    /// Column number (0-indexed).
    pub column: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// File event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    /// File path.
    pub path: String,
    /// File type/extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    /// Language ID (for syntax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl FileEvent {
    /// Create a new file event.
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let file_type = std::path::Path::new(&path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        Self {
            path,
            file_type,
            language: None,
        }
    }

    /// Set the language.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
}

/// Theme change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeEvent {
    /// New theme name.
    pub theme: String,
    /// Whether it's a dark theme.
    pub is_dark: bool,
}

/// Timer event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerEvent {
    /// Timer ID.
    pub id: String,
    /// Elapsed milliseconds.
    pub elapsed_ms: u64,
}

impl PluginEvent {
    /// Get the event type name.
    pub fn event_type(&self) -> &str {
        match self {
            Self::Lifecycle(_) => "lifecycle",
            Self::Key(_) => "key",
            Self::Command(_) => "command",
            Self::SelectionChanged(_) => "selection_changed",
            Self::FileOpened(_) => "file_opened",
            Self::FileSaved(_) => "file_saved",
            Self::ThemeChanged(_) => "theme_changed",
            Self::Timer(_) => "timer",
            Self::Custom { name, .. } => name,
        }
    }

    /// Convert to JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Create from JSON.
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event() {
        let event = KeyEventData::new("s", vec!["ctrl".to_string()]);
        assert!(event.has_ctrl());
        assert!(!event.has_alt());
    }

    #[test]
    fn test_command_event() {
        let mut args = HashMap::new();
        args.insert("count".to_string(), serde_json::json!(5));
        args.insert("name".to_string(), serde_json::json!("test"));

        let event = CommandEvent::with_args("my_command", args);
        assert_eq!(event.get_i64("count"), Some(5));
        assert_eq!(event.get_str("name"), Some("test"));
    }

    #[test]
    fn test_file_event() {
        let event = FileEvent::new("/path/to/file.rs");
        assert_eq!(event.file_type, Some("rs".to_string()));
    }

    #[test]
    fn test_event_serialization() {
        let event = PluginEvent::Command(CommandEvent::new("test"));
        let json = event.to_json();
        assert!(json.is_object());

        let restored = PluginEvent::from_json(json).unwrap();
        assert_eq!(restored.event_type(), "command");
    }
}
