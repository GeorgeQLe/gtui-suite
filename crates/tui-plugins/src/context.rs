//! Plugin context for host-plugin communication.

use crate::error::PluginResult;
use crate::response::LogLevel;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Context provided to plugins by the host application.
#[derive(Clone)]
pub struct PluginContext {
    /// Application name.
    app_name: String,
    /// Application version.
    app_version: String,
    /// Plugin's data directory.
    data_dir: PathBuf,
    /// Plugin's config directory.
    config_dir: PathBuf,
    /// Host callback functions.
    callbacks: Arc<HostCallbacks>,
    /// Shared state.
    state: Arc<std::sync::RwLock<HashMap<String, serde_json::Value>>>,
}

/// Callbacks from plugin to host.
pub struct HostCallbacks {
    /// Log a message.
    pub log: Box<dyn Fn(LogLevel, &str) + Send + Sync>,
    /// Show notification.
    pub notify: Box<dyn Fn(&str) + Send + Sync>,
    /// Get current selection.
    pub get_selection: Box<dyn Fn() -> Option<String> + Send + Sync>,
    /// Set clipboard.
    pub set_clipboard: Box<dyn Fn(&str) -> PluginResult<()> + Send + Sync>,
    /// Run command.
    pub run_command: Box<dyn Fn(&str, &HashMap<String, serde_json::Value>) -> PluginResult<()> + Send + Sync>,
}

impl Default for HostCallbacks {
    fn default() -> Self {
        Self {
            log: Box::new(|level, msg| {
                eprintln!("[{:?}] {}", level, msg);
            }),
            notify: Box::new(|msg| {
                println!("[NOTIFY] {}", msg);
            }),
            get_selection: Box::new(|| None),
            set_clipboard: Box::new(|_| Ok(())),
            run_command: Box::new(|_, _| Ok(())),
        }
    }
}

impl PluginContext {
    /// Create a new plugin context.
    pub fn new(
        app_name: impl Into<String>,
        app_version: impl Into<String>,
        data_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            app_version: app_version.into(),
            data_dir,
            config_dir,
            callbacks: Arc::new(HostCallbacks::default()),
            state: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom callbacks.
    pub fn with_callbacks(mut self, callbacks: HostCallbacks) -> Self {
        self.callbacks = Arc::new(callbacks);
        self
    }

    /// Get application name.
    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    /// Get application version.
    pub fn app_version(&self) -> &str {
        &self.app_version
    }

    /// Get plugin's data directory.
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Get plugin's config directory.
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Log a debug message.
    pub fn log_debug(&self, msg: &str) {
        (self.callbacks.log)(LogLevel::Debug, msg);
    }

    /// Log an info message.
    pub fn log_info(&self, msg: &str) {
        (self.callbacks.log)(LogLevel::Info, msg);
    }

    /// Log a warning message.
    pub fn log_warn(&self, msg: &str) {
        (self.callbacks.log)(LogLevel::Warn, msg);
    }

    /// Log an error message.
    pub fn log_error(&self, msg: &str) {
        (self.callbacks.log)(LogLevel::Error, msg);
    }

    /// Log with specific level.
    pub fn log(&self, level: LogLevel, msg: &str) {
        (self.callbacks.log)(level, msg);
    }

    /// Show a notification.
    pub fn notify(&self, msg: &str) {
        (self.callbacks.notify)(msg);
    }

    /// Get current selection.
    pub fn get_selection(&self) -> Option<String> {
        (self.callbacks.get_selection)()
    }

    /// Set clipboard content.
    pub fn set_clipboard(&self, text: &str) -> PluginResult<()> {
        (self.callbacks.set_clipboard)(text)
    }

    /// Run an application command.
    pub fn run_command(
        &self,
        name: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> PluginResult<()> {
        (self.callbacks.run_command)(name, args)
    }

    /// Get a value from shared state.
    pub fn get_state(&self, key: &str) -> Option<serde_json::Value> {
        self.state.read().ok()?.get(key).cloned()
    }

    /// Set a value in shared state.
    pub fn set_state(&self, key: &str, value: serde_json::Value) {
        if let Ok(mut state) = self.state.write() {
            state.insert(key.to_string(), value);
        }
    }

    /// Remove a value from shared state.
    pub fn remove_state(&self, key: &str) -> Option<serde_json::Value> {
        self.state.write().ok()?.remove(key)
    }
}

/// Builder for creating plugin contexts.
pub struct PluginContextBuilder {
    app_name: String,
    app_version: String,
    data_dir: Option<PathBuf>,
    config_dir: Option<PathBuf>,
    callbacks: HostCallbacks,
}

impl PluginContextBuilder {
    /// Create a new builder.
    pub fn new(app_name: impl Into<String>, app_version: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            app_version: app_version.into(),
            data_dir: None,
            config_dir: None,
            callbacks: HostCallbacks::default(),
        }
    }

    /// Set the data directory.
    pub fn data_dir(mut self, dir: PathBuf) -> Self {
        self.data_dir = Some(dir);
        self
    }

    /// Set the config directory.
    pub fn config_dir(mut self, dir: PathBuf) -> Self {
        self.config_dir = Some(dir);
        self
    }

    /// Set the log callback.
    pub fn on_log<F>(mut self, f: F) -> Self
    where
        F: Fn(LogLevel, &str) + Send + Sync + 'static,
    {
        self.callbacks.log = Box::new(f);
        self
    }

    /// Set the notify callback.
    pub fn on_notify<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.callbacks.notify = Box::new(f);
        self
    }

