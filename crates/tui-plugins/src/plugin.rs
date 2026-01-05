//! Plugin trait definition.

use crate::capability::Capability;
use crate::context::PluginContext;
use crate::error::PluginResult;
use crate::event::PluginEvent;
use crate::response::PluginResponse;
use crate::Backend;

/// Core plugin trait that all plugins must implement.
pub trait Plugin: Send {
    /// Get the unique plugin identifier.
    fn id(&self) -> &str;

    /// Get the human-readable name.
    fn name(&self) -> &str;

    /// Get the version string.
    fn version(&self) -> &str;

    /// Get the plugin description.
    fn description(&self) -> Option<&str> {
        None
    }

    /// Get the backend type.
    fn backend(&self) -> Backend;

    /// Get capabilities this plugin provides.
    fn capabilities(&self) -> &[Capability];

    /// Initialize the plugin with app context.
    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()>;

    /// Clean shutdown.
    fn shutdown(&mut self) -> PluginResult<()>;

    /// Handle an event from the host application.
    fn on_event(&mut self, event: &PluginEvent) -> PluginResult<Option<PluginResponse>>;

    /// Get commands provided by this plugin.
    fn get_commands(&self) -> Vec<PluginCommand> {
        Vec::new()
    }

    /// Get keybindings provided by this plugin.
    fn get_keybindings(&self) -> Vec<PluginKeybinding> {
        Vec::new()
    }

    /// Check if the plugin is initialized.
    fn is_initialized(&self) -> bool;
}

/// A command provided by a plugin.
#[derive(Debug, Clone)]
pub struct PluginCommand {
    /// Command ID.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Description.
    pub description: Option<String>,
    /// Keywords for searching.
    pub keywords: Vec<String>,
    /// Category for grouping.
    pub category: Option<String>,
    /// Required parameters.
    pub params: Vec<CommandParam>,
}

/// A command parameter.
#[derive(Debug, Clone)]
pub struct CommandParam {
    /// Parameter name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Parameter type.
    pub param_type: ParamType,
    /// Whether required.
    pub required: bool,
    /// Default value.
    pub default: Option<serde_json::Value>,
}

/// Parameter types.
#[derive(Debug, Clone)]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Select(Vec<String>),
    File,
    Directory,
}

/// A keybinding provided by a plugin.
#[derive(Debug, Clone)]
pub struct PluginKeybinding {
    /// Key sequence (e.g., "ctrl+shift+p").
    pub keys: String,
    /// Command to execute.
    pub command: String,
    /// Context where binding is active.
    pub context: Option<String>,
    /// Description.
    pub description: Option<String>,
}

/// Plugin information for display.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin ID.
    pub id: String,
    /// Name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Description.
    pub description: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Backend type.
    pub backend: Backend,
    /// Capabilities.
    pub capabilities: Vec<String>,
    /// Is enabled.
    pub enabled: bool,
    /// Is initialized.
    pub initialized: bool,
}

impl PluginInfo {
    /// Create from a plugin reference.
    pub fn from_plugin(plugin: &dyn Plugin, enabled: bool) -> Self {
        Self {
            id: plugin.id().to_string(),
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            description: plugin.description().map(|s| s.to_string()),
            author: None,
            backend: plugin.backend(),
            capabilities: plugin.capabilities().iter().map(|c| c.name().to_string()).collect(),
            enabled,
            initialized: plugin.is_initialized(),
        }
    }
}

/// Plugin state tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Not yet loaded.
    Unloaded,
    /// Loaded but not initialized.
    Loaded,
    /// Initialized and ready.
    Ready,
    /// Disabled by user.
    Disabled,
    /// Error state.
    Error,
}

impl Default for PluginState {
    fn default() -> Self {
        Self::Unloaded
    }
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unloaded => write!(f, "unloaded"),
            Self::Loaded => write!(f, "loaded"),
            Self::Ready => write!(f, "ready"),
            Self::Disabled => write!(f, "disabled"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        initialized: bool,
    }

    impl Plugin for TestPlugin {
        fn id(&self) -> &str {
            "test-plugin"
        }

        fn name(&self) -> &str {
            "Test Plugin"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn backend(&self) -> Backend {
            Backend::Lua
        }

        fn capabilities(&self) -> &[Capability] {
            &[]
        }

        fn init(&mut self, _ctx: &PluginContext) -> PluginResult<()> {
            self.initialized = true;
            Ok(())
        }

        fn shutdown(&mut self) -> PluginResult<()> {
            self.initialized = false;
            Ok(())
        }

        fn on_event(&mut self, _event: &PluginEvent) -> PluginResult<Option<PluginResponse>> {
            Ok(None)
        }

        fn is_initialized(&self) -> bool {
            self.initialized
        }
    }

    #[test]
    fn test_plugin_trait() {
        let plugin = TestPlugin { initialized: false };
        assert_eq!(plugin.id(), "test-plugin");
        assert_eq!(plugin.name(), "Test Plugin");
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_plugin_info() {
        let plugin = TestPlugin { initialized: true };
        let info = PluginInfo::from_plugin(&plugin, true);

        assert_eq!(info.id, "test-plugin");
        assert!(info.enabled);
        assert!(info.initialized);
    }

    #[test]
    fn test_plugin_state() {
        assert_eq!(PluginState::default(), PluginState::Unloaded);
        assert_eq!(PluginState::Ready.to_string(), "ready");
    }
}
