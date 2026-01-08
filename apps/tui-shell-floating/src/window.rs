use serde::{Deserialize, Serialize};

pub type WindowId = u32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn to_ratatui(&self) -> ratatui::prelude::Rect {
        ratatui::prelude::Rect::new(self.x, self.y, self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    Normal,
    Maximized,
    Minimized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapPosition {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: WindowId,
    pub name: String,
    pub title: String,
    pub rect: Rect,
    pub saved_rect: Rect,
    pub state: WindowState,
    pub z_order: u32,
    pub always_on_top: bool,
}

impl Window {
    pub fn new(id: WindowId, name: String, title: String, rect: Rect) -> Self {
        Self {
            id,
            name,
            title,
            rect,
            saved_rect: rect,
            state: WindowState::Normal,
            z_order: id,
            always_on_top: false,
        }
    }

    pub fn maximize(&mut self, screen_width: u16, screen_height: u16) {
        if self.state != WindowState::Maximized {
            self.saved_rect = self.rect;
            self.rect = Rect::new(0, 0, screen_width, screen_height.saturating_sub(1));
            self.state = WindowState::Maximized;
        }
    }

    pub fn restore(&mut self) {
        if self.state == WindowState::Maximized || self.state == WindowState::Minimized {
            self.rect = self.saved_rect;
            self.state = WindowState::Normal;
        }
    }

    pub fn minimize(&mut self) {
        if self.state != WindowState::Minimized {
            if self.state == WindowState::Normal {
                self.saved_rect = self.rect;
            }
            self.state = WindowState::Minimized;
        }
    }

    pub fn snap(&mut self, position: SnapPosition, screen_width: u16, screen_height: u16) {
        let content_height = screen_height.saturating_sub(1); // Account for taskbar
        let half_width = screen_width / 2;
        let half_height = content_height / 2;

        if self.state == WindowState::Normal {
            self.saved_rect = self.rect;
        }

        self.rect = match position {
            SnapPosition::Left => Rect::new(0, 0, half_width, content_height),
            SnapPosition::Right => Rect::new(half_width, 0, half_width, content_height),
            SnapPosition::TopLeft => Rect::new(0, 0, half_width, half_height),
            SnapPosition::TopRight => Rect::new(half_width, 0, half_width, half_height),
            SnapPosition::BottomLeft => Rect::new(0, half_height, half_width, half_height),
            SnapPosition::BottomRight => Rect::new(half_width, half_height, half_width, half_height),
        };

        self.state = WindowState::Normal;
    }

    pub fn move_by(&mut self, dx: i16, dy: i16, screen_width: u16, screen_height: u16) {
        let new_x = (self.rect.x as i16 + dx).max(0) as u16;
        let new_y = (self.rect.y as i16 + dy).max(0) as u16;

        self.rect.x = new_x.min(screen_width.saturating_sub(self.rect.width));
        self.rect.y = new_y.min(screen_height.saturating_sub(self.rect.height + 1));
    }

    pub fn resize_by(&mut self, dw: i16, dh: i16, min_width: u16, min_height: u16) {
        let new_width = ((self.rect.width as i16 + dw).max(min_width as i16)) as u16;
        let new_height = ((self.rect.height as i16 + dh).max(min_height as i16)) as u16;

        self.rect.width = new_width;
        self.rect.height = new_height;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desktop {
    pub id: u32,
    pub name: String,
    pub windows: Vec<Window>,
    pub focused: Option<WindowId>,
}

impl Desktop {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            windows: Vec::new(),
            focused: None,
        }
    }

    pub fn add_window(&mut self, window: Window) {
        let id = window.id;
        self.windows.push(window);
        self.focused = Some(id);
        self.raise_window(id);
    }

    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
        if self.focused == Some(id) {
            self.focused = self.windows.last().map(|w| w.id);
        }
    }

    pub fn focused_window(&self) -> Option<&Window> {
        self.focused.and_then(|id| self.windows.iter().find(|w| w.id == id))
    }

    pub fn focused_window_mut(&mut self) -> Option<&mut Window> {
        let focused = self.focused?;
        self.windows.iter_mut().find(|w| w.id == focused)
    }

    pub fn raise_window(&mut self, id: WindowId) {
        let max_z = self.windows.iter().map(|w| w.z_order).max().unwrap_or(0);
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
            window.z_order = max_z + 1;
        }
        self.focused = Some(id);
    }

    pub fn windows_sorted_by_z(&self) -> Vec<&Window> {
        let mut windows: Vec<_> = self.windows.iter()
            .filter(|w| w.state != WindowState::Minimized)
            .collect();
        windows.sort_by_key(|w| w.z_order);
        windows
    }

    pub fn cycle_focus(&mut self, reverse: bool) {
        let visible: Vec<WindowId> = self.windows.iter()
            .filter(|w| w.state != WindowState::Minimized)
            .map(|w| w.id)
            .collect();

        if visible.is_empty() {
            return;
        }

        let current_idx = self.focused
            .and_then(|id| visible.iter().position(|&w| w == id))
            .unwrap_or(0);

        let next_idx = if reverse {
            if current_idx == 0 {
                visible.len() - 1
            } else {
                current_idx - 1
            }
        } else {
            (current_idx + 1) % visible.len()
        };

        if let Some(&id) = visible.get(next_idx) {
            self.raise_window(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 20, 20);
        assert!(rect.contains(15, 15));
        assert!(rect.contains(10, 10));
        assert!(!rect.contains(30, 30));
        assert!(!rect.contains(5, 5));
    }

    #[test]
    fn test_window_maximize_restore() {
        let mut window = Window::new(1, "test".to_string(), "Test".to_string(), Rect::new(10, 10, 40, 20));
        let original_rect = window.rect;

        window.maximize(100, 50);
        assert_eq!(window.state, WindowState::Maximized);
        assert_eq!(window.rect.x, 0);
        assert_eq!(window.rect.y, 0);

        window.restore();
        assert_eq!(window.state, WindowState::Normal);
        assert_eq!(window.rect.x, original_rect.x);
        assert_eq!(window.rect.y, original_rect.y);
    }

    #[test]
    fn test_desktop_add_remove() {
        let mut desktop = Desktop::new(1, "Test".to_string());
        let window = Window::new(1, "app".to_string(), "App".to_string(), Rect::new(0, 0, 50, 20));

        desktop.add_window(window);
        assert_eq!(desktop.windows.len(), 1);
        assert_eq!(desktop.focused, Some(1));

        desktop.remove_window(1);
        assert_eq!(desktop.windows.len(), 0);
        assert_eq!(desktop.focused, None);
    }
}
