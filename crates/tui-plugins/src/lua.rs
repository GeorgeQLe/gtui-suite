//! Lua plugin backend.

use crate::capability::Capability;
use crate::context::PluginContext;
use crate::error::{PluginError, PluginResult};
use crate::event::PluginEvent;
use crate::plugin::{Plugin, PluginCommand};
use crate::response::PluginResponse;
use crate::sandbox::SandboxConfig;
use crate::Backend;
use mlua::{Function, Lua, Table, Value};
use std::path::Path;
use std::sync::Arc;

/// A Lua-based plugin.
pub struct LuaPlugin {
    /// Plugin ID.
    id: String,
    /// Plugin name.
    name: String,
    /// Plugin version.
    version: String,
    /// Plugin description.
    description: Option<String>,
    /// Capabilities.
    capabilities: Vec<Capability>,
    /// Lua runtime.
    lua: Lua,
    /// Whether initialized.
    initialized: bool,
    /// Sandbox configuration.
    sandbox: SandboxConfig,
}

impl LuaPlugin {
    /// Load a Lua plugin from a file.
    pub fn load(path: &Path, sandbox: &SandboxConfig) -> PluginResult<Box<dyn Plugin>> {
        let lua = Lua::new();

        // Configure sandbox
        Self::setup_sandbox(&lua, sandbox)?;

        // Load and execute the plugin file
        let source = std::fs::read_to_string(path)?;
        let plugin_table: Table = lua.load(&source).eval()?;

        // Extract metadata
        let id: String = plugin_table
            .get("id")
            .map_err(|_| PluginError::ManifestError("Missing plugin.id".to_string()))?;

        let name: String = plugin_table
            .get("name")
            .unwrap_or_else(|_| id.clone());

        let version: String = plugin_table
            .get("version")
            .unwrap_or_else(|_| "0.0.0".to_string());

        let description: Option<String> = plugin_table.get("description").ok();

        // Parse capabilities
        let capabilities = Self::parse_capabilities(&plugin_table)?;

        // Store the plugin table in registry for later access
        lua.set_named_registry_value("plugin", plugin_table)?;

        Ok(Box::new(Self {
            id,
            name,
            version,
            description,
            capabilities,
            lua,
            initialized: false,
            sandbox: sandbox.clone(),
        }))
    }

    /// Setup the Lua sandbox.
    fn setup_sandbox(lua: &Lua, sandbox: &SandboxConfig) -> PluginResult<()> {
        // Set memory limit
        lua.set_memory_limit(sandbox.memory_limit)?;

        // Remove dangerous globals
        let globals = lua.globals();

        // Remove os module if not allowed
        if !sandbox.is_lua_module_allowed("os") {
            globals.set("os", Value::Nil)?;
        }

        // Remove io module if not allowed
        if !sandbox.is_lua_module_allowed("io") {
            globals.set("io", Value::Nil)?;
        }

        // Remove debug module if not allowed
        if !sandbox.is_lua_module_allowed("debug") {
            globals.set("debug", Value::Nil)?;
        }

        // Remove package/require if not allowed
        if !sandbox.is_lua_module_allowed("package") {
            globals.set("package", Value::Nil)?;
            globals.set("require", Value::Nil)?;
            globals.set("dofile", Value::Nil)?;
            globals.set("loadfile", Value::Nil)?;
        }

        // Setup TUI API
        Self::setup_tui_api(lua, sandbox)?;

        Ok(())
    }

    /// Setup the TUI API for plugins.
    fn setup_tui_api(lua: &Lua, _sandbox: &SandboxConfig) -> PluginResult<()> {
        let tui = lua.create_table()?;

        // tui.log(message)
        let log_fn = lua.create_function(|_, msg: String| {
            println!("[plugin] {}", msg);
            Ok(())
        })?;
        tui.set("log", log_fn)?;

        // tui.notify(message)
        let notify_fn = lua.create_function(|_, msg: String| {
            println!("[notify] {}", msg);
            Ok(())
        })?;
        tui.set("notify", notify_fn)?;

        // Set global tui table
        lua.globals().set("tui", tui)?;

        Ok(())
    }

    /// Parse capabilities from plugin table.
    fn parse_capabilities(table: &Table) -> PluginResult<Vec<Capability>> {
        let mut capabilities = Vec::new();

        if let Ok(caps) = table.get::<Table>("capabilities") {
            for pair in caps.pairs::<usize, String>() {
                if let Ok((_, cap_name)) = pair {
                    capabilities.push(Capability::from_str_simple(&cap_name));
                }
            }
        }

        Ok(capabilities)
    }

    /// Get the plugin table from registry.
    fn get_plugin_table(&self) -> PluginResult<Table> {
        self.lua
            .named_registry_value("plugin")
            .map_err(|e| PluginError::Lua(e.to_string()))
    }

