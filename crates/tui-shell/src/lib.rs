//! Multi-app orchestration shell for TUI applications.
//!
//! This crate provides a unified shell environment for running multiple TUI apps
//! simultaneously with shared notifications, context switching, and background
//! task coordination.
//!
//! # Features
//!
//! - **App Lifecycle**: Launch, suspend, resume, and manage multiple apps
//! - **Unified Notifications**: Priority queue with expandable panel
//! - **Context Switching**: Multiple methods (recent, numbered, fuzzy search, workspace)
//! - **IPC**: Unix domain sockets with JSON messages
//! - **Session Management**: Full state persistence and restore
//! - **Multi-Workspace**: Apps can be visible in multiple workspaces

pub mod app;
pub mod compositor;
pub mod config;
pub mod error;
pub mod ipc;
pub mod launcher;
pub mod notification;
pub mod prefix;
pub mod session;
pub mod task;
pub mod workspace;

// Re-exports from shared crates
pub use tui_keybinds;
pub use tui_theme;
pub use tui_widgets;

// Re-exports
pub use app::{AppHandle, AppId, AppManager, LaunchMode};
pub use compositor::{Compositor, LayoutState};
pub use config::ShellConfig;
pub use error::{ShellError, ShellResult};
pub use ipc::{IpcChannel, IpcMessage};
pub use launcher::{AppLauncher, AppMeta};
pub use notification::{Notification, NotificationLevel, NotificationQueue};
pub use prefix::PrefixKeyHandler;
pub use session::{AppSession, Session, SessionState};
pub use task::{TaskCoordinator, TaskInfo, TaskStatus};
pub use workspace::{Workspace, WorkspaceId, WorkspaceManager};

use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};

/// Shell variant types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellVariant {
    /// Tiled layout (i3/sway style).
    #[default]
    Tiled,
    /// Floating windows.
    Floating,
    /// Tabbed layout.
    Tabbed,
    /// Fullscreen single app.
    Fullscreen,
}

impl std::fmt::Display for ShellVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tiled => write!(f, "tiled"),
            Self::Floating => write!(f, "floating"),
            Self::Tabbed => write!(f, "tabbed"),
            Self::Fullscreen => write!(f, "fullscreen"),
        }
    }
}

impl std::str::FromStr for ShellVariant {
    type Err = ShellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tiled" => Ok(Self::Tiled),
            "floating" => Ok(Self::Floating),
            "tabbed" => Ok(Self::Tabbed),
            "fullscreen" => Ok(Self::Fullscreen),
            _ => Err(ShellError::Config(format!("Unknown variant: {}", s))),
        }
    }
}

/// The main shell instance.
pub struct Shell {
    /// Shell configuration.
    config: ShellConfig,
    /// App manager.
    apps: AppManager,
    /// Notification queue.
    notifications: NotificationQueue,
    /// Workspace manager.
    workspaces: WorkspaceManager,
    /// Task coordinator.
    tasks: TaskCoordinator,
    /// App launcher.
    launcher: AppLauncher,
    /// Prefix key handler.
    prefix: PrefixKeyHandler,
    /// Compositor for rendering.
    compositor: Compositor,
    /// Current terminal size.
    size: Rect,
    /// Whether shell is running.
    running: bool,
}

impl Shell {
    /// Create a new shell with the given configuration.
    pub fn new(config: ShellConfig) -> Self {
        let mut prefix = PrefixKeyHandler::new();
        prefix.set_prefix(&config.prefix_key);

        Self {
            config: config.clone(),
            apps: AppManager::new(),
            notifications: NotificationQueue::new(config.notifications.clone()),
            workspaces: WorkspaceManager::new(),
            tasks: TaskCoordinator::default(),
            launcher: AppLauncher::default(),
            prefix,
            compositor: Compositor::new(80, 24),
            size: Rect::default(),
            running: false,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ShellConfig::default())
    }

    /// Get the shell configuration.
    pub fn config(&self) -> &ShellConfig {
        &self.config
    }

    /// Get the app manager.
    pub fn apps(&self) -> &AppManager {
        &self.apps
    }

