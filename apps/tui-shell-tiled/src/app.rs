use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::container::{Container, ContainerId, Direction, Workspace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Resize,
    Move,
    Launcher,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDir {
    Left,
    Right,
    Up,
    Down,
}

pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub workspaces: Vec<Workspace>,
    pub active_workspace: usize,
    pub next_container_id: ContainerId,
    pub scratchpad: Vec<Container>,
    pub show_help: bool,
    pub message: Option<String>,
    pub launcher_input: String,
}

impl App {
    pub fn new(config: Config) -> Self {
        // Create default workspaces
        let workspaces: Vec<Workspace> = config
            .workspaces
            .names
            .iter()
            .enumerate()
            .map(|(i, name)| Workspace::new(i as u32 + 1, name.clone()))
            .collect();

        let mut app = Self {
            config,
            mode: Mode::Normal,
            workspaces,
            active_workspace: 0,
            next_container_id: 1,
            scratchpad: Vec::new(),
            show_help: false,
            message: None,
            launcher_input: String::new(),
        };

        // Add some demo apps
        app.spawn_app("task-manager", "Task Manager");
        app.split_current(Direction::Horizontal);
        app.spawn_app("note-manager", "Note Manager");

        app
    }

    fn next_id(&mut self) -> ContainerId {
        let id = self.next_container_id;
        self.next_container_id += 1;
        id
    }

    pub fn current_workspace(&self) -> &Workspace {
        &self.workspaces[self.active_workspace]
    }

