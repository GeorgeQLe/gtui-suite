//! Native dynamic library plugin backend.
//!
//! This module provides support for native plugins compiled as dynamic libraries.
//! Native plugins have full system access and should only be loaded from trusted sources.

use crate::capability::Capability;
use crate::context::PluginContext;
use crate::error::{PluginError, PluginResult};
use crate::event::PluginEvent;
use crate::plugin::Plugin;
use crate::response::PluginResponse;
use crate::Backend;
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

/// FFI plugin info structure.
#[repr(C)]
pub struct FfiPluginInfo {
    pub id: *const c_char,
    pub name: *const c_char,
    pub version: *const c_char,
}

/// Type for plugin_info function.
type PluginInfoFn = unsafe extern "C" fn() -> *const FfiPluginInfo;

/// Type for plugin_init function.
type PluginInitFn = unsafe extern "C" fn() -> i32;

/// Type for plugin_shutdown function.
type PluginShutdownFn = unsafe extern "C" fn() -> i32;

/// A native dynamic library plugin.
pub struct NativePlugin {
    /// Plugin ID.
    id: String,
    /// Plugin name.
    name: String,
    /// Plugin version.
    version: String,
    /// Capabilities.
    capabilities: Vec<Capability>,
    /// The loaded library (kept alive).
    #[allow(dead_code)]
    library: Library,
    /// Whether initialized.
    initialized: bool,
}

impl NativePlugin {
    /// Load a native plugin from a dynamic library.
    ///
    /// # Safety
    ///
    /// Loading native plugins is inherently unsafe as they have full system access.
    /// Only load plugins from trusted sources.
    pub fn load(path: &Path) -> PluginResult<Box<dyn Plugin>> {
        // Safety: Loading dynamic libraries is unsafe
        let library = unsafe {
            Library::new(path).map_err(|e| PluginError::Native(e.to_string()))?
        };

        // Get plugin info
        let (id, name, version) = unsafe {
            let info_fn: Symbol<PluginInfoFn> = library
                .get(b"plugin_info\0")
                .map_err(|e| PluginError::Native(format!("Missing plugin_info: {}", e)))?;

            let info_ptr = info_fn();
            if info_ptr.is_null() {
                return Err(PluginError::Native("plugin_info returned null".to_string()));
            }

            let info = &*info_ptr;

            let id = if info.id.is_null() {
                "unknown".to_string()
            } else {
                CStr::from_ptr(info.id).to_string_lossy().into_owned()
            };

            let name = if info.name.is_null() {
                id.clone()
            } else {
                CStr::from_ptr(info.name).to_string_lossy().into_owned()
            };

            let version = if info.version.is_null() {
                "0.0.0".to_string()
            } else {
                CStr::from_ptr(info.version).to_string_lossy().into_owned()
            };

            (id, name, version)
        };

        Ok(Box::new(Self {
            id,
            name,
            version,
            capabilities: Vec::new(),
            library,
            initialized: false,
        }))
    }
}

impl Plugin for NativePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn backend(&self) -> Backend {
        Backend::Native
    }

    fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    fn init(&mut self, _ctx: &PluginContext) -> PluginResult<()> {
        // Safety: Calling into native plugin
        unsafe {
            if let Ok(init_fn) = self.library.get::<PluginInitFn>(b"plugin_init\0") {
                let result = init_fn();
                if result != 0 {
                    return Err(PluginError::InitError(format!(
                        "plugin_init returned {}",
                        result
                    )));
                }
            }
        }

        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        if self.initialized {
            // Safety: Calling into native plugin
            unsafe {
                if let Ok(shutdown_fn) = self.library.get::<PluginShutdownFn>(b"plugin_shutdown\0") {
                    let result = shutdown_fn();
                    if result != 0 {
                        return Err(PluginError::ExecutionError(format!(
                            "plugin_shutdown returned {}",
                            result
                        )));
                    }
                }
            }
            self.initialized = false;
        }
        Ok(())
    }

    fn on_event(&mut self, _event: &PluginEvent) -> PluginResult<Option<PluginResponse>> {
        if !self.initialized {
            return Err(PluginError::InvalidState("Plugin not initialized".to_string()));
        }

        // TODO: Implement event handling via FFI
        Ok(None)
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Check if native plugins are supported on this platform.
pub fn is_supported() -> bool {
    true
}

/// Display a security warning about loading native plugins.
pub fn security_warning() -> &'static str {
    "WARNING: Native plugins have full system access. Only load plugins from trusted sources."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_backend() {
        assert!(is_supported());
        assert!(security_warning().contains("WARNING"));
    }

    #[test]
    fn test_backend_extension() {
        #[cfg(target_os = "windows")]
        assert_eq!(Backend::Native.extension(), "dll");

        #[cfg(target_os = "macos")]
        assert_eq!(Backend::Native.extension(), "dylib");

        #[cfg(target_os = "linux")]
        assert_eq!(Backend::Native.extension(), "so");
    }
}
