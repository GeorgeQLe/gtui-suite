//! App lifecycle management.

use crate::error::{ShellError, ShellResult};
use crate::ipc::IpcChannel;
use crate::session::AppSession;
use crate::workspace::WorkspaceId;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::process::Child;

/// Unique identifier for an app instance.
pub type AppId = u64;

/// App manifest describing an app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    /// App name.
    pub name: String,
    /// Display name.
    pub display_name: String,
    /// Description.
    pub description: Option<String>,
    /// Preferred launch mode.
    pub preferred_launch: PreferredLaunch,
    /// Whether the app supports session state.
    pub supports_session: bool,
    /// Whether to auto-restart on crash.
    pub auto_restart: bool,
    /// Initial restart backoff in milliseconds.
    pub restart_backoff_ms: u64,
}

impl Default for AppManifest {
    fn default() -> Self {
        Self {
            name: String::new(),
            display_name: String::new(),
            description: None,
            preferred_launch: PreferredLaunch::Subprocess,
            supports_session: false,
            auto_restart: false,
            restart_backoff_ms: 1000,
        }
    }
}

/// Preferred launch mode for an app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferredLaunch {
    /// Load as in-process plugin.
    InProcess,
    /// Spawn as subprocess.
    #[default]
    Subprocess,
    /// Shell decides based on conditions.
    Either,
}

/// App launch mode.
pub enum LaunchMode {
    /// In-process plugin.
    InProcess {
        /// Plugin trait object.
        plugin: Box<dyn AppPlugin>,
    },
    /// Subprocess.
    Subprocess {
        /// Child process.
        process: Child,
    },
}

/// Trait for in-process app plugins.
pub trait AppPlugin: Send {
    /// Get the app name.
    fn name(&self) -> &str;

    /// Initialize the app.
    fn init(&mut self) -> ShellResult<()>;

    /// Render to a buffer.
    fn render(&self, buf: &mut Buffer, area: Rect);

    /// Handle input.
    fn handle_input(&mut self, event: crossterm::event::Event) -> bool;

    /// Tick for updates.
    fn tick(&mut self);

    /// Shutdown the app.
    fn shutdown(&mut self) -> ShellResult<()>;

    /// Save session state.
    fn save_state(&self) -> Option<serde_json::Value> {
        None
    }

    /// Restore session state.
    fn restore_state(&mut self, _state: serde_json::Value) -> ShellResult<()> {
        Ok(())
    }
}

/// Handle to a running app.
pub struct AppHandle {
    /// Unique ID.
    pub id: AppId,
    /// App manifest.
    pub manifest: AppManifest,
    /// Launch mode.
    pub launch_mode: LaunchMode,
    /// Render buffer.
    pub buffer: AppBuffer,
    /// IPC channel (for subprocess apps).
    pub ipc: Option<IpcChannel>,
    /// Workspace memberships.
    pub workspaces: HashSet<WorkspaceId>,
    /// Whether this app is "sticky" (visible in all workspaces).
    pub sticky: bool,
    /// Current restart count.
    pub restart_count: u32,
    /// Last crash time.
    pub last_crash: Option<std::time::Instant>,
}

/// Sandboxed render buffer for an app.
pub struct AppBuffer {
    /// Buffer content.
    buffer: Buffer,
    /// Allocated area.
    area: Rect,
}

impl AppBuffer {
    /// Create a new app buffer.
    pub fn new(area: Rect) -> Self {
        Self {
            buffer: Buffer::empty(area),
            area,
        }
    }

    /// Get the buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get mutable buffer.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Get the area.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Resize the buffer.
    pub fn resize(&mut self, area: Rect) {
        self.area = area;
        self.buffer = Buffer::empty(area);
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.reset();
    }
}

/// Manages app lifecycle.
pub struct AppManager {
    /// Running apps.
    apps: HashMap<AppId, AppHandle>,
    /// Focused app.
    focused: Option<AppId>,
    /// Focus history (most recent first).
    history: Vec<AppId>,
    /// Next app ID.
    next_id: AppId,
    /// App registry.
    registry: HashMap<String, AppManifest>,
}

