//! Macro recording and playback.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A recorded macro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    /// Macro name
    pub name: String,
    /// Recorded key events
    pub keys: Vec<MacroKey>,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// When the macro was created
    pub created_at: DateTime<Utc>,
}

/// A key in a macro (can be a key event or literal text).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MacroKey {
    /// A key event
    Key { key: String },
    /// Literal text to insert
    Text { text: String },
    /// A single character
    Char { char: char },
}

impl Macro {
    /// Create a new macro.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            keys: Vec::new(),
            description: None,
            created_at: Utc::now(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a key to the macro.
    pub fn push_key(&mut self, key: String) {
        self.keys.push(MacroKey::Key { key });
    }

    /// Add text to the macro.
    pub fn push_text(&mut self, text: String) {
        self.keys.push(MacroKey::Text { text });
    }

    /// Get the number of steps in this macro.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Check if the macro is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

/// Recording state for a macro being recorded.
#[derive(Debug)]
struct MacroRecording {
    name: String,
    keys: Vec<MacroKey>,
    started_at: DateTime<Utc>,
}

/// Macro manager for recording and playback.
#[derive(Debug, Default)]
pub struct MacroManager {
    /// Stored macros
    macros: HashMap<String, Macro>,
    /// Current recording
    recording: Option<MacroRecording>,
}

impl MacroManager {
    /// Create a new macro manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start recording a new macro.
    pub fn start_recording(&mut self, name: impl Into<String>) {
        self.recording = Some(MacroRecording {
            name: name.into(),
            keys: Vec::new(),
            started_at: Utc::now(),
        });
    }

    /// Stop recording and save the macro.
    pub fn stop_recording(&mut self) -> Option<Macro> {
        let recording = self.recording.take()?;

        let macro_def = Macro {
            name: recording.name.clone(),
            keys: recording.keys,
            description: None,
            created_at: recording.started_at,
        };

        self.macros.insert(recording.name, macro_def.clone());
        Some(macro_def)
    }

    /// Cancel recording without saving.
    pub fn cancel_recording(&mut self) {
        self.recording = None;
    }

    /// Check if currently recording.
    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    /// Get the name of the macro being recorded.
    pub fn recording_name(&self) -> Option<&str> {
        self.recording.as_ref().map(|r| r.name.as_str())
    }

    /// Record a key event.
    pub fn record_key(&mut self, key: &str) {
        if let Some(ref mut recording) = self.recording {
            recording.keys.push(MacroKey::Key {
                key: key.to_string(),
            });
        }
    }

    /// Record text input.
    pub fn record_text(&mut self, text: &str) {
        if let Some(ref mut recording) = self.recording {
            recording.keys.push(MacroKey::Text {
                text: text.to_string(),
            });
        }
    }

    /// Get a macro by name.
    pub fn get(&self, name: &str) -> Option<&Macro> {
        self.macros.get(name)
    }

    /// List all macros.
    pub fn list(&self) -> impl Iterator<Item = &Macro> {
        self.macros.values()
    }

    /// Delete a macro.
    pub fn delete(&mut self, name: &str) -> bool {
        self.macros.remove(name).is_some()
    }

    /// Save macros to a TOML file.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        #[derive(Serialize)]
        struct MacroFile {
            macros: Vec<Macro>,
        }

        let file = MacroFile {
            macros: self.macros.values().cloned().collect(),
        };

        let content = toml::to_string_pretty(&file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        std::fs::write(path, content)
    }

    /// Load macros from a TOML file.
    pub fn load(&mut self, path: &Path) -> Result<(), std::io::Error> {
        #[derive(Deserialize)]
        struct MacroFile {
            #[serde(default)]
            macros: Vec<Macro>,
        }

        let content = std::fs::read_to_string(path)?;
        let file: MacroFile = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        for macro_def in file.macros {
            self.macros.insert(macro_def.name.clone(), macro_def);
        }

        Ok(())
    }

    /// Get keys to replay for a macro.
    pub fn play(&self, name: &str) -> Option<&[MacroKey]> {
        self.macros.get(name).map(|m| m.keys.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_creation() {
        let macro_def = Macro::new("test")
            .with_description("Test macro");

        assert_eq!(macro_def.name, "test");
        assert!(macro_def.description.is_some());
        assert!(macro_def.is_empty());
    }

    #[test]
    fn test_recording() {
        let mut manager = MacroManager::new();

        manager.start_recording("test");
        assert!(manager.is_recording());
        assert_eq!(manager.recording_name(), Some("test"));

        manager.record_key("j");
        manager.record_key("k");
        manager.record_text("hello");

        let macro_def = manager.stop_recording().unwrap();
        assert_eq!(macro_def.name, "test");
        assert_eq!(macro_def.len(), 3);

        assert!(!manager.is_recording());
    }

    #[test]
    fn test_cancel_recording() {
        let mut manager = MacroManager::new();

        manager.start_recording("test");
        manager.record_key("j");
        manager.cancel_recording();

        assert!(!manager.is_recording());
        assert!(manager.get("test").is_none());
    }

    #[test]
    fn test_play() {
        let mut manager = MacroManager::new();

        manager.start_recording("test");
        manager.record_key("j");
        manager.record_key("k");
        manager.stop_recording();

        let keys = manager.play("test").unwrap();
        assert_eq!(keys.len(), 2);

        assert!(manager.play("nonexistent").is_none());
    }
}
