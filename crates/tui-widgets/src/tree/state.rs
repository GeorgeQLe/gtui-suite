//! State management for TreeView.

use std::collections::HashSet;

/// State for TreeView widget.
#[derive(Debug, Clone, Default)]
pub struct TreeState {
    /// Set of expanded node IDs
    pub expanded: HashSet<String>,
    /// Currently selected node ID
    pub selected: Option<String>,
    /// Current search query
    pub search_query: Option<String>,
    /// Whether breadcrumb mode is active
    pub breadcrumb_mode: bool,
    /// Focus path for breadcrumb navigation
    pub focus_path: Vec<String>,
    /// Scroll offset
    pub scroll_offset: usize,
    /// List of visible node IDs (for navigation)
    pub(crate) visible_nodes: Vec<String>,
}

impl TreeState {
    /// Create a new empty tree state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a node is expanded.
    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded.contains(id)
    }

    /// Expand a node.
    pub fn expand(&mut self, id: impl Into<String>) {
        self.expanded.insert(id.into());
    }

    /// Collapse a node.
    pub fn collapse(&mut self, id: &str) {
        self.expanded.remove(id);
    }

    /// Toggle expansion of a node.
    pub fn toggle_expand(&mut self, id: &str) {
        if self.expanded.contains(id) {
            self.expanded.remove(id);
        } else {
            self.expanded.insert(id.to_string());
        }
    }

    /// Expand all ancestors of a node to make it visible.
    pub fn expand_to(&mut self, path: &[String]) {
        for id in path {
            self.expanded.insert(id.clone());
        }
    }

    /// Collapse all nodes.
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    /// Select a node.
    pub fn select(&mut self, id: impl Into<String>) {
        self.selected = Some(id.into());
    }

    /// Clear selection.
    pub fn deselect(&mut self) {
        self.selected = None;
    }

    /// Move selection by delta.
    pub fn move_selection(&mut self, delta: isize) {
        if self.visible_nodes.is_empty() {
            return;
        }

        let current_idx = self
            .selected
            .as_ref()
            .and_then(|s| self.visible_nodes.iter().position(|n| n == s))
            .unwrap_or(0);

        let new_idx = if delta < 0 {
            current_idx.saturating_sub((-delta) as usize)
        } else {
            (current_idx + delta as usize).min(self.visible_nodes.len() - 1)
        };

        self.selected = Some(self.visible_nodes[new_idx].clone());
    }

    /// Move to parent node.
    pub fn move_to_parent(&mut self) {
        if let Some(ref selected) = self.selected {
            if let Some(idx) = self.visible_nodes.iter().position(|n| n == selected) {
                // Find parent by looking for node with lower depth
                // In a flat list, we'd need depth info, but for now just move up
                if idx > 0 {
                    self.selected = Some(self.visible_nodes[idx - 1].clone());
                }
            }
        }
    }

    /// Enter breadcrumb mode at current selection.
    pub fn enter_breadcrumb_mode(&mut self) {
        self.breadcrumb_mode = true;
        if let Some(ref selected) = self.selected {
            self.focus_path.push(selected.clone());
        }
    }

    /// Exit breadcrumb mode.
    pub fn exit_breadcrumb_mode(&mut self) {
        self.breadcrumb_mode = false;
        self.focus_path.clear();
    }

    /// Navigate up in breadcrumb mode.
    pub fn breadcrumb_up(&mut self) {
        if !self.focus_path.is_empty() {
            self.focus_path.pop();
            self.selected = self.focus_path.last().cloned();
        }
    }

    /// Set search query.
    pub fn set_search(&mut self, query: impl Into<String>) {
        self.search_query = Some(query.into());
    }

    /// Clear search query.
    pub fn clear_search(&mut self) {
        self.search_query = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_expand_collapse() {
        let mut state = TreeState::new();

        state.expand("node1");
        assert!(state.is_expanded("node1"));

        state.collapse("node1");
        assert!(!state.is_expanded("node1"));

        state.toggle_expand("node2");
        assert!(state.is_expanded("node2"));

        state.toggle_expand("node2");
        assert!(!state.is_expanded("node2"));
    }

    #[test]
    fn test_state_selection() {
        let mut state = TreeState::new();

        state.select("node1");
        assert_eq!(state.selected, Some("node1".into()));

        state.deselect();
        assert_eq!(state.selected, None);
    }

    #[test]
    fn test_state_navigation() {
        let mut state = TreeState::new();
        state.visible_nodes = vec!["a".into(), "b".into(), "c".into(), "d".into()];

        state.select("b");
        assert_eq!(state.selected, Some("b".into()));

        state.move_selection(1);
        assert_eq!(state.selected, Some("c".into()));

        state.move_selection(-1);
        assert_eq!(state.selected, Some("b".into()));

        state.move_selection(-10);
        assert_eq!(state.selected, Some("a".into()));

        state.move_selection(10);
        assert_eq!(state.selected, Some("d".into()));
    }

    #[test]
    fn test_state_breadcrumb() {
        let mut state = TreeState::new();
        state.select("child");
        state.enter_breadcrumb_mode();

        assert!(state.breadcrumb_mode);
        assert_eq!(state.focus_path, vec!["child".to_string()]);

        state.exit_breadcrumb_mode();
        assert!(!state.breadcrumb_mode);
        assert!(state.focus_path.is_empty());
    }
}
