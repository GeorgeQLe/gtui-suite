//! Plugin manager for loading and managing plugins.

use crate::capability::Capability;
use crate::context::PluginContext;
use crate::error::{PluginError, PluginResult};
use crate::event::PluginEvent;
use crate::manifest::Manifest;
use crate::plugin::{Plugin, PluginInfo, PluginState};
use crate::response::PluginResponse;
use crate::sandbox::SandboxConfig;
use crate::Backend;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration for the plugin manager.
#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    /// Application name.
    pub app_name: String,
    /// Application version.
    pub app_version: String,
    /// Plugin search directories.
    pub plugin_dirs: Vec<PathBuf>,
    /// Default sandbox configuration.
    pub sandbox: SandboxConfig,
    /// Whether to auto-discover plugins.
    pub auto_discover: bool,
    /// Enabled backends.
    pub enabled_backends: Vec<Backend>,
    /// Disabled plugin IDs.
    pub disabled_plugins: Vec<String>,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            app_name: "tui-app".to_string(),
            app_version: "0.1.0".to_string(),
            plugin_dirs: Self::default_plugin_dirs("tui-app"),
            sandbox: SandboxConfig::default(),
            auto_discover: true,
            enabled_backends: vec![Backend::Lua],
            disabled_plugins: Vec::new(),
        }
    }
}

impl PluginManagerConfig {
    /// Create config for a specific app.
    pub fn for_app(app_name: impl Into<String>, version: impl Into<String>) -> Self {
        let app_name = app_name.into();
        Self {
            plugin_dirs: Self::default_plugin_dirs(&app_name),
            app_name,
            app_version: version.into(),
            ..Default::default()
        }
    }

    /// Get default plugin directories.
    fn default_plugin_dirs(app_name: &str) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // User config directory
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", app_name) {
            dirs.push(proj_dirs.config_dir().join("plugins"));
            dirs.push(proj_dirs.data_dir().join("plugins"));
        }

        // System directories
        #[cfg(unix)]
        {
            dirs.push(PathBuf::from(format!("/usr/share/{}/plugins", app_name)));
            dirs.push(PathBuf::from(format!("/usr/local/share/{}/plugins", app_name)));
        }

        dirs
    }

    /// Add a plugin directory.
    pub fn add_plugin_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.plugin_dirs.push(dir.into());
        self
    }

    /// Enable a backend.
    pub fn enable_backend(mut self, backend: Backend) -> Self {
        if !self.enabled_backends.contains(&backend) {
            self.enabled_backends.push(backend);
        }
        self
    }

    /// Disable a plugin by ID.
    pub fn disable_plugin(mut self, id: impl Into<String>) -> Self {
        self.disabled_plugins.push(id.into());
        self
    }

    /// Set sandbox configuration.
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.sandbox = sandbox;
        self
    }
}

/// Manages plugin lifecycle.
pub struct PluginManager {
    config: PluginManagerConfig,
    plugins: HashMap<String, PluginEntry>,
    context: PluginContext,
}

/// Entry for a loaded plugin.
#[allow(dead_code)]
struct PluginEntry {
    plugin: Box<dyn Plugin>,
    state: PluginState,
    manifest: Option<Manifest>,
}

impl PluginManager {
    /// Create a new plugin manager.
    pub fn new(config: PluginManagerConfig) -> Self {
        let context = PluginContext::new(
            &config.app_name,
            &config.app_version,
            config.plugin_dirs.first().cloned().unwrap_or_default(),
            config.plugin_dirs.first().cloned().unwrap_or_default(),
        );

        Self {
            config,
            plugins: HashMap::new(),
            context,
        }
    }

    /// Set the plugin context.
    pub fn set_context(&mut self, context: PluginContext) {
        self.context = context;
    }

