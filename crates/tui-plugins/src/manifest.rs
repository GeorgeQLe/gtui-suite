//! Plugin manifest parsing.

use crate::error::{PluginError, PluginResult};
use crate::Backend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Plugin manifest (plugin.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Plugin metadata.
    pub plugin: ManifestPlugin,
    /// Capabilities.
    #[serde(default)]
    pub capabilities: ManifestCapabilities,
    /// Backend configuration.
    pub backend: ManifestBackend,
    /// Permissions.
    #[serde(default)]
    pub permissions: ManifestPermissions,
    /// Dependencies.
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

/// Plugin metadata section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPlugin {
    /// Unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Author.
    #[serde(default)]
    pub author: Option<String>,
    /// License.
    #[serde(default)]
    pub license: Option<String>,
    /// Homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,
    /// Repository URL.
    #[serde(default)]
    pub repository: Option<String>,
}

/// Capabilities section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestCapabilities {
    /// Provides commands.
    #[serde(default)]
    pub commands: bool,
    /// Provides keybindings.
    #[serde(default)]
    pub keybindings: bool,
    /// Provides theming.
    #[serde(default)]
    pub theming: bool,
    /// File extensions handled.
    #[serde(default)]
    pub file_extensions: Vec<String>,
    /// Provides transformer.
    #[serde(default)]
    pub transformer: bool,
    /// Custom capabilities.
    #[serde(default)]
    pub custom: Vec<String>,
}

/// Backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestBackend {
    /// Backend type.
    #[serde(rename = "type")]
    pub backend_type: String,
    /// Entry point file.
    pub entry: String,
}

/// Permission configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestPermissions {
    /// Network access allowed.
    #[serde(default)]
    pub network: bool,
    /// Allowed network hosts.
    #[serde(default)]
    pub network_hosts: Vec<String>,
    /// Filesystem paths allowed.
    #[serde(default)]
    pub filesystem: Vec<String>,
    /// Environment variables allowed.
    #[serde(default)]
    pub env_vars: Vec<String>,
    /// Subprocesses allowed.
    #[serde(default)]
    pub subprocess: bool,
}

impl Manifest {
    /// Load manifest from a file.
    pub fn load(path: &Path) -> PluginResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse manifest from TOML string.
    pub fn parse(content: &str) -> PluginResult<Self> {
        toml::from_str(content).map_err(PluginError::from)
    }

    /// Get the backend type.
    pub fn backend(&self) -> PluginResult<Backend> {
        self.backend.backend_type.parse()
    }

    /// Validate the manifest.
    pub fn validate(&self) -> PluginResult<()> {
        // Check required fields
        if self.plugin.id.is_empty() {
            return Err(PluginError::ManifestError("Missing plugin.id".to_string()));
        }
        if self.plugin.name.is_empty() {
            return Err(PluginError::ManifestError("Missing plugin.name".to_string()));
        }
        if self.plugin.version.is_empty() {
            return Err(PluginError::ManifestError("Missing plugin.version".to_string()));
        }
        if self.backend.entry.is_empty() {
            return Err(PluginError::ManifestError("Missing backend.entry".to_string()));
        }

        // Validate backend type
        let _ = self.backend()?;

        Ok(())
    }

    /// Get the entry point path relative to manifest directory.
    pub fn entry_path(&self, manifest_dir: &Path) -> std::path::PathBuf {
        manifest_dir.join(&self.backend.entry)
    }
}

impl ManifestCapabilities {
    /// Check if any capability is enabled.
    pub fn has_any(&self) -> bool {
        self.commands
            || self.keybindings
            || self.theming
            || self.transformer
            || !self.file_extensions.is_empty()
            || !self.custom.is_empty()
    }

    /// Get list of capability names.
    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if self.commands {
            names.push("commands".to_string());
        }
        if self.keybindings {
            names.push("keybindings".to_string());
        }
        if self.theming {
            names.push("theming".to_string());
        }
        if self.transformer {
            names.push("transformer".to_string());
        }
        if !self.file_extensions.is_empty() {
            names.push("file_handler".to_string());
        }
        names.extend(self.custom.iter().cloned());
        names
    }
}

impl ManifestPermissions {
    /// Check if any permission is requested.
    pub fn has_any(&self) -> bool {
        self.network || self.subprocess || !self.filesystem.is_empty() || !self.env_vars.is_empty()
    }

    /// Get list of requested permissions.
    pub fn summary(&self) -> Vec<String> {
        let mut perms = Vec::new();
        if self.network {
            if self.network_hosts.is_empty() {
                perms.push("network (all hosts)".to_string());
            } else {
                perms.push(format!("network ({} hosts)", self.network_hosts.len()));
            }
        }
        if !self.filesystem.is_empty() {
            perms.push(format!("filesystem ({} paths)", self.filesystem.len()));
        }
        if !self.env_vars.is_empty() {
            perms.push(format!("env_vars ({} vars)", self.env_vars.len()));
        }
        if self.subprocess {
            perms.push("subprocess".to_string());
        }
        perms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_MANIFEST: &str = r#"
[plugin]
id = "my-plugin"
name = "My Plugin"
version = "1.0.0"
description = "Example plugin"
author = "Test Author"

[capabilities]
commands = true
keybindings = true

[backend]
type = "lua"
entry = "plugin.lua"

[permissions]
network = false
filesystem = ["~/.config/my-plugin/*"]
"#;

    #[test]
    fn test_parse_manifest() {
        let manifest = Manifest::parse(EXAMPLE_MANIFEST).unwrap();

        assert_eq!(manifest.plugin.id, "my-plugin");
        assert_eq!(manifest.plugin.name, "My Plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert!(manifest.capabilities.commands);
        assert!(manifest.capabilities.keybindings);
        assert!(!manifest.capabilities.theming);
        assert_eq!(manifest.backend.backend_type, "lua");
    }

    #[test]
    fn test_validate() {
        let manifest = Manifest::parse(EXAMPLE_MANIFEST).unwrap();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_backend_detection() {
        let manifest = Manifest::parse(EXAMPLE_MANIFEST).unwrap();
        assert_eq!(manifest.backend().unwrap(), Backend::Lua);
    }

    #[test]
    fn test_capabilities_names() {
        let manifest = Manifest::parse(EXAMPLE_MANIFEST).unwrap();
        let names = manifest.capabilities.names();
        assert!(names.contains(&"commands".to_string()));
        assert!(names.contains(&"keybindings".to_string()));
    }

    #[test]
    fn test_permissions_summary() {
        let manifest = Manifest::parse(EXAMPLE_MANIFEST).unwrap();
        let summary = manifest.permissions.summary();
        assert!(summary.iter().any(|s| s.contains("filesystem")));
    }

    #[test]
    fn test_invalid_manifest() {
        let invalid = r#"
[plugin]
name = "Missing ID"
version = "1.0.0"

[backend]
type = "lua"
entry = "plugin.lua"
"#;
        let manifest = Manifest::parse(invalid);
        assert!(manifest.is_err());
    }
}
