//! TreeView widget - expandable hierarchical view with lazy loading.
//!
//! # Example
//!
//! ```ignore
//! use tui_widgets::{TreeView, TreeNode, TreeChildren};
//!
//! struct FileNode {
//!     name: String,
//!     is_dir: bool,
//!     children: Vec<FileNode>,
//! }
//!
//! impl TreeNode for FileNode {
//!     fn id(&self) -> &str {
//!         &self.name
//!     }
//!
//!     fn label(&self) -> &str {
//!         &self.name
//!     }
//!
//!     fn children(&self) -> TreeChildren {
//!         TreeChildren::Loaded(
//!             self.children.iter()
//!                 .map(|c| Box::new(c.clone()) as Box<dyn TreeNode>)
//!                 .collect()
//!         )
//!     }
//!
//!     fn is_expandable(&self) -> bool {
//!         self.is_dir
//!     }
//!
//!     fn icon(&self) -> Option<&str> {
//!         if self.is_dir {
//!             Some("\u{f07b}") // Folder icon
//!         } else {
//!             Some("\u{f15b}") // File icon
//!         }
//!     }
//! }
//! ```

mod state;

pub use state::TreeState;

use crate::accessibility::{Accessible, SoundCue};
use crate::WidgetConfig;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, StatefulWidget, Widget};

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

/// Children of a tree node.
pub enum TreeChildren {
    /// Children already loaded
    Loaded(Vec<Box<dyn TreeNode>>),
    /// Lazy-loaded children
    Lazy(Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Vec<Box<dyn TreeNode>>, TreeError>> + Send>> + Send + Sync>),
}

impl std::fmt::Debug for TreeChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loaded(children) => f.debug_tuple("Loaded").field(&children.len()).finish(),
            Self::Lazy(_) => f.debug_tuple("Lazy").finish(),
        }
    }
}

/// Error type for tree operations.
#[derive(Debug, Clone)]
pub struct TreeError {
    pub message: String,
}

impl std::fmt::Display for TreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TreeError {}

/// Trait for tree node data.
pub trait TreeNode: Send + Sync {
    /// Unique identifier for this node.
    fn id(&self) -> &str;

    /// Display label for this node.
    fn label(&self) -> &str;

    /// Get children of this node.
    fn children(&self) -> TreeChildren;

    /// Whether this node can be expanded (has or may have children).
    fn is_expandable(&self) -> bool;

    /// Optional Nerd Font icon for this node.
    fn icon(&self) -> Option<&str> {
        None
    }
}

/// Action to take when lazy loading fails.
#[derive(Debug, Clone)]
pub enum LoadErrorAction {
    /// Show inline error with optional retry button
    InlineError { message: String, retry: bool },
    /// Show toast notification
    Toast { message: String },
    /// Show empty placeholder
    EmptyPlaceholder { message: String },
}

impl Default for LoadErrorAction {
    fn default() -> Self {
        Self::InlineError {
            message: "Failed to load".into(),
            retry: true,
        }
    }
}

/// Expandable hierarchical tree view.
pub struct TreeView<T: TreeNode + 'static> {
    /// Root node
    root: T,
    /// Widget configuration
    config: WidgetConfig,
    /// Block wrapper
    block: Option<Block<'static>>,
    /// Error handler
    on_load_error: Box<dyn Fn(TreeError) -> LoadErrorAction + Send + Sync>,
    /// Callbacks
    on_select: Option<Box<dyn Fn(&T) + Send + Sync>>,
    on_expand: Option<Box<dyn Fn(&T) + Send + Sync>>,
    on_collapse: Option<Box<dyn Fn(&T) + Send + Sync>>,
    /// Maximum indent depth before capping
    max_indent: usize,
}

