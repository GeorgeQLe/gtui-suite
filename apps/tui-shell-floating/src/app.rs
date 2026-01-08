use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::window::{Desktop, Rect, SnapPosition, Window, WindowId, WindowState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Move,
    Resize,
    Launcher,
}

pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub desktops: Vec<Desktop>,
    pub active_desktop: usize,
    pub next_window_id: WindowId,
    pub cascade_offset: u16,
    pub show_help: bool,
    pub message: Option<String>,
    pub launcher_input: String,
    pub screen_width: u16,
    pub screen_height: u16,
}

impl App {
    pub fn new(config: Config) -> Self {
        let desktops: Vec<Desktop> = config
            .desktops
            .names
            .iter()
            .enumerate()
            .map(|(i, name)| Desktop::new(i as u32 + 1, name.clone()))
            .collect();

        let mut app = Self {
            config,
            mode: Mode::Normal,
            desktops,
            active_desktop: 0,
            next_window_id: 1,
            cascade_offset: 0,
            show_help: false,
            message: None,
            launcher_input: String::new(),
            screen_width: 80,
            screen_height: 24,
        };

        // Add demo windows
        app.spawn_window("task-manager", "Task Manager");
        app.spawn_window("note-manager", "Note Manager");
        app.spawn_window("file-manager", "File Manager");

        app
    }

    pub fn set_screen_size(&mut self, width: u16, height: u16) {
        self.screen_width = width;
        self.screen_height = height;
    }

    fn next_id(&mut self) -> WindowId {
        let id = self.next_window_id;
        self.next_window_id += 1;
        id
    }

    pub fn current_desktop(&self) -> &Desktop {
        &self.desktops[self.active_desktop]
    }

    pub fn current_desktop_mut(&mut self) -> &mut Desktop {
        &mut self.desktops[self.active_desktop]
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;

        match self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Move => self.handle_move_key(key),
            Mode::Resize => self.handle_resize_key(key),
            Mode::Launcher => self.handle_launcher_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_mod = key.modifiers.contains(KeyModifiers::CONTROL);
        let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            // Exit
            KeyCode::Char('q') if is_mod => return true,
            KeyCode::Char('c') if is_mod => return true,

            // Help
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
            }
            KeyCode::Esc => {
                self.show_help = false;
            }

            // Launcher
            KeyCode::Char('p') if is_mod => {
                self.mode = Mode::Launcher;
                self.launcher_input.clear();
            }

            // Window cycling
            KeyCode::Tab if is_mod && is_shift => {
                self.current_desktop_mut().cycle_focus(true);
            }
            KeyCode::Tab if is_mod => {
                self.current_desktop_mut().cycle_focus(false);
            }

            // Close window
            KeyCode::Char('w') if is_mod => {
                self.close_focused();
            }

            // Move mode
            KeyCode::Char('m') if is_mod && !is_shift => {
                self.mode = Mode::Move;
                self.message = Some("Move mode: arrows to move, Enter/Esc to exit".to_string());
            }

            // Resize mode
            KeyCode::Char('r') if is_mod => {
                self.mode = Mode::Resize;
                self.message = Some("Resize mode: arrows to resize, Enter/Esc to exit".to_string());
            }

            // Maximize
            KeyCode::Char('m') if is_mod && is_shift => {
                self.toggle_maximize();
            }
            KeyCode::Up if is_mod && !is_shift => {
                self.toggle_maximize();
            }