    /// Get mutable app manager.
    pub fn apps_mut(&mut self) -> &mut AppManager {
        &mut self.apps
    }

    /// Get the notification queue.
    pub fn notifications(&self) -> &NotificationQueue {
        &self.notifications
    }

    /// Get mutable notification queue.
    pub fn notifications_mut(&mut self) -> &mut NotificationQueue {
        &mut self.notifications
    }

    /// Get the workspace manager.
    pub fn workspaces(&self) -> &WorkspaceManager {
        &self.workspaces
    }

    /// Get mutable workspace manager.
    pub fn workspaces_mut(&mut self) -> &mut WorkspaceManager {
        &mut self.workspaces
    }

    /// Get the task coordinator.
    pub fn tasks(&self) -> &TaskCoordinator {
        &self.tasks
    }

    /// Get mutable task coordinator.
    pub fn tasks_mut(&mut self) -> &mut TaskCoordinator {
        &mut self.tasks
    }

    /// Get the app launcher.
    pub fn launcher(&self) -> &AppLauncher {
        &self.launcher
    }

    /// Get mutable app launcher.
    pub fn launcher_mut(&mut self) -> &mut AppLauncher {
        &mut self.launcher
    }

    /// Get the prefix key handler.
    pub fn prefix(&self) -> &PrefixKeyHandler {
        &self.prefix
    }

    /// Get mutable prefix key handler.
    pub fn prefix_mut(&mut self) -> &mut PrefixKeyHandler {
        &mut self.prefix
    }

    /// Get the compositor.
    pub fn compositor(&self) -> &Compositor {
        &self.compositor
    }

    /// Get mutable compositor.
    pub fn compositor_mut(&mut self) -> &mut Compositor {
        &mut self.compositor
    }

    /// Set the terminal size.
    pub fn set_size(&mut self, width: u16, height: u16) {
        self.size = Rect::new(0, 0, width, height);
        self.compositor.resize(width, height);
    }

    /// Check if shell is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Start the shell.
    pub fn start(&mut self) -> ShellResult<()> {
        self.running = true;

        // Restore session if configured
        if self.config.session.restore_on_start {
            if let Err(e) = self.restore_session() {
                // Log but don't fail
                eprintln!("Failed to restore session: {}", e);
            }
        }

        Ok(())
    }

    /// Stop the shell.
    pub fn stop(&mut self) -> ShellResult<()> {
        // Save session if configured
        if self.config.session.auto_save {
            if let Err(e) = self.save_session() {
                eprintln!("Failed to save session: {}", e);
            }
        }

        // Kill all apps
        self.apps.shutdown_all();

        self.running = false;
        Ok(())
    }

    /// Save the current session.
    pub fn save_session(&self) -> ShellResult<Session> {
        let session = Session {
            layout: self.compositor.state().clone(),
            apps: Vec::new(), // Apps save their own state
            focused: self.compositor.focused(),
            workspaces: self.workspaces.to_vec(),
        };

        // Write to file
        let path = self.session_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&session)?;
        std::fs::write(&path, json)?;

        Ok(session)
    }

    /// Restore a saved session.
    pub fn restore_session(&mut self) -> ShellResult<()> {
        let path = self.session_path();
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&content)?;

        // Restore workspaces
        self.workspaces = WorkspaceManager::from_workspaces(session.workspaces);

        // Restore layout
        self.compositor.restore_state(session.layout);

        // Restore focus
        if let Some(focused) = session.focused {
            self.compositor.focus(focused);
        }

        Ok(())
    }

    /// Get the session file path.
    fn session_path(&self) -> std::path::PathBuf {
        directories::ProjectDirs::from("", "", "tui-shell")
            .map(|d| d.data_dir().join("session.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("session.json"))
    }

    /// Push a notification.
    pub fn notify(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    /// Get current screen size.
    pub fn size(&self) -> Rect {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_variant() {
        assert_eq!(ShellVariant::Tiled.to_string(), "tiled");
        assert_eq!("floating".parse::<ShellVariant>().unwrap(), ShellVariant::Floating);
    }

    #[test]
    fn test_shell_creation() {
        let shell = Shell::with_defaults();
        assert!(!shell.is_running());
    }
}