    /// Set the get_selection callback.
    pub fn on_get_selection<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Option<String> + Send + Sync + 'static,
    {
        self.callbacks.get_selection = Box::new(f);
        self
    }

    /// Set the set_clipboard callback.
    pub fn on_set_clipboard<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> PluginResult<()> + Send + Sync + 'static,
    {
        self.callbacks.set_clipboard = Box::new(f);
        self
    }

    /// Set the run_command callback.
    pub fn on_run_command<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &HashMap<String, serde_json::Value>) -> PluginResult<()> + Send + Sync + 'static,
    {
        self.callbacks.run_command = Box::new(f);
        self
    }

    /// Build the context.
    pub fn build(self) -> PluginContext {
        let base_dir = directories::ProjectDirs::from("", "", &self.app_name)
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        PluginContext {
            app_name: self.app_name,
            app_version: self.app_version,
            data_dir: self.data_dir.unwrap_or_else(|| base_dir.join("plugins")),
            config_dir: self.config_dir.unwrap_or_else(|| base_dir.join("config")),
            callbacks: Arc::new(self.callbacks),
            state: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = PluginContext::new(
            "test-app",
            "1.0.0",
            PathBuf::from("/tmp/data"),
            PathBuf::from("/tmp/config"),
        );

        assert_eq!(ctx.app_name(), "test-app");
        assert_eq!(ctx.app_version(), "1.0.0");
    }

    #[test]
    fn test_state() {
        let ctx = PluginContext::new(
            "test",
            "1.0",
            PathBuf::from("/tmp"),
            PathBuf::from("/tmp"),
        );

        ctx.set_state("key", serde_json::json!("value"));
        assert_eq!(
            ctx.get_state("key"),
            Some(serde_json::json!("value"))
        );

        ctx.remove_state("key");
        assert!(ctx.get_state("key").is_none());
    }

    #[test]
    fn test_builder() {
        let logged = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let logged_clone = logged.clone();

        let ctx = PluginContextBuilder::new("myapp", "2.0")
            .on_log(move |level, msg| {
                logged_clone.lock().unwrap().push((level, msg.to_string()));
            })
            .build();

        ctx.log_info("test message");

        let logs = logged.lock().unwrap();
        assert_eq!(logs.len(), 1);
    }
}
