//! Workspace management.

use crate::AppId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique workspace identifier.
pub type WorkspaceId = u64;

/// A workspace containing apps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique ID.
    pub id: WorkspaceId,
    /// Workspace name.
    pub name: String,
    /// Apps in this workspace.
    pub apps: Vec<AppId>,
    /// Whether this is the active workspace.
    #[serde(default)]
    pub active: bool,
}

impl Workspace {
    /// Create a new workspace.
    pub fn new(id: WorkspaceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            apps: Vec::new(),
            active: false,
        }
    }

    /// Add an app to the workspace.
    pub fn add_app(&mut self, app_id: AppId) {
        if !self.apps.contains(&app_id) {
            self.apps.push(app_id);
        }
    }

    /// Remove an app from the workspace.
    pub fn remove_app(&mut self, app_id: AppId) {
        self.apps.retain(|&id| id != app_id);
    }

    /// Check if workspace contains an app.
    pub fn contains(&self, app_id: AppId) -> bool {
        self.apps.contains(&app_id)
    }

    /// Get app count.
    pub fn app_count(&self) -> usize {
        self.apps.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }
}

/// Manages workspaces.
pub struct WorkspaceManager {
    /// All workspaces.
    workspaces: HashMap<WorkspaceId, Workspace>,
    /// Active workspace.
    active: Option<WorkspaceId>,
    /// Next workspace ID.
    next_id: WorkspaceId,
    /// Workspace order.
    order: Vec<WorkspaceId>,
}

impl WorkspaceManager {
    /// Create a new workspace manager.
    pub fn new() -> Self {
        let mut manager = Self {
            workspaces: HashMap::new(),
            active: None,
            next_id: 1,
            order: Vec::new(),
        };

        // Create default workspace
        manager.create("default");

        manager
    }

    /// Create from existing workspaces.
    pub fn from_workspaces(workspaces: Vec<Workspace>) -> Self {
        let mut manager = Self {
            workspaces: HashMap::new(),
            active: None,
            next_id: 1,
            order: Vec::new(),
        };

        for ws in workspaces {
            if ws.id >= manager.next_id {
                manager.next_id = ws.id + 1;
            }
            if ws.active {
                manager.active = Some(ws.id);
            }
            manager.order.push(ws.id);
            manager.workspaces.insert(ws.id, ws);
        }

        if manager.active.is_none() && !manager.workspaces.is_empty() {
            manager.active = manager.order.first().copied();
        }

        manager
    }

    /// Create a new workspace.
    pub fn create(&mut self, name: impl Into<String>) -> WorkspaceId {
        let id = self.next_id;
        self.next_id += 1;

        let workspace = Workspace::new(id, name);
        self.workspaces.insert(id, workspace);
        self.order.push(id);

        if self.active.is_none() {
            self.active = Some(id);
        }

        id
    }

    /// Delete a workspace.
    pub fn delete(&mut self, id: WorkspaceId) -> Option<Workspace> {
        let workspace = self.workspaces.remove(&id)?;
        self.order.retain(|&ws| ws != id);

        if self.active == Some(id) {
            self.active = self.order.first().copied();
        }

        Some(workspace)
    }

    /// Get a workspace.
    pub fn get(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.get(&id)
    }

    /// Get mutable workspace.
    pub fn get_mut(&mut self, id: WorkspaceId) -> Option<&mut Workspace> {
        self.workspaces.get_mut(&id)
    }

