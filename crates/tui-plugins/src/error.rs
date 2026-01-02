//! Plugin error types.

use thiserror::Error;

/// Errors that can occur during plugin operations.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found.
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Plugin already loaded.
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),

    /// Invalid backend type.
    #[error("Invalid backend: {0}")]
    InvalidBackend(String),

    /// Backend not available (not compiled in).
    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),

    /// Manifest parsing error.
    #[error("Manifest error: {0}")]
    ManifestError(String),

    /// Plugin initialization failed.
    #[error("Initialization failed: {0}")]
    InitError(String),

    /// Plugin execution error.
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Sandbox violation.
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Resource limit exceeded.
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing error.
    #[error("TOML error: {0}")]
    Toml(String),

    /// Lua error.
    #[cfg(feature = "lua")]
    #[error("Lua error: {0}")]
    Lua(String),

    /// WASM error.
    #[cfg(feature = "wasm")]
    #[error("WASM error: {0}")]
    Wasm(String),

    /// Native plugin error.
    #[cfg(feature = "native")]
    #[error("Native plugin error: {0}")]
    Native(String),

    /// Plugin returned an error.
    #[error("Plugin error: {0}")]
    PluginReturned(String),

    /// Timeout.
    #[error("Plugin operation timed out")]
    Timeout,

    /// Invalid state.
    #[error("Invalid plugin state: {0}")]
    InvalidState(String),
}

impl From<toml::de::Error> for PluginError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e.to_string())
    }
}

#[cfg(feature = "lua")]
impl From<mlua::Error> for PluginError {
    fn from(e: mlua::Error) -> Self {
        Self::Lua(e.to_string())
    }
}

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;
