use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::tab::{Direction, Tab, TabId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Launcher,
}

pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub next_tab_id: TabId,
    pub show_help: bool,
    pub message: Option<String>,
    pub launcher_input: String,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut app = Self {
            config,
            mode: Mode::Normal,
            tabs: Vec::new(),
            active_tab: 0,
            next_tab_id: 1,
            show_help: false,
            message: None,
            launcher_input: String::new(),
        };

        // Create initial tabs
        app.new_tab("task-manager", "Task Manager");
        app.new_tab("note-manager", "Note Manager");
        app.new_tab("file-manager", "File Manager");

        // Pin first tab
        if let Some(tab) = app.tabs.first_mut() {
            tab.pinned = true;
        }

        app
    }

    fn next_id(&mut self) -> TabId {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        id
    }

    pub fn current_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    pub fn current_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;

        match self.mode {
            Mode::Normal => self.handle_normal_key(key),
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

            // New tab
            KeyCode::Char('t') if is_mod => {
                self.mode = Mode::Launcher;
                self.launcher_input.clear();
            }

            // Close tab
            KeyCode::Char('w') if is_mod => {
                self.close_current_tab();
            }

            // Tab navigation
            KeyCode::Tab if is_mod && is_shift => {
                self.prev_tab();
            }
            KeyCode::Tab if is_mod => {
                self.next_tab();
            }

            // Jump to tab by number
            KeyCode::Char('1') if is_mod => self.go_to_tab(0),
            KeyCode::Char('2') if is_mod => self.go_to_tab(1),
            KeyCode::Char('3') if is_mod => self.go_to_tab(2),
            KeyCode::Char('4') if is_mod => self.go_to_tab(3),
            KeyCode::Char('5') if is_mod => self.go_to_tab(4),
            KeyCode::Char('6') if is_mod => self.go_to_tab(5),
            KeyCode::Char('7') if is_mod => self.go_to_tab(6),
            KeyCode::Char('8') if is_mod => self.go_to_tab(7),
            KeyCode::Char('9') if is_mod => self.go_to_tab(8),
            KeyCode::Char('0') if is_mod => {
                // Go to last tab
                if !self.tabs.is_empty() {
                    self.active_tab = self.tabs.len() - 1;
                }
            }

            // Move tab
            KeyCode::Left if is_mod && is_shift => {
                self.move_tab_left();
            }
            KeyCode::Right if is_mod && is_shift => {
                self.move_tab_right();
            }

            // Splits
            KeyCode::Char('v') if is_mod => {
                self.split_current(Direction::Vertical);
            }
            KeyCode::Char('h') if is_mod && !is_shift => {
                self.split_current(Direction::Horizontal);
            }

            // Focus pane
            KeyCode::Left if is_mod && !is_shift => {
                if let Some(tab) = self.current_tab_mut() {
                    tab.focus_pane(Direction::Horizontal);
                }
            }
            KeyCode::Right if is_mod && !is_shift => {
                if let Some(tab) = self.current_tab_mut() {
                    tab.focus_pane(Direction::Horizontal);
                }
            }
            KeyCode::Up if is_mod && !is_shift => {
                if let Some(tab) = self.current_tab_mut() {
                    tab.focus_pane(Direction::Vertical);
                }
            }
            KeyCode::Down if is_mod && !is_shift => {
                if let Some(tab) = self.current_tab_mut() {
                    tab.focus_pane(Direction::Vertical);
                }
            }

            // Close pane
            KeyCode::Char('x') if is_mod => {
                if let Some(tab) = self.current_tab_mut() {
                    if !tab.close_focused_pane() {
                        // No split, close tab instead
                        self.close_current_tab();
                    }
                }
            }

            // Pin tab
            KeyCode::Char('p') if is_mod => {
                if let Some(tab) = self.current_tab_mut() {
                    tab.pinned = !tab.pinned;
                    self.message = Some(format!(
                        "Tab {}",
                        if tab.pinned { "pinned" } else { "unpinned" }
                    ));
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
                    self.new_tab(&name, &name);
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

    fn new_tab(&mut self, name: &str, title: &str) {
        let id = self.next_id();
        let tab = Tab::new(id, name, title);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
    }

    fn close_current_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        // Don't close pinned tabs easily
        if let Some(tab) = self.current_tab() {
            if tab.pinned {
                self.message = Some("Cannot close pinned tab".to_string());
                return;
            }
        }

        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
    }

    fn go_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    fn move_tab_left(&mut self) {
        if self.active_tab > 0 {
            self.tabs.swap(self.active_tab, self.active_tab - 1);
            self.active_tab -= 1;
        }
    }

    fn move_tab_right(&mut self) {
        if self.active_tab < self.tabs.len() - 1 {
            self.tabs.swap(self.active_tab, self.active_tab + 1);
            self.active_tab += 1;
        }
    }

    fn split_current(&mut self, direction: Direction) {
        self.mode = Mode::Launcher;
        self.launcher_input.clear();

        // Store direction for when launcher completes
        // For simplicity, we'll create the split when a new app is selected
        if let Some(tab) = self.current_tab_mut() {
            tab.split(direction, "new-app", "New App");
        }
    }
}