            // Minimize
            KeyCode::Char('n') if is_mod => {
                self.minimize_focused();
            }
            KeyCode::Down if is_mod && !is_shift => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    if window.state == WindowState::Maximized {
                        window.restore();
                    } else {
                        window.minimize();
                    }
                }
            }

            // Snap left/right
            KeyCode::Left if is_mod && !is_shift => {
                self.snap_focused(SnapPosition::Left);
            }
            KeyCode::Right if is_mod && !is_shift => {
                self.snap_focused(SnapPosition::Right);
            }

            // Desktop switching
            KeyCode::Char('1') if is_mod && !is_shift => self.switch_desktop(0),
            KeyCode::Char('2') if is_mod && !is_shift => self.switch_desktop(1),
            KeyCode::Char('3') if is_mod && !is_shift => self.switch_desktop(2),
            KeyCode::Char('4') if is_mod && !is_shift => self.switch_desktop(3),

            // Move to desktop
            KeyCode::Char('1') if is_mod && is_shift => self.move_to_desktop(0),
            KeyCode::Char('2') if is_mod && is_shift => self.move_to_desktop(1),
            KeyCode::Char('3') if is_mod && is_shift => self.move_to_desktop(2),
            KeyCode::Char('4') if is_mod && is_shift => self.move_to_desktop(3),

            // Toggle always on top
            KeyCode::Char('t') if is_mod => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.always_on_top = !window.always_on_top;
                    self.message = Some(format!(
                        "Always on top: {}",
                        if window.always_on_top { "ON" } else { "OFF" }
                    ));
                }
            }

            // Cascade windows
            KeyCode::Char('a') if is_mod => {
                self.cascade_windows();
            }

            _ => {}
        }

        false
    }

    fn handle_move_key(&mut self, key: KeyEvent) -> bool {
        let screen_width = self.screen_width;
        let screen_height = self.screen_height;

        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            KeyCode::Left => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.move_by(-2, 0, screen_width, screen_height);
                }
            }
            KeyCode::Right => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.move_by(2, 0, screen_width, screen_height);
                }
            }
            KeyCode::Up => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.move_by(0, -1, screen_width, screen_height);
                }
            }
            KeyCode::Down => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.move_by(0, 1, screen_width, screen_height);
                }
            }
            _ => {}
        }
        false
    }

    fn handle_resize_key(&mut self, key: KeyEvent) -> bool {
        let min_width = self.config.window.min_width;
        let min_height = self.config.window.min_height;

        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            KeyCode::Left => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.resize_by(-2, 0, min_width, min_height);
                }
            }
            KeyCode::Right => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.resize_by(2, 0, min_width, min_height);
                }
            }
            KeyCode::Up => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.resize_by(0, -1, min_width, min_height);
                }
            }
            KeyCode::Down => {
                if let Some(window) = self.current_desktop_mut().focused_window_mut() {
                    window.resize_by(0, 1, min_width, min_height);
                }
            }
            _ => {}
        }
        false
    }

    fn handle_launcher_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                if !self.launcher_input.is_empty() {
                    let name = self.launcher_input.clone();
                    self.spawn_window(&name, &name);
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.launcher_input.pop();
            }
            KeyCode::Char(c) => {
                self.launcher_input.push(c);
            }
            _ => {}
        }
        false
    }

    fn spawn_window(&mut self, name: &str, title: &str) {
        let id = self.next_id();
        let x = 2 + self.cascade_offset;
        let y = 1 + (self.cascade_offset / 2);
        let width = self.config.window.default_width.min(self.screen_width.saturating_sub(x + 2));
        let height = self.config.window.default_height.min(self.screen_height.saturating_sub(y + 2));

        let rect = Rect::new(x, y, width, height);
        let window = Window::new(id, name.to_string(), title.to_string(), rect);

        self.current_desktop_mut().add_window(window);

        // Update cascade offset
        self.cascade_offset = (self.cascade_offset + 3) % 15;
    }

    fn close_focused(&mut self) {
        if let Some(id) = self.current_desktop().focused {
            self.current_desktop_mut().remove_window(id);
        }
    }

    fn toggle_maximize(&mut self) {
        let screen_width = self.screen_width;
        let screen_height = self.screen_height;

        if let Some(window) = self.current_desktop_mut().focused_window_mut() {
            if window.state == WindowState::Maximized {
                window.restore();
            } else {
                window.maximize(screen_width, screen_height);
            }
        }
    }

    fn minimize_focused(&mut self) {
        if let Some(window) = self.current_desktop_mut().focused_window_mut() {
            window.minimize();
        }
        // Focus next window
        self.current_desktop_mut().cycle_focus(false);
    }

    fn snap_focused(&mut self, position: SnapPosition) {
        let screen_width = self.screen_width;
        let screen_height = self.screen_height;

        if let Some(window) = self.current_desktop_mut().focused_window_mut() {
            window.snap(position, screen_width, screen_height);
        }
    }

    fn switch_desktop(&mut self, index: usize) {
        if index < self.desktops.len() {
            self.active_desktop = index;
            self.message = Some(format!("Switched to {}", self.desktops[index].name));
        }
    }

    fn move_to_desktop(&mut self, index: usize) {
        if index >= self.desktops.len() || index == self.active_desktop {
            return;
        }

        if let Some(focused_id) = self.current_desktop().focused {
            if let Some(idx) = self.desktops[self.active_desktop]
                .windows
                .iter()
                .position(|w| w.id == focused_id)
            {
                let window = self.desktops[self.active_desktop].windows.remove(idx);
                self.desktops[self.active_desktop].focused = self.desktops[self.active_desktop]
                    .windows
                    .last()
                    .map(|w| w.id);

                self.desktops[index].add_window(window);
                self.message = Some(format!("Moved to {}", self.desktops[index].name));
            }
        }
    }

    fn cascade_windows(&mut self) {
        let mut offset: u16 = 0;
        for window in self.current_desktop_mut().windows.iter_mut() {
            if window.state != WindowState::Minimized {
                window.rect.x = 2 + offset;
                window.rect.y = 1 + offset / 2;
                offset += 3;
                if offset > 15 {
                    offset = 0;
                }
            }
        }
        self.message = Some("Windows cascaded".to_string());
    }

    pub fn focused_title(&self) -> String {
        self.current_desktop()
            .focused_window()
            .map(|w| w.title.clone())
            .unwrap_or_else(|| "No window".to_string())
    }
}