    pub fn current_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace]
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;

        match self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Resize => self.handle_resize_key(key),
            Mode::Move => self.handle_move_key(key),
            Mode::Launcher => self.handle_launcher_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_mod = key.modifiers.contains(KeyModifiers::CONTROL);

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

            // Focus movement
            KeyCode::Char('h') if is_mod => self.focus_direction(FocusDir::Left),
            KeyCode::Char('l') if is_mod => self.focus_direction(FocusDir::Right),
            KeyCode::Char('j') if is_mod => self.focus_direction(FocusDir::Down),
            KeyCode::Char('k') if is_mod => self.focus_direction(FocusDir::Up),

            // Split operations
            KeyCode::Char('v') if is_mod => {
                self.split_current(Direction::Vertical);
            }
            KeyCode::Char('b') if is_mod => {
                self.split_current(Direction::Horizontal);
            }

            // Resize mode
            KeyCode::Char('r') if is_mod => {
                self.mode = Mode::Resize;
                self.message = Some("Resize mode: h/l=width, j/k=height, Enter/Esc=exit".to_string());
            }

            // Move mode
            KeyCode::Char('m') if is_mod => {
                self.mode = Mode::Move;
                self.message = Some("Move mode: h/j/k/l=move, Enter/Esc=exit".to_string());
            }

            // Workspace switching
            KeyCode::Char('1') if is_mod => self.switch_workspace(0),
            KeyCode::Char('2') if is_mod => self.switch_workspace(1),
            KeyCode::Char('3') if is_mod => self.switch_workspace(2),
            KeyCode::Char('4') if is_mod => self.switch_workspace(3),
            KeyCode::Char('5') if is_mod => self.switch_workspace(4),
            KeyCode::Char('6') if is_mod => self.switch_workspace(5),
            KeyCode::Char('7') if is_mod => self.switch_workspace(6),
            KeyCode::Char('8') if is_mod => self.switch_workspace(7),
            KeyCode::Char('9') if is_mod => self.switch_workspace(8),

            // Close focused app
            KeyCode::Char('w') if is_mod => {
                self.close_focused();
            }

            // Toggle fullscreen (placeholder)
            KeyCode::Char('f') if is_mod => {
                self.message = Some("Fullscreen toggle (not implemented in demo)".to_string());
            }

            _ => {}
        }

        false
    }

    fn handle_resize_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.resize_current(-0.05, 0.0);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.resize_current(0.05, 0.0);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.resize_current(0.0, 0.05);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.resize_current(0.0, -0.05);
            }
            _ => {}
        }
        false
    }

    fn handle_move_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.move_focused(FocusDir::Left);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.move_focused(FocusDir::Right);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_focused(FocusDir::Down);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_focused(FocusDir::Up);
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
                    self.spawn_app(&name, &name);
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

    fn focus_direction(&mut self, dir: FocusDir) {
        let ws = self.current_workspace_mut();
        if let Container::Split {
            children, focused, direction, ..
        } = &mut ws.root
        {
            let can_move = match (direction, dir) {
                (Direction::Horizontal, FocusDir::Left) => *focused > 0,
                (Direction::Horizontal, FocusDir::Right) => *focused < children.len() - 1,
                (Direction::Vertical, FocusDir::Up) => *focused > 0,
                (Direction::Vertical, FocusDir::Down) => *focused < children.len() - 1,
                _ => false,
            };

            if can_move {
                match dir {
                    FocusDir::Left | FocusDir::Up => *focused = focused.saturating_sub(1),
                    FocusDir::Right | FocusDir::Down => *focused = (*focused + 1).min(children.len() - 1),
                }
            }
        }
    }

    fn split_current(&mut self, direction: Direction) {
        // Get IDs first to avoid borrow conflicts
        let id = self.next_id();
        let id2 = self.next_id();
        let ws_idx = self.active_workspace;

        let ws = &mut self.workspaces[ws_idx];
        match &mut ws.root {
            Container::Empty { .. } => {
                ws.root = Container::new_split(
                    id,
                    direction,
                    vec![Container::Empty { id: id2 }],
                );
            }
            Container::Split { children, ratios, focused, direction: split_dir, .. } => {
                if *split_dir == direction {
                    children.insert(*focused + 1, Container::Empty { id: id2 });
                    ratios.push(0.0);
                    let ratio = 1.0 / children.len() as f32;
                    for r in ratios.iter_mut() {
                        *r = ratio;
                    }
                    *focused += 1;
                } else {
                    let current = children[*focused].clone();
                    children[*focused] = Container::new_split(
                        id,
                        direction,
                        vec![current, Container::Empty { id: id2 }],
                    );
                }
            }
            Container::App { .. } => {
                let old_root = ws.root.clone();
                ws.root = Container::new_split(
                    id,
                    direction,
                    vec![old_root, Container::Empty { id: id2 }],
                );
            }
            Container::Tabbed { .. } => {}
        }
    }

    fn spawn_app(&mut self, name: &str, title: &str) {
        let id = self.next_id();
        let ws_idx = self.active_workspace;

        let ws = &mut self.workspaces[ws_idx];
        match &mut ws.root {
            Container::Empty { .. } => {
                ws.root = Container::new_app(id, name.to_string(), title.to_string());
            }
            Container::Split { children, focused, .. } => {
                if let Some(child) = children.get_mut(*focused) {
                    if child.is_empty() {
                        *child = Container::new_app(id, name.to_string(), title.to_string());
                    }
                }
            }
            Container::App { .. } => {}
            Container::Tabbed { .. } => {}
        }
    }

    fn close_focused(&mut self) {
        let id = self.next_id();
        let ws_idx = self.active_workspace;

        let ws = &mut self.workspaces[ws_idx];
        match &mut ws.root {
            Container::App { .. } => {
                ws.root = Container::Empty { id };
            }
            Container::Split { children, focused, ratios, .. } => {
                if children.len() > 1 {
                    children.remove(*focused);
                    ratios.pop();
                    let ratio = 1.0 / children.len() as f32;
                    for r in ratios.iter_mut() {
                        *r = ratio;
                    }
                    if *focused >= children.len() {
                        *focused = children.len() - 1;
                    }
                } else if children.len() == 1 {
                    if matches!(children.first(), Some(Container::App { .. })) {
                        children[0] = Container::Empty { id };
                    }
                }
            }
            Container::Empty { .. } => {}
            Container::Tabbed { .. } => {}
        }
    }

    fn resize_current(&mut self, _dw: f32, _dh: f32) {
        // Simplified resize - would adjust ratios in real implementation
        self.message = Some("Resize (demo only)".to_string());
    }

    fn move_focused(&mut self, _dir: FocusDir) {
        // Simplified move - would swap containers in real implementation
        self.message = Some("Move (demo only)".to_string());
    }

    fn switch_workspace(&mut self, index: usize) {
        if index < self.workspaces.len() {
            self.active_workspace = index;
            self.message = Some(format!(
                "Switched to workspace {}",
                self.workspaces[index].name
            ));
        }
    }

    pub fn focused_title(&self) -> String {
        if let Some(app) = self.current_workspace().root.find_focused_app() {
            match app {
                Container::App { title, .. } => title.clone(),
                _ => "Empty".to_string(),
            }
        } else {
            "Empty".to_string()
        }
    }
}
