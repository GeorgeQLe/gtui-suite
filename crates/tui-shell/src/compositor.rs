//! Compositor for rendering apps in the shell.

use crate::AppId;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Layout split direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Layout node in the tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutNode {
    /// Leaf node containing an app.
    Leaf { app_id: AppId },
    /// Split node with children.
    Split {
        direction: SplitDirection,
        /// Ratio (0.0-1.0) for first child.
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

impl LayoutNode {
    /// Create a leaf node.
    pub fn leaf(app_id: AppId) -> Self {
        Self::Leaf { app_id }
    }

    /// Create a horizontal split.
    pub fn hsplit(first: LayoutNode, second: LayoutNode, ratio: f32) -> Self {
        Self::Split {
            direction: SplitDirection::Horizontal,
            ratio: ratio.clamp(0.1, 0.9),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Create a vertical split.
    pub fn vsplit(first: LayoutNode, second: LayoutNode, ratio: f32) -> Self {
        Self::Split {
            direction: SplitDirection::Vertical,
            ratio: ratio.clamp(0.1, 0.9),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Get all app IDs in this layout.
    pub fn app_ids(&self) -> Vec<AppId> {
        match self {
            Self::Leaf { app_id } => vec![*app_id],
            Self::Split { first, second, .. } => {
                let mut ids = first.app_ids();
                ids.extend(second.app_ids());
                ids
            }
        }
    }

    /// Check if layout contains an app.
    pub fn contains(&self, app_id: AppId) -> bool {
        match self {
            Self::Leaf { app_id: id } => *id == app_id,
            Self::Split { first, second, .. } => first.contains(app_id) || second.contains(app_id),
        }
    }

    /// Remove an app from the layout, returning the simplified tree.
    pub fn remove(&self, app_id: AppId) -> Option<LayoutNode> {
        match self {
            Self::Leaf { app_id: id } => {
                if *id == app_id {
                    None
                } else {
                    Some(self.clone())
                }
            }
            Self::Split { direction, ratio, first, second } => {
                let first_removed = first.remove(app_id);
                let second_removed = second.remove(app_id);

                match (first_removed, second_removed) {
                    (None, None) => None,
                    (Some(node), None) | (None, Some(node)) => Some(node),
                    (Some(f), Some(s)) => Some(Self::Split {
                        direction: *direction,
                        ratio: *ratio,
                        first: Box::new(f),
                        second: Box::new(s),
                    }),
                }
            }
        }
    }

    /// Calculate rectangles for all apps.
    pub fn layout(&self, area: Rect) -> HashMap<AppId, Rect> {
        let mut result = HashMap::new();
        self.layout_inner(area, &mut result);
        result
    }

    fn layout_inner(&self, area: Rect, result: &mut HashMap<AppId, Rect>) {
        match self {
            Self::Leaf { app_id } => {
                result.insert(*app_id, area);
            }
            Self::Split { direction, ratio, first, second } => {
                let (first_area, second_area) = match direction {
                    SplitDirection::Horizontal => {
                        let height1 = (area.height as f32 * ratio) as u16;
                        let height2 = area.height.saturating_sub(height1);
                        (
                            Rect::new(area.x, area.y, area.width, height1),
                            Rect::new(area.x, area.y + height1, area.width, height2),
                        )
                    }
                    SplitDirection::Vertical => {
                        let width1 = (area.width as f32 * ratio) as u16;
                        let width2 = area.width.saturating_sub(width1);
                        (
                            Rect::new(area.x, area.y, width1, area.height),
                            Rect::new(area.x + width1, area.y, width2, area.height),
                        )
                    }
                };

                first.layout_inner(first_area, result);
                second.layout_inner(second_area, result);
            }
        }
    }
}

/// Floating window state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingWindow {
    /// App ID.
    pub app_id: AppId,
    /// Position X.
    pub x: u16,
    /// Position Y.
    pub y: u16,
    /// Width.
    pub width: u16,
    /// Height.
    pub height: u16,
    /// Z-order (higher = on top).
    pub z_order: u16,
    /// Whether minimized.
    pub minimized: bool,
    /// Whether maximized.
    pub maximized: bool,
}

impl FloatingWindow {
    /// Create new floating window.
    pub fn new(app_id: AppId, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            app_id,
            x,
            y,
            width,
            height,
            z_order: 0,
            minimized: false,
            maximized: false,
        }
    }

    /// Get rectangle.
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    /// Check if point is inside.
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Tab in tabbed layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    /// App ID.
    pub app_id: AppId,
    /// Tab title.
    pub title: String,
}

/// Complete layout state for serialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutState {
    /// Tiled layout tree.
    pub tiled: Option<LayoutNode>,
    /// Floating windows.
    pub floating: Vec<FloatingWindow>,
    /// Tabs.
    pub tabs: Vec<Tab>,
    /// Active tab index.
    pub active_tab: usize,
    /// Fullscreen app (if any).
    pub fullscreen: Option<AppId>,
}

impl LayoutState {
    /// Get all app IDs.
    pub fn all_apps(&self) -> Vec<AppId> {
        let mut apps = Vec::new();

        if let Some(ref tiled) = self.tiled {
            apps.extend(tiled.app_ids());
        }

        for win in &self.floating {
            apps.push(win.app_id);
        }

        for tab in &self.tabs {
            apps.push(tab.app_id);
        }

        apps
    }
}

/// Compositor for managing app layout and rendering.
pub struct Compositor {
    /// Current layout state.
    state: LayoutState,
    /// Screen size.
    screen: Rect,
    /// Focused app.
    focused: Option<AppId>,
    /// Next z-order for floating windows.
    next_z: u16,
    /// Whether to show borders.
    show_borders: bool,
    /// Status bar height.
    status_bar_height: u16,
}

impl Compositor {
    /// Create new compositor.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            state: LayoutState::default(),
            screen: Rect::new(0, 0, width, height),
            focused: None,
            next_z: 1,
            show_borders: true,
            status_bar_height: 1,
        }
    }

    /// Resize screen.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.screen = Rect::new(0, 0, width, height);
    }

    /// Get usable area (excluding status bar).
    pub fn usable_area(&self) -> Rect {
        Rect::new(
            0,
            0,
            self.screen.width,
            self.screen.height.saturating_sub(self.status_bar_height),
        )
    }

    /// Get status bar area.
    pub fn status_bar_area(&self) -> Rect {
        Rect::new(
            0,
            self.screen.height.saturating_sub(self.status_bar_height),
            self.screen.width,
            self.status_bar_height,
        )
    }

    /// Set fullscreen app.
    pub fn set_fullscreen(&mut self, app_id: Option<AppId>) {
        self.state.fullscreen = app_id;
        if app_id.is_some() {
            self.focused = app_id;
        }
    }

    /// Toggle fullscreen for app.
    pub fn toggle_fullscreen(&mut self, app_id: AppId) {
        if self.state.fullscreen == Some(app_id) {
            self.state.fullscreen = None;
        } else {
            self.state.fullscreen = Some(app_id);
            self.focused = Some(app_id);
        }
    }

    /// Add app to tiled layout.
    pub fn add_tiled(&mut self, app_id: AppId, direction: Option<SplitDirection>) {
        let new_leaf = LayoutNode::leaf(app_id);

        self.state.tiled = Some(match self.state.tiled.take() {
            None => new_leaf,
            Some(existing) => {
                let dir = direction.unwrap_or(SplitDirection::Vertical);
                match dir {
                    SplitDirection::Horizontal => LayoutNode::hsplit(existing, new_leaf, 0.5),
                    SplitDirection::Vertical => LayoutNode::vsplit(existing, new_leaf, 0.5),
                }
            }
        });

        self.focused = Some(app_id);
    }

    /// Add floating window.
    pub fn add_floating(&mut self, app_id: AppId) {
        let area = self.usable_area();
        let width = (area.width * 3) / 4;
        let height = (area.height * 3) / 4;
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;

        let mut window = FloatingWindow::new(app_id, x, y, width, height);
        window.z_order = self.next_z;
        self.next_z += 1;

        self.state.floating.push(window);
        self.focused = Some(app_id);
    }

    /// Add tab.
    pub fn add_tab(&mut self, app_id: AppId, title: impl Into<String>) {
        self.state.tabs.push(Tab {
            app_id,
            title: title.into(),
        });
        self.state.active_tab = self.state.tabs.len() - 1;
        self.focused = Some(app_id);
    }

    /// Remove app from layout.
    pub fn remove(&mut self, app_id: AppId) {
        // Remove from tiled
        if let Some(ref tiled) = self.state.tiled {
            self.state.tiled = tiled.remove(app_id);
        }

        // Remove from floating
        self.state.floating.retain(|w| w.app_id != app_id);

        // Remove from tabs
        if let Some(pos) = self.state.tabs.iter().position(|t| t.app_id == app_id) {
            self.state.tabs.remove(pos);
            if self.state.active_tab >= self.state.tabs.len() && !self.state.tabs.is_empty() {
                self.state.active_tab = self.state.tabs.len() - 1;
            }
        }

        // Clear focus if needed
        if self.focused == Some(app_id) {
            self.focused = None;
        }

        // Clear fullscreen if needed
        if self.state.fullscreen == Some(app_id) {
            self.state.fullscreen = None;
        }
    }

    /// Focus an app.
    pub fn focus(&mut self, app_id: AppId) {
        self.focused = Some(app_id);

        // Bring floating window to top
        if let Some(window) = self.state.floating.iter_mut().find(|w| w.app_id == app_id) {
            window.z_order = self.next_z;
            self.next_z += 1;
        }

        // Switch to tab
        if let Some(pos) = self.state.tabs.iter().position(|t| t.app_id == app_id) {
            self.state.active_tab = pos;
        }
    }

    /// Get focused app.
    pub fn focused(&self) -> Option<AppId> {
        self.focused
    }

    /// Get rectangle for an app.
    pub fn get_rect(&self, app_id: AppId) -> Option<Rect> {
        // Fullscreen takes priority
        if self.state.fullscreen == Some(app_id) {
            return Some(self.usable_area());
        }

        // Check floating
        if let Some(window) = self.state.floating.iter().find(|w| w.app_id == app_id) {
            if window.maximized {
                return Some(self.usable_area());
            }
            if !window.minimized {
                return Some(window.rect());
            }
        }

        // Check tabs
        if let Some(pos) = self.state.tabs.iter().position(|t| t.app_id == app_id) {
            if pos == self.state.active_tab {
                // Tab content area (excluding tab bar)
                let area = self.usable_area();
                return Some(Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1)));
            }
        }

        // Check tiled
        if let Some(ref tiled) = self.state.tiled {
            let layouts = tiled.layout(self.usable_area());
            return layouts.get(&app_id).copied();
        }

        None
    }

    /// Get all visible app rectangles.
    pub fn get_all_rects(&self) -> HashMap<AppId, Rect> {
        let mut result = HashMap::new();

        // Fullscreen overrides everything
        if let Some(app_id) = self.state.fullscreen {
            result.insert(app_id, self.usable_area());
            return result;
        }

        // Tiled layout
        if let Some(ref tiled) = self.state.tiled {
            result.extend(tiled.layout(self.usable_area()));
        }

        // Floating windows (sorted by z-order)
        let mut floating: Vec<_> = self.state.floating.iter().collect();
        floating.sort_by_key(|w| w.z_order);

        for window in floating {
            if !window.minimized {
                let rect = if window.maximized {
                    self.usable_area()
                } else {
                    window.rect()
                };
                result.insert(window.app_id, rect);
            }
        }

        // Active tab
        if !self.state.tabs.is_empty() && self.state.active_tab < self.state.tabs.len() {
            let tab = &self.state.tabs[self.state.active_tab];
            let area = self.usable_area();
            result.insert(
                tab.app_id,
                Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1)),
            );
        }

        result
    }

    /// Get app at position.
    pub fn app_at(&self, x: u16, y: u16) -> Option<AppId> {
        // Check fullscreen
        if let Some(app_id) = self.state.fullscreen {
            if self.usable_area().contains(ratatui::layout::Position { x, y }) {
                return Some(app_id);
            }
        }

        // Check floating (reverse z-order)
        let mut floating: Vec<_> = self.state.floating.iter().collect();
        floating.sort_by_key(|w| std::cmp::Reverse(w.z_order));

        for window in floating {
            if !window.minimized && window.contains(x, y) {
                return Some(window.app_id);
            }
        }

        // Check tiled
        if let Some(ref tiled) = self.state.tiled {
            for (app_id, rect) in tiled.layout(self.usable_area()) {
                if rect.contains(ratatui::layout::Position { x, y }) {
                    return Some(app_id);
                }
            }
        }

        None
    }

    /// Move floating window.
    pub fn move_floating(&mut self, app_id: AppId, dx: i16, dy: i16) {
        if let Some(window) = self.state.floating.iter_mut().find(|w| w.app_id == app_id) {
            window.x = (window.x as i16 + dx).max(0) as u16;
            window.y = (window.y as i16 + dy).max(0) as u16;
        }
    }

    /// Resize floating window.
    pub fn resize_floating(&mut self, app_id: AppId, dw: i16, dh: i16) {
        if let Some(window) = self.state.floating.iter_mut().find(|w| w.app_id == app_id) {
            window.width = ((window.width as i16 + dw).max(10)) as u16;
            window.height = ((window.height as i16 + dh).max(5)) as u16;
        }
    }

    /// Toggle maximize for floating window.
    pub fn toggle_maximize(&mut self, app_id: AppId) {
        if let Some(window) = self.state.floating.iter_mut().find(|w| w.app_id == app_id) {
            window.maximized = !window.maximized;
        }
    }

    /// Minimize floating window.
    pub fn minimize(&mut self, app_id: AppId) {
        if let Some(window) = self.state.floating.iter_mut().find(|w| w.app_id == app_id) {
            window.minimized = true;
        }
    }

    /// Restore minimized window.
    pub fn restore(&mut self, app_id: AppId) {
        if let Some(window) = self.state.floating.iter_mut().find(|w| w.app_id == app_id) {
            window.minimized = false;
        }
    }

    /// Switch to next tab.
    pub fn next_tab(&mut self) {
        if !self.state.tabs.is_empty() {
            self.state.active_tab = (self.state.active_tab + 1) % self.state.tabs.len();
            self.focused = Some(self.state.tabs[self.state.active_tab].app_id);
        }
    }

    /// Switch to previous tab.
    pub fn prev_tab(&mut self) {
        if !self.state.tabs.is_empty() {
            self.state.active_tab = if self.state.active_tab == 0 {
                self.state.tabs.len() - 1
            } else {
                self.state.active_tab - 1
            };
            self.focused = Some(self.state.tabs[self.state.active_tab].app_id);
        }
    }

    /// Get layout state for serialization.
    pub fn state(&self) -> &LayoutState {
        &self.state
    }

    /// Restore layout state.
    pub fn restore_state(&mut self, state: LayoutState) {
        self.state = state;
    }

    /// Set border visibility.
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Get border visibility.
    pub fn show_borders(&self) -> bool {
        self.show_borders
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_node_leaf() {
        let node = LayoutNode::leaf(1);
        assert_eq!(node.app_ids(), vec![1]);
        assert!(node.contains(1));
        assert!(!node.contains(2));
    }

    #[test]
    fn test_layout_node_split() {
        let node = LayoutNode::vsplit(LayoutNode::leaf(1), LayoutNode::leaf(2), 0.5);

        assert_eq!(node.app_ids().len(), 2);
        assert!(node.contains(1));
        assert!(node.contains(2));
    }

    #[test]
    fn test_layout_calculation() {
        let node = LayoutNode::vsplit(LayoutNode::leaf(1), LayoutNode::leaf(2), 0.5);

        let area = Rect::new(0, 0, 100, 50);
        let rects = node.layout(area);

        assert!(rects.contains_key(&1));
        assert!(rects.contains_key(&2));

        let r1 = rects[&1];
        let r2 = rects[&2];

        assert_eq!(r1.width, 50);
        assert_eq!(r2.width, 50);
        assert_eq!(r1.x + r1.width, r2.x);
    }

    #[test]
    fn test_compositor_tiled() {
        let mut comp = Compositor::new(100, 50);

        comp.add_tiled(1, None);
        comp.add_tiled(2, Some(SplitDirection::Vertical));

        let rects = comp.get_all_rects();
        assert_eq!(rects.len(), 2);
    }

    #[test]
    fn test_compositor_floating() {
        let mut comp = Compositor::new(100, 50);

        comp.add_floating(1);
        comp.add_floating(2);

        assert!(comp.get_rect(1).is_some());
        assert!(comp.get_rect(2).is_some());
    }

    #[test]
    fn test_compositor_focus() {
        let mut comp = Compositor::new(100, 50);

        comp.add_tiled(1, None);
        comp.add_tiled(2, None);

        comp.focus(1);
        assert_eq!(comp.focused(), Some(1));

        comp.focus(2);
        assert_eq!(comp.focused(), Some(2));
    }

    #[test]
    fn test_remove_node() {
        let node = LayoutNode::vsplit(LayoutNode::leaf(1), LayoutNode::leaf(2), 0.5);

        let remaining = node.remove(1).unwrap();
        assert!(!remaining.contains(1));
        assert!(remaining.contains(2));
    }
}