impl<T: TreeNode + 'static> TreeView<T> {
    /// Create a new tree view with a root node.
    pub fn new(root: T) -> Self {
        Self {
            root,
            config: WidgetConfig::default(),
            block: None,
            on_load_error: Box::new(|_| LoadErrorAction::default()),
            on_select: None,
            on_expand: None,
            on_collapse: None,
            max_indent: 8,
        }
    }

    /// Set the block wrapper.
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set error handler.
    pub fn on_load_error(
        mut self,
        handler: impl Fn(TreeError) -> LoadErrorAction + Send + Sync + 'static,
    ) -> Self {
        self.on_load_error = Box::new(handler);
        self
    }

    /// Set selection callback.
    pub fn on_select(mut self, f: impl Fn(&T) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Set expand callback.
    pub fn on_expand(mut self, f: impl Fn(&T) + Send + Sync + 'static) -> Self {
        self.on_expand = Some(Box::new(f));
        self
    }

    /// Set collapse callback.
    pub fn on_collapse(mut self, f: impl Fn(&T) + Send + Sync + 'static) -> Self {
        self.on_collapse = Some(Box::new(f));
        self
    }

    /// Set maximum indent depth.
    pub fn max_indent(mut self, depth: usize) -> Self {
        self.max_indent = depth;
        self
    }

    /// Get the root node.
    pub fn root(&self) -> &T {
        &self.root
    }

    /// Handle a key event.
    pub fn handle_key(&self, key: KeyEvent, state: &mut TreeState) -> bool {
        if self.config.disabled {
            return false;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.move_selection(-1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.move_selection(1);
                true
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(ref selected) = state.selected {
                    if state.expanded.contains(selected) {
                        state.expanded.remove(selected);
                        return true;
                    }
                }
                // Move to parent
                state.move_to_parent();
                true
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                if let Some(ref selected) = state.selected {
                    if !state.expanded.contains(selected) {
                        state.expanded.insert(selected.clone());
                        return true;
                    }
                }
                // Enter/expand already expanded node - could select or expand children
                true
            }
            KeyCode::Char(' ') => {
                // Toggle expand/collapse
                if let Some(ref selected) = state.selected {
                    if state.expanded.contains(selected) {
                        state.expanded.remove(selected);
                    } else {
                        state.expanded.insert(selected.clone());
                    }
                }
                true
            }
            KeyCode::Char('/') => {
                // Enter search mode
                state.search_query = Some(String::new());
                true
            }
            KeyCode::Esc => {
                state.search_query = None;
                true
            }
            _ => false,
        }
    }

    /// Collect visible nodes for rendering.
    fn collect_visible_nodes(&self, state: &TreeState) -> Vec<VisibleNode> {
        let mut nodes = Vec::new();
        self.collect_node(&self.root, 0, state, &mut nodes);
        nodes
    }

    fn collect_node(&self, node: &dyn TreeNode, depth: usize, state: &TreeState, nodes: &mut Vec<VisibleNode>) {
        let id = node.id().to_string();
        let is_expanded = state.expanded.contains(&id);
        let matches_search = state.search_query.as_ref().map_or(true, |q| {
            node.label().to_lowercase().contains(&q.to_lowercase())
        });

        if matches_search || is_expanded {
            nodes.push(VisibleNode {
                id: id.clone(),
                label: node.label().to_string(),
                icon: node.icon().map(|s| s.to_string()),
                depth,
                is_expandable: node.is_expandable(),
                is_expanded,
            });
        }

        if is_expanded {
            if let TreeChildren::Loaded(children) = node.children() {
                for child in children {
                    self.collect_node(child.as_ref(), depth + 1, state, nodes);
                }
            }
        }
    }
}

/// A visible node for rendering.
struct VisibleNode {
    id: String,
    label: String,
    icon: Option<String>,
    depth: usize,
    is_expandable: bool,
    is_expanded: bool,
}