    /// Call a function on the plugin table.
    fn call_plugin_fn<'lua, A, R>(&'lua self, name: &str, args: A) -> PluginResult<Option<R>>
    where
        A: mlua::IntoLuaMulti<'lua>,
        R: mlua::FromLuaMulti<'lua>,
    {
        let plugin_table = self.get_plugin_table()?;

        match plugin_table.get::<Function>(name) {
            Ok(func) => {
                let result = func.call::<R>(args)?;
                Ok(Some(result))
            }
            Err(mlua::Error::FromLuaConversionError { .. }) => Ok(None),
            Err(e) => Err(PluginError::Lua(e.to_string())),
        }
    }
}

impl Plugin for LuaPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn backend(&self) -> Backend {
        Backend::Lua
    }

    fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    fn init(&mut self, ctx: &PluginContext) -> PluginResult<()> {
        // Create context table for Lua
        let ctx_table = self.lua.create_table()?;
        ctx_table.set("app_name", ctx.app_name())?;
        ctx_table.set("app_version", ctx.app_version())?;

        // Add log function to context
        let log_fn = self.lua.create_function(|_, msg: String| {
            println!("[plugin] {}", msg);
            Ok(())
        })?;
        ctx_table.set("log", log_fn)?;

        // Call init function if it exists
        let _: Option<()> = self.call_plugin_fn("init", ctx_table)?;

        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        if self.initialized {
            let _: Option<()> = self.call_plugin_fn("shutdown", ())?;
            self.initialized = false;
        }
        Ok(())
    }

    fn on_event(&mut self, event: &PluginEvent) -> PluginResult<Option<PluginResponse>> {
        if !self.initialized {
            return Err(PluginError::InvalidState("Plugin not initialized".to_string()));
        }

        // Convert event to Lua table
        let event_json = serde_json::to_string(event)?;
        let event_table: Value = self.lua.load(&format!("return {}",
            event_json.replace("\"", "'")
                .replace("null", "nil")
        )).eval().unwrap_or(Value::Nil);

        // Actually, let's use a simpler approach - create a table directly
        let event_table = self.lua.create_table()?;
        event_table.set("type", event.event_type())?;
        event_table.set("data", event_json)?;

        // Call on_event
        let result: Option<Table> = self.call_plugin_fn("on_event", event_table)?;

        if let Some(response_table) = result {
            // Parse response from Lua table
            let action: String = response_table.get("action").unwrap_or_default();
            let message: Option<String> = response_table.get("message").ok();

            if action == "notify" {
                if let Some(msg) = message {
                    return Ok(Some(PluginResponse::notify(msg)));
                }
            }

            // Handle other actions...
        }

        Ok(None)
    }

    fn get_commands(&self) -> Vec<PluginCommand> {
        let mut commands = Vec::new();

        if let Ok(plugin_table) = self.get_plugin_table() {
            if let Ok(cmds) = plugin_table.get::<Table>("commands") {
                for pair in cmds.pairs::<String, Table>() {
                    if let Ok((id, cmd_table)) = pair {
                        let label: String = cmd_table.get("label").unwrap_or_else(|_| id.clone());
                        let description: Option<String> = cmd_table.get("description").ok();

                        commands.push(PluginCommand {
                            id: format!("{}:{}", self.id, id),
                            label,
                            description,
                            keywords: Vec::new(),
                            category: Some(self.name.clone()),
                            params: Vec::new(),
                        });
                    }
                }
            }
        }

        commands
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_plugin() -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".lua").unwrap();
        write!(
            file,
            r#"
local plugin = {{}}
plugin.id = "test-plugin"
plugin.name = "Test Plugin"
plugin.version = "1.0.0"
plugin.capabilities = {{ "commands" }}

function plugin.init(ctx)
    ctx.log("Plugin initialized")
end

function plugin.on_event(event)
    if event.type == "command" then
        return {{ action = "notify", message = "Hello!" }}
    end
    return nil
end

function plugin.shutdown()
end

return plugin
"#
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_plugin() {
        let file = create_test_plugin();
        let sandbox = SandboxConfig::default();
        let plugin = LuaPlugin::load(file.path(), &sandbox).unwrap();

        assert_eq!(plugin.id(), "test-plugin");
        assert_eq!(plugin.name(), "Test Plugin");
        assert_eq!(plugin.version(), "1.0.0");
        assert_eq!(plugin.backend(), Backend::Lua);
    }

    #[test]
    fn test_plugin_init() {
        let file = create_test_plugin();
        let sandbox = SandboxConfig::default();
        let mut plugin = LuaPlugin::load(file.path(), &sandbox).unwrap();

        let ctx = PluginContext::new(
            "test-app",
            "1.0.0",
            std::path::PathBuf::from("/tmp"),
            std::path::PathBuf::from("/tmp"),
        );

        plugin.init(&ctx).unwrap();
        assert!(plugin.is_initialized());
    }
}