    /// Discover and load all plugins from configured directories.
    pub fn discover_and_load(&mut self) -> PluginResult<Vec<String>> {
        let mut loaded = Vec::new();

        for dir in &self.config.plugin_dirs.clone() {
            if !dir.exists() {
                continue;
            }

            // Look for plugin.toml files
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    // Plugin directory with manifest
                    if path.is_dir() {
                        let manifest_path = path.join("plugin.toml");
                        if manifest_path.exists() {
                            match self.load_from_manifest(&manifest_path) {
                                Ok(id) => loaded.push(id),
                                Err(e) => {
                                    eprintln!("Failed to load plugin from {:?}: {}", path, e);
                                }
                            }
                        }
                    }

                    // Direct plugin file
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if Backend::from_extension(ext).is_some() {
                                match self.load(&path) {
                                    Ok(id) => loaded.push(id),
                                    Err(e) => {
                                        eprintln!("Failed to load plugin {:?}: {}", path, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(loaded)
    }

    /// Load a plugin from a manifest file.
    pub fn load_from_manifest(&mut self, manifest_path: &Path) -> PluginResult<String> {
        let manifest = Manifest::load(manifest_path)?;
        manifest.validate()?;

        let backend = manifest.backend()?;

        if !self.config.enabled_backends.contains(&backend) {
            return Err(PluginError::BackendNotAvailable(backend.to_string()));
        }

        if self.config.disabled_plugins.contains(&manifest.plugin.id) {
            return Err(PluginError::PermissionDenied(format!(
                "Plugin {} is disabled",
                manifest.plugin.id
            )));
        }

        let plugin_dir = manifest_path.parent().unwrap_or(Path::new("."));
        let entry_path = manifest.entry_path(plugin_dir);

        let plugin = self.create_plugin(backend, &entry_path, Some(&manifest))?;
        let id = plugin.id().to_string();

        self.plugins.insert(
            id.clone(),
            PluginEntry {
                plugin,
                state: PluginState::Loaded,
                manifest: Some(manifest),
            },
        );

        Ok(id)
    }

    /// Load a plugin from a file path.
    pub fn load(&mut self, path: &Path) -> PluginResult<String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| PluginError::InvalidBackend("Unknown file type".to_string()))?;

        let backend = Backend::from_extension(ext)
            .ok_or_else(|| PluginError::InvalidBackend(ext.to_string()))?;

        if !self.config.enabled_backends.contains(&backend) {
            return Err(PluginError::BackendNotAvailable(backend.to_string()));
        }

        let plugin = self.create_plugin(backend, path, None)?;
        let id = plugin.id().to_string();

        if self.config.disabled_plugins.contains(&id) {
            return Err(PluginError::PermissionDenied(format!(
                "Plugin {} is disabled",
                id
            )));
        }

        self.plugins.insert(
            id.clone(),
            PluginEntry {
                plugin,
                state: PluginState::Loaded,
                manifest: None,
            },
        );

        Ok(id)
    }

    /// Create a plugin instance based on backend.
    fn create_plugin(
        &self,
        backend: Backend,
        path: &Path,
        _manifest: Option<&Manifest>,
    ) -> PluginResult<Box<dyn Plugin>> {
        match backend {
            #[cfg(feature = "lua")]
            Backend::Lua => {
                crate::lua::LuaPlugin::load(path, &self.config.sandbox)
            }
            #[cfg(feature = "wasm")]
            Backend::Wasm => {
                crate::wasm::WasmPlugin::load(path, &self.config.sandbox)
            }
            #[cfg(feature = "native")]
            Backend::Native => {
                crate::native::NativePlugin::load(path)
            }
            #[allow(unreachable_patterns)]
            _ => Err(PluginError::BackendNotAvailable(backend.to_string())),
        }
    }

    /// Initialize a plugin.
    pub fn init(&mut self, id: &str) -> PluginResult<()> {
        let entry = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        if entry.state == PluginState::Ready {
            return Ok(());
        }

        entry.plugin.init(&self.context)?;
        entry.state = PluginState::Ready;

        Ok(())
    }

    /// Initialize all loaded plugins.
    pub fn init_all(&mut self) -> Vec<(String, PluginResult<()>)> {
        let ids: Vec<_> = self.plugins.keys().cloned().collect();
        ids.into_iter()
            .map(|id| {
                let result = self.init(&id);
                (id, result)
            })
            .collect()
    }

    /// Unload a plugin.
    pub fn unload(&mut self, id: &str) -> PluginResult<()> {
        let mut entry = self
            .plugins
            .remove(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        if entry.state == PluginState::Ready {
            entry.plugin.shutdown()?;
        }

        Ok(())
    }

    /// Get a plugin by ID.
    pub fn get(&self, id: &str) -> Option<&(dyn Plugin + '_)> {
        self.plugins.get(id).map(|e| e.plugin.as_ref())
    }

    /// Get a mutable plugin by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut (dyn Plugin + '_)> {
        match self.plugins.get_mut(id) {
            Some(entry) => Some(entry.plugin.as_mut()),
            None => None,
        }
    }

    /// Check if a plugin is loaded.
    pub fn is_loaded(&self, id: &str) -> bool {
        self.plugins.contains_key(id)
    }

    /// Get plugin state.
    pub fn state(&self, id: &str) -> Option<PluginState> {
        self.plugins.get(id).map(|e| e.state)
    }

    /// Get all plugin IDs.
    pub fn plugin_ids(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get info for all plugins.
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .map(|e| {
                PluginInfo::from_plugin(e.plugin.as_ref(), e.state == PluginState::Ready)
            })
            .collect()
    }

    /// Iterate over all plugins.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Plugin> {
        self.plugins.values().map(|e| e.plugin.as_ref())
    }

    /// Broadcast an event to all ready plugins.
    pub fn broadcast(&mut self, event: &PluginEvent) -> Vec<PluginResponse> {
        let mut responses = Vec::new();

        for entry in self.plugins.values_mut() {
            if entry.state != PluginState::Ready {
                continue;
            }

            match entry.plugin.on_event(event) {
                Ok(Some(response)) => responses.push(response),
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Plugin {} error: {}", entry.plugin.id(), e);
                }
            }
        }

        responses
    }

    /// Send an event to a specific plugin.
    pub fn send(
        &mut self,
        id: &str,
        event: &PluginEvent,
    ) -> PluginResult<Option<PluginResponse>> {
        let entry = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        if entry.state != PluginState::Ready {
            return Err(PluginError::InvalidState(format!(
                "Plugin {} is not ready (state: {})",
                id, entry.state
            )));
        }

        entry.plugin.on_event(event)
    }

    /// Get plugins with a specific capability.
    pub fn with_capability(&self, capability: &Capability) -> Vec<&dyn Plugin> {
        self.plugins
            .values()
            .filter(|e| {
                e.state == PluginState::Ready
                    && e.plugin.capabilities().iter().any(|c| c.name() == capability.name())
            })
            .map(|e| e.plugin.as_ref())
            .collect()
    }

    /// Shutdown all plugins.
    pub fn shutdown_all(&mut self) -> Vec<(String, PluginResult<()>)> {
        let ids: Vec<_> = self.plugins.keys().cloned().collect();
        ids.into_iter()
            .map(|id| {
                let result = self.unload(&id);
                (id, result)
            })
            .collect()
    }

    /// Get plugin count.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = PluginManagerConfig::for_app("test-app", "1.0.0");
        assert_eq!(config.app_name, "test-app");
        assert_eq!(config.app_version, "1.0.0");
    }

    #[test]
    fn test_manager_creation() {
        let config = PluginManagerConfig::default();
        let manager = PluginManager::new(config);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_plugin_ids() {
        let config = PluginManagerConfig::default();
        let manager = PluginManager::new(config);
        assert!(manager.plugin_ids().is_empty());
    }
}