    /// Get workspace by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.values().find(|ws| ws.name == name)
    }

    /// Get active workspace.
    pub fn active(&self) -> Option<&Workspace> {
        self.active.and_then(|id| self.workspaces.get(&id))
    }

    /// Get active workspace ID.
    pub fn active_id(&self) -> Option<WorkspaceId> {
        self.active
    }

    /// Switch to a workspace.
    pub fn switch_to(&mut self, id: WorkspaceId) -> bool {
        if self.workspaces.contains_key(&id) {
            // Mark old workspace as inactive
            if let Some(old_id) = self.active {
                if let Some(old_ws) = self.workspaces.get_mut(&old_id) {
                    old_ws.active = false;
                }
            }

            // Mark new workspace as active
            if let Some(ws) = self.workspaces.get_mut(&id) {
                ws.active = true;
            }

            self.active = Some(id);
            true
        } else {
            false
        }
    }

    /// Switch to next workspace.
    pub fn switch_next(&mut self) -> Option<WorkspaceId> {
        if self.order.is_empty() {
            return None;
        }

        let current_idx = self
            .active
            .and_then(|id| self.order.iter().position(|&ws| ws == id))
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % self.order.len();
        let next_id = self.order[next_idx];
        self.switch_to(next_id);
        Some(next_id)
    }

    /// Switch to previous workspace.
    pub fn prev(&mut self) -> Option<WorkspaceId> {
        if self.order.is_empty() {
            return None;
        }

        let current_idx = self
            .active
            .and_then(|id| self.order.iter().position(|&ws| ws == id))
            .unwrap_or(0);

        let prev_idx = if current_idx == 0 {
            self.order.len() - 1
        } else {
            current_idx - 1
        };

        let prev_id = self.order[prev_idx];
        self.switch_to(prev_id);
        Some(prev_id)
    }

    /// List all workspaces.
    pub fn list(&self) -> Vec<&Workspace> {
        self.order
            .iter()
            .filter_map(|id| self.workspaces.get(id))
            .collect()
    }

    /// Get workspace count.
    pub fn count(&self) -> usize {
        self.workspaces.len()
    }

    /// Rename a workspace.
    pub fn rename(&mut self, id: WorkspaceId, name: impl Into<String>) -> bool {
        if let Some(ws) = self.workspaces.get_mut(&id) {
            ws.name = name.into();
            true
        } else {
            false
        }
    }

    /// Add app to workspace.
    pub fn add_app(&mut self, workspace_id: WorkspaceId, app_id: AppId) {
        if let Some(ws) = self.workspaces.get_mut(&workspace_id) {
            ws.add_app(app_id);
        }
    }

    /// Remove app from workspace.
    pub fn remove_app(&mut self, workspace_id: WorkspaceId, app_id: AppId) {
        if let Some(ws) = self.workspaces.get_mut(&workspace_id) {
            ws.remove_app(app_id);
        }
    }

    /// Remove app from all workspaces.
    pub fn remove_app_everywhere(&mut self, app_id: AppId) {
        for ws in self.workspaces.values_mut() {
            ws.remove_app(app_id);
        }
    }

    /// Get workspaces containing an app.
    pub fn workspaces_for_app(&self, app_id: AppId) -> Vec<WorkspaceId> {
        self.workspaces
            .iter()
            .filter(|(_, ws)| ws.contains(app_id))
            .map(|(&id, _)| id)
            .collect()
    }

    /// Convert to vec for serialization.
    pub fn to_vec(&self) -> Vec<Workspace> {
        self.order
            .iter()
            .filter_map(|id| self.workspaces.get(id).cloned())
            .collect()
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let ws = Workspace::new(1, "test");
        assert_eq!(ws.name, "test");
        assert!(ws.is_empty());
    }

    #[test]
    fn test_workspace_apps() {
        let mut ws = Workspace::new(1, "test");
        ws.add_app(100);
        ws.add_app(200);

        assert_eq!(ws.app_count(), 2);
        assert!(ws.contains(100));

        ws.remove_app(100);
        assert!(!ws.contains(100));
    }

    #[test]
    fn test_workspace_manager() {
        let mut manager = WorkspaceManager::new();
        assert_eq!(manager.count(), 1); // Default workspace

        let ws2 = manager.create("workspace2");
        assert_eq!(manager.count(), 2);

        manager.switch_to(ws2);
        assert_eq!(manager.active_id(), Some(ws2));
    }

    #[test]
    fn test_workspace_navigation() {
        let mut manager = WorkspaceManager::new();
        let ws2 = manager.create("ws2");
        let ws3 = manager.create("ws3");

        manager.switch_to(ws2);
        manager.switch_next();
        assert_eq!(manager.active_id(), Some(ws3));

        manager.prev();
        assert_eq!(manager.active_id(), Some(ws2));
    }
}
