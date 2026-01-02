//! Multi-backend plugin system for TUI applications.
//!
//! This crate provides a unified plugin architecture supporting:
//! - Lua plugins (Phase 1) - Most accessible, fastest to implement
//! - WASM plugins (Phase 2) - Language-agnostic, secure sandbox
//! - Native plugins (Phase 3) - Maximum performance
//!
//! # Example
//!
//! ```ignore
//! use tui_plugins::{PluginManager, PluginConfig};
//!
//! let mut manager = PluginManager::new(PluginConfig::default());
//! manager.discover_and_load()?;
//!
//! for plugin in manager.iter() {
//!     println!("Loaded: {} v{}", plugin.name(), plugin.version());
//! }
//! ```

pub mod capability;
pub mod context;
pub mod error;
pub mod event;
pub mod manifest;
pub mod manager;
pub mod plugin;
pub mod response;
pub mod sandbox;

#[cfg(feature = "lua")]
pub mod lua;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "native")]
pub mod native;

// Re-exports
pub use capability::Capability;
pub use context::PluginContext;
pub use error::PluginError;
pub use event::PluginEvent;
pub use manifest::{Manifest, ManifestBackend, ManifestCapabilities, ManifestPermissions};
pub use manager::{PluginManager, PluginManagerConfig};
pub use plugin::Plugin;
pub use response::PluginResponse;
pub use sandbox::SandboxConfig;

/// Plugin backend type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Lua scripting backend.
    Lua,
    /// WebAssembly backend.
    Wasm,
    /// Native dynamic library backend.
    Native,
}

impl Backend {
    /// Detect backend from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "lua" => Some(Self::Lua),
            "wasm" => Some(Self::Wasm),
            "so" | "dll" | "dylib" => Some(Self::Native),
            _ => None,
        }
    }

    /// Get the typical file extension for this backend.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Lua => "lua",
            Self::Wasm => "wasm",
            Self::Native => {
                #[cfg(target_os = "windows")]
                {
                    "dll"
                }
                #[cfg(target_os = "macos")]
                {
                    "dylib"
                }
                #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                {
                    "so"
                }
            }
        }
    }

    /// Check if this backend is available (compiled in).
    pub fn is_available(&self) -> bool {
        match self {
            Self::Lua => cfg!(feature = "lua"),
            Self::Wasm => cfg!(feature = "wasm"),
            Self::Native => cfg!(feature = "native"),
        }
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lua => write!(f, "lua"),
            Self::Wasm => write!(f, "wasm"),
            Self::Native => write!(f, "native"),
        }
    }
}

impl std::str::FromStr for Backend {
    type Err = PluginError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lua" => Ok(Self::Lua),
            "wasm" => Ok(Self::Wasm),
            "native" => Ok(Self::Native),
            _ => Err(PluginError::InvalidBackend(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_from_extension() {
        assert_eq!(Backend::from_extension("lua"), Some(Backend::Lua));
        assert_eq!(Backend::from_extension("wasm"), Some(Backend::Wasm));
        assert_eq!(Backend::from_extension("so"), Some(Backend::Native));
        assert_eq!(Backend::from_extension("dll"), Some(Backend::Native));
        assert_eq!(Backend::from_extension("dylib"), Some(Backend::Native));
        assert_eq!(Backend::from_extension("unknown"), None);
    }

    #[test]
    fn test_backend_display() {
        assert_eq!(Backend::Lua.to_string(), "lua");
        assert_eq!(Backend::Wasm.to_string(), "wasm");
        assert_eq!(Backend::Native.to_string(), "native");
    }

    #[test]
    fn test_backend_from_str() {
        assert_eq!("lua".parse::<Backend>().unwrap(), Backend::Lua);
        assert_eq!("WASM".parse::<Backend>().unwrap(), Backend::Wasm);
        assert_eq!("Native".parse::<Backend>().unwrap(), Backend::Native);
        assert!("unknown".parse::<Backend>().is_err());
    }
}