impl AppManager {
    /// Create a new app manager.
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
            focused: None,
            history: Vec::new(),
            next_id: 1,
            registry: HashMap::new(),
        }
    }

    /// Register an app manifest.
    pub fn register(&mut self, manifest: AppManifest) {
        self.registry.insert(manifest.name.clone(), manifest);
    }

    /// Launch an app.
    pub fn launch(&mut self, app_name: &str, _args: &[&str]) -> ShellResult<AppId> {
        let manifest = self.registry.get(app_name).cloned().unwrap_or_else(|| {
            AppManifest {
                name: app_name.to_string(),
                display_name: app_name.to_string(),
                ..Default::default()
            }
        });

        let id = self.next_id;
        self.next_id += 1;

        // For now, create a placeholder handle
        // In a real implementation, this would spawn a subprocess or load a plugin
        let handle = AppHandle {
            id,
            manifest,
            launch_mode: LaunchMode::Subprocess {
                process: std::process::Command::new("true")
                    .spawn()
                    .map_err(|e| ShellError::LaunchFailed(e.to_string()))?,
            },
            buffer: AppBuffer::new(Rect::default()),
            ipc: None,
            workspaces: HashSet::new(),
            sticky: false,
            restart_count: 0,
            last_crash: None,
        };

        self.apps.insert(id, handle);
        self.history.push(id);

        // Auto-focus first app
        if self.focused.is_none() {
            self.focused = Some(id);
        }

        Ok(id)
    }

    /// Suspend an app.
    pub fn suspend(&mut self, id: AppId) -> ShellResult<()> {
        let _handle = self
            .apps
            .get_mut(&id)
            .ok_or_else(|| ShellError::AppNotFound(id.to_string()))?;

        // Send suspend signal via IPC
        Ok(())
    }

    /// Resume an app.
    pub fn resume(&mut self, id: AppId) -> ShellResult<()> {
        let _handle = self
            .apps
            .get_mut(&id)
            .ok_or_else(|| ShellError::AppNotFound(id.to_string()))?;

        // Send resume signal via IPC
        Ok(())
    }

    /// Kill an app.
    pub fn kill(&mut self, id: AppId) -> ShellResult<()> {
        let mut handle = self
            .apps
            .remove(&id)
            .ok_or_else(|| ShellError::AppNotFound(id.to_string()))?;

        // Clean up based on launch mode
        match &mut handle.launch_mode {
            LaunchMode::InProcess { plugin } => {
                plugin.shutdown()?;
            }
            LaunchMode::Subprocess { process } => {
                let _ = process.kill();
            }
        }

        // Update focus
        if self.focused == Some(id) {
            self.focused = self.history.iter().rev().find(|&&h| h != id).copied();
        }

        // Remove from history
        self.history.retain(|&h| h != id);

        Ok(())
    }

    /// Focus an app.
    pub fn focus(&mut self, id: AppId) -> ShellResult<()> {
        if !self.apps.contains_key(&id) {
            return Err(ShellError::AppNotFound(id.to_string()));
        }

        self.focused = Some(id);

        // Move to front of history
        self.history.retain(|&h| h != id);
        self.history.push(id);

        Ok(())
    }

    /// Get the focused app ID.
    pub fn focused(&self) -> Option<AppId> {
        self.focused
    }

    /// Get the focused app handle.
    pub fn focused_app(&self) -> Option<&AppHandle> {
        self.focused.and_then(|id| self.apps.get(&id))
    }

    /// Get mutable focused app handle.
    pub fn focused_app_mut(&mut self) -> Option<&mut AppHandle> {
        self.focused.and_then(|id| self.apps.get_mut(&id))
    }

    /// List running app IDs.
    pub fn list_running(&self) -> Vec<AppId> {
        self.apps.keys().copied().collect()
    }

    /// Get an app handle.
    pub fn get(&self, id: AppId) -> Option<&AppHandle> {
        self.apps.get(&id)
    }

    /// Get mutable app handle.
    pub fn get_mut(&mut self, id: AppId) -> Option<&mut AppHandle> {
        self.apps.get_mut(&id)
    }

    /// Get app count.
    pub fn count(&self) -> usize {
        self.apps.len()
    }

    /// Add app to workspace.
    pub fn add_to_workspace(&mut self, app_id: AppId, workspace_id: WorkspaceId) {
        if let Some(handle) = self.apps.get_mut(&app_id) {
            handle.workspaces.insert(workspace_id);
        }
    }

    /// Remove app from workspace.
    pub fn remove_from_workspace(&mut self, app_id: AppId, workspace_id: WorkspaceId) {
        if let Some(handle) = self.apps.get_mut(&app_id) {
            handle.workspaces.remove(&workspace_id);
        }
    }

    /// Set app as sticky.
    pub fn set_sticky(&mut self, app_id: AppId, sticky: bool) {
        if let Some(handle) = self.apps.get_mut(&app_id) {
            handle.sticky = sticky;
        }
    }

    /// Save all app sessions.
    pub fn save_sessions(&self) -> ShellResult<Vec<AppSession>> {
        let mut sessions = Vec::new();

        for handle in self.apps.values() {
            let state = match &handle.launch_mode {
                LaunchMode::InProcess { plugin } => plugin.save_state(),
                LaunchMode::Subprocess { .. } => None,
            };

            sessions.push(AppSession {
                app_name: handle.manifest.name.clone(),
                args: Vec::new(),
                state,
                workspace_memberships: handle.workspaces.iter().copied().collect(),
            });
        }

        Ok(sessions)
    }

    /// Restore an app from session.
    pub fn restore_app(&mut self, session: &AppSession) -> ShellResult<AppId> {
        let id = self.launch(&session.app_name, &[])?;

        // Restore workspace memberships
        if let Some(handle) = self.apps.get_mut(&id) {
            for ws in &session.workspace_memberships {
                handle.workspaces.insert(*ws);
            }

            // Restore state if supported
            if let Some(state) = &session.state {
                if let LaunchMode::InProcess { plugin } = &mut handle.launch_mode {
                    plugin.restore_state(state.clone())?;
                }
            }
        }

        Ok(id)
    }

    /// Shutdown all apps.
    pub fn shutdown_all(&mut self) -> Vec<ShellResult<()>> {
        let ids: Vec<_> = self.apps.keys().copied().collect();
        ids.into_iter().map(|id| self.kill(id)).collect()
    }

    /// Iterate over apps.
    pub fn iter(&self) -> impl Iterator<Item = &AppHandle> {
        self.apps.values()
    }

    /// Get focus history (most recent first).
    pub fn focus_history(&self) -> &[AppId] {
        &self.history
    }
}

impl Default for AppManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_manager_creation() {
        let manager = AppManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.focused().is_none());
    }

    #[test]
    fn test_app_buffer() {
        let area = Rect::new(0, 0, 10, 5);
        let buffer = AppBuffer::new(area);
        assert_eq!(buffer.area(), area);
    }

    #[test]
    fn test_manifest_default() {
        let manifest = AppManifest::default();
        assert_eq!(manifest.preferred_launch, PreferredLaunch::Subprocess);
        assert!(!manifest.auto_restart);
    }
}
