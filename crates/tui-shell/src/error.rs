//! Shell error types.

use thiserror::Error;

/// Errors that can occur in the shell.
#[derive(Debug, Error)]
pub enum ShellError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// App error.
    #[error("App error: {0}")]
    App(String),

    /// Task error.
    #[error("Task error: {0}")]
    Task(String),

    /// App not found.
    #[error("App not found: {0}")]
    AppNotFound(String),

    /// App already running.
    #[error("App already running: {0}")]
    AppAlreadyRunning(String),

    /// App launch failed.
    #[error("Failed to launch app: {0}")]
    LaunchFailed(String),

    /// IPC error.
    #[error("IPC error: {0}")]
    Ipc(String),

    /// Session error.
    #[error("Session error: {0}")]
    Session(String),

    /// Workspace error.
    #[error("Workspace error: {0}")]
    Workspace(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML error.
    #[error("TOML error: {0}")]
    Toml(String),

    /// Invalid state.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Timeout.
    #[error("Operation timed out")]
    Timeout,

    /// App crashed.
    #[error("App crashed: {0}")]
    AppCrashed(String),
}

impl From<toml::de::Error> for ShellError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e.to_string())
    }
}

/// Result type for shell operations.
pub type ShellResult<T> = Result<T, ShellError>;
