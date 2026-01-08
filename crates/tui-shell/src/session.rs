//! Session state persistence.

use crate::compositor::LayoutState;
use crate::workspace::{Workspace, WorkspaceId};
use crate::AppId;
use serde::{Deserialize, Serialize};

/// Trait for apps that support session state.
pub trait SessionState {
    /// Save current state.
    fn save_state(&self) -> Result<serde_json::Value, crate::ShellError>;

    /// Restore from saved state.
    fn restore_state(&mut self, state: serde_json::Value) -> Result<(), crate::ShellError>;
}

/// Complete session snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    /// Layout state.
    pub layout: LayoutState,
    /// App sessions.
    pub apps: Vec<AppSession>,
    /// Focused app ID.
    pub focused: Option<AppId>,
    /// Workspaces.
    pub workspaces: Vec<Workspace>,
}

/// Session data for a single app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSession {
    /// App name.
    pub app_name: String,
    /// Launch arguments.
    pub args: Vec<String>,
    /// Saved state (if app supports it).
    pub state: Option<serde_json::Value>,
    /// Workspace memberships.
    pub workspace_memberships: Vec<WorkspaceId>,
}

impl AppSession {
    /// Create a new app session.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            args: Vec::new(),
            state: None,
            workspace_memberships: Vec::new(),
        }
    }

    /// Set arguments.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set state.
    pub fn with_state(mut self, state: serde_json::Value) -> Self {
        self.state = Some(state);
        self
    }

    /// Add workspace membership.
    pub fn in_workspace(mut self, workspace: WorkspaceId) -> Self {
        self.workspace_memberships.push(workspace);
        self
    }
}

/// Session manager for auto-save and restore.
pub struct SessionManager {
    /// Current session.
    session: Session,
    /// Session file path.
    path: std::path::PathBuf,
    /// Last save time.
    last_save: Option<std::time::Instant>,
    /// Auto-save interval.
    save_interval: std::time::Duration,
    /// Whether dirty (unsaved changes).
    dirty: bool,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(path: std::path::PathBuf, save_interval_secs: u64) -> Self {
        Self {
            session: Session::default(),
            path,
            last_save: None,
            save_interval: std::time::Duration::from_secs(save_interval_secs),
            dirty: false,
        }
    }

    /// Load session from disk.
    pub fn load(&mut self) -> Result<(), crate::ShellError> {
        if self.path.exists() {
            let content = std::fs::read_to_string(&self.path)?;
            self.session = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// Save session to disk.
    pub fn save(&mut self) -> Result<(), crate::ShellError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.session)?;
        std::fs::write(&self.path, content)?;
        self.last_save = Some(std::time::Instant::now());
        self.dirty = false;
        Ok(())
    }

    /// Update session data.
    pub fn update(&mut self, session: Session) {
        self.session = session;
        self.dirty = true;
    }

    /// Get current session.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Check if auto-save is needed.
    pub fn needs_save(&self) -> bool {
        if !self.dirty {
            return false;
        }

        match self.last_save {
            Some(last) => last.elapsed() >= self.save_interval,
            None => true,
        }
    }

    /// Try auto-save if needed.
    pub fn try_auto_save(&mut self) -> Result<bool, crate::ShellError> {
        if self.needs_save() {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Mark as dirty.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_session() {
        let session = AppSession::new("test-app")
            .with_args(vec!["--flag".to_string()])
            .in_workspace(1);

        assert_eq!(session.app_name, "test-app");
        assert_eq!(session.args.len(), 1);
        assert_eq!(session.workspace_memberships.len(), 1);
    }

    #[test]
    fn test_session_default() {
        let session = Session::default();
        assert!(session.apps.is_empty());
        assert!(session.focused.is_none());
    }

    #[test]
    fn test_session_manager() {
        let path = std::path::PathBuf::from("/tmp/test_session.json");
        let mut manager = SessionManager::new(path, 300);

        assert!(!manager.is_dirty());
        manager.mark_dirty();
        assert!(manager.is_dirty());
    }
}