impl<T: TreeNode + 'static> StatefulWidget for TreeView<T> {
    type State = TreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render block if present
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width < 3 || inner.height < 1 {
            return;
        }

        let nodes = self.collect_visible_nodes(state);

        if nodes.is_empty() {
            let msg = "No items";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
            return;
        }

        // Update visible node list for navigation
        state.visible_nodes = nodes.iter().map(|n| n.id.clone()).collect();

        let visible_height = inner.height as usize;

        // Adjust scroll
        if let Some(ref selected) = state.selected {
            if let Some(idx) = nodes.iter().position(|n| &n.id == selected) {
                if idx < state.scroll_offset {
                    state.scroll_offset = idx;
                } else if idx >= state.scroll_offset + visible_height {
                    state.scroll_offset = idx.saturating_sub(visible_height - 1);
                }
            }
        }

        for (i, node) in nodes.iter().skip(state.scroll_offset).take(visible_height).enumerate() {
            let y = inner.y + i as u16;
            let is_selected = state.selected.as_ref() == Some(&node.id);

            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            // Clear line
            for x in inner.x..inner.x + inner.width {
                buf[(x, y)].set_style(style);
            }

            // Indent (capped at max_indent)
            let indent = node.depth.min(self.max_indent);
            let indent_str = "  ".repeat(indent);

            // Depth indicator for deep nodes
            let depth_indicator = if node.depth > self.max_indent {
                format!("[{}] ", node.depth)
            } else {
                String::new()
            };

            // Expand/collapse indicator
            let expand_char = if node.is_expandable {
                if node.is_expanded { "\u{25bc} " } else { "\u{25b6} " } // Down/Right triangles
            } else {
                "  "
            };

            // Icon
            let icon = node.icon.as_deref().map_or("", |i| i);
            let icon_space = if icon.is_empty() { "" } else { " " };

            let line = format!(
                "{}{}{}{}{}{}",
                indent_str, depth_indicator, expand_char, icon, icon_space, node.label
            );

            // Truncate if needed
            let max_len = inner.width as usize;
            let display = if line.len() > max_len {
                format!("{}...", &line[..max_len.saturating_sub(3)])
            } else {
                line
            };

            buf.set_string(inner.x, y, &display, style);
        }

        // Scrollbar
        if nodes.len() > visible_height {
            let scrollbar_height = (visible_height as f32 / nodes.len() as f32 * inner.height as f32)
                .max(1.0) as u16;
            let scrollbar_pos = (state.scroll_offset as f32 / (nodes.len() - visible_height) as f32
                * (inner.height - scrollbar_height) as f32) as u16;

            for y_off in 0..inner.height {
                let y = inner.y + y_off;
                let ch = if y_off >= scrollbar_pos && y_off < scrollbar_pos + scrollbar_height {
                    '\u{2588}'
                } else {
                    '\u{2591}'
                };
                buf[(inner.x + inner.width - 1, y)].set_char(ch);
            }
        }
    }
}

impl<T: TreeNode + 'static> Accessible for TreeView<T> {
    fn aria_role(&self) -> &str {
        "tree"
    }

    fn aria_label(&self) -> String {
        format!("Tree view rooted at {}", self.root.label())
    }

    fn announce(&self, _message: &str) {
        // Would integrate with announcement buffer
    }

    fn play_sound(&self, _sound: SoundCue) {
        // Would integrate with sound system
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestNode {
        id: String,
        label: String,
        children: Vec<TestNode>,
    }

    impl TreeNode for TestNode {
        fn id(&self) -> &str {
            &self.id
        }

        fn label(&self) -> &str {
            &self.label
        }

        fn children(&self) -> TreeChildren {
            TreeChildren::Loaded(
                self.children
                    .iter()
                    .map(|c| {
                        Box::new(TestNode {
                            id: c.id.clone(),
                            label: c.label.clone(),
                            children: c.children.clone(),
                        }) as Box<dyn TreeNode>
                    })
                    .collect(),
            )
        }

        fn is_expandable(&self) -> bool {
            !self.children.is_empty()
        }
    }

    impl Clone for TestNode {
        fn clone(&self) -> Self {
            TestNode {
                id: self.id.clone(),
                label: self.label.clone(),
                children: self.children.clone(),
            }
        }
    }

    #[test]
    fn test_tree_creation() {
        let root = TestNode {
            id: "root".into(),
            label: "Root".into(),
            children: vec![
                TestNode {
                    id: "child1".into(),
                    label: "Child 1".into(),
                    children: vec![],
                },
                TestNode {
                    id: "child2".into(),
                    label: "Child 2".into(),
                    children: vec![],
                },
            ],
        };

        let tree = TreeView::new(root);
        assert_eq!(tree.root().label(), "Root");
    }

    #[test]
    fn test_tree_state() {
        let mut state = TreeState::new();
        assert!(state.selected.is_none());

        state.selected = Some("node1".into());
        assert_eq!(state.selected, Some("node1".into()));

        state.expanded.insert("node1".into());
        assert!(state.is_expanded("node1"));
    }
}
