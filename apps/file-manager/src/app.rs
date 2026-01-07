use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::entry::format_bytes;
use crate::pane::Pane;

/// Which pane is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Left,
    Right,
}

/// Application mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search(String),
    Rename(String),
    NewFile(String),
    NewDir(String),
    Confirm(ConfirmAction),
    Help,
    Bookmarks,
    Sort,
}

/// Action requiring confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    Delete(Vec<PathBuf>),
}

/// Application state
pub struct App {
    pub config: Config,
    pub left_pane: Pane,
    pub right_pane: Pane,
    pub active: ActivePane,
    pub mode: Mode,
    pub message: Option<String>,
    pub show_preview: bool,
}

impl App {
    pub fn new(initial_path: Option<&str>) -> Result<Self> {
        let config = Config::load()?;

        let left_pane = Pane::new(initial_path);
        let right_pane = Pane::new(initial_path);

        Ok(Self {
            config,
            left_pane,
            right_pane,
            active: ActivePane::Left,
            mode: Mode::Normal,
            message: None,
            show_preview: true,
        })
    }

    pub fn active_pane(&self) -> &Pane {
        match self.active {
            ActivePane::Left => &self.left_pane,
            ActivePane::Right => &self.right_pane,
        }
    }

    pub fn active_pane_mut(&mut self) -> &mut Pane {
        match self.active {
            ActivePane::Left => &mut self.left_pane,
            ActivePane::Right => &mut self.right_pane,
        }
    }

    pub fn inactive_pane(&self) -> &Pane {
        match self.active {
            ActivePane::Left => &self.right_pane,
            ActivePane::Right => &self.left_pane,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Search(_) => self.handle_search_key(key),
            Mode::Rename(_) => self.handle_rename_key(key),
            Mode::NewFile(_) => self.handle_new_file_key(key),
            Mode::NewDir(_) => self.handle_new_dir_key(key),
            Mode::Confirm(_) => self.handle_confirm_key(key),
            Mode::Help => self.handle_help_key(key),
            Mode::Bookmarks => self.handle_bookmarks_key(key),
            Mode::Sort => self.handle_sort_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        self.message = None;

        match key.code {
            // Navigation
            KeyCode::Down | KeyCode::Char('j') => self.active_pane_mut().move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.active_pane_mut().move_up(),
            KeyCode::Home | KeyCode::Char('g') => self.active_pane_mut().move_to_top(),
            KeyCode::End | KeyCode::Char('G') => self.active_pane_mut().move_to_bottom(),

            // Pane switching
            KeyCode::Tab => {
                self.active = match self.active {
                    ActivePane::Left => ActivePane::Right,
                    ActivePane::Right => ActivePane::Left,
                };
            }

            // Directory navigation
            KeyCode::Left | KeyCode::Char('h') => self.active_pane_mut().go_parent(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                self.active_pane_mut().enter();
            }
            KeyCode::Backspace => self.active_pane_mut().go_parent(),

            // Selection
            KeyCode::Char(' ') => self.active_pane_mut().toggle_selection(),
            KeyCode::Char('*') => self.active_pane_mut().invert_selection(),

            // File operations
            KeyCode::Char('c') => self.copy_to_other_pane(),
            KeyCode::Char('m') => self.move_to_other_pane(),
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('r') => {
                if let Some(entry) = self.active_pane().current_entry() {
                    if entry.name != ".." {
                        self.mode = Mode::Rename(entry.name.clone());
                    }
                }
            }

            // Create
            KeyCode::Char('n') => {
                self.mode = Mode::NewFile(String::new());
            }
            KeyCode::Char('N') => {
                self.mode = Mode::NewDir(String::new());
            }

            // Display toggles
            KeyCode::Char('.') => {
                self.left_pane.toggle_hidden();
                self.right_pane.toggle_hidden();
            }
            KeyCode::Char('p') => {
                self.show_preview = !self.show_preview;
            }

            // Modes
            KeyCode::Char('/') => {
                self.mode = Mode::Search(String::new());
            }
            KeyCode::Char('s') => {
                self.mode = Mode::Sort;
            }
            KeyCode::Char('b') => {
                self.mode = Mode::Bookmarks;
            }
            KeyCode::Char('B') => {
                self.add_bookmark();
            }
            KeyCode::Char('?') => {
                self.mode = Mode::Help;
            }

            // Refresh
            KeyCode::F(5) => {
                self.left_pane.refresh();
                self.right_pane.refresh();
                self.message = Some("Refreshed".to_string());
            }

            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        if let Mode::Search(ref mut query) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let query = query.clone().to_lowercase();
                    let pane = self.active_pane_mut();
                    for (i, entry) in pane.entries.iter().enumerate() {
                        if entry.name.to_lowercase().contains(&query) {
                            pane.selected = i;
                            break;
                        }
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    query.pop();
                }
                KeyCode::Char(c) => {
                    query.push(c);
                }
                _ => {}
            }
        }
    }

    fn handle_rename_key(&mut self, key: KeyEvent) {
        if let Mode::Rename(ref mut new_name) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let new_name = new_name.clone();
                    self.do_rename(&new_name);
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    new_name.pop();
                }
                KeyCode::Char(c) => {
                    new_name.push(c);
                }
                _ => {}
            }
        }
    }

    fn handle_new_file_key(&mut self, key: KeyEvent) {
        if let Mode::NewFile(ref mut name) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let name = name.clone();
                    self.create_file(&name);
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    name.pop();
                }
                KeyCode::Char(c) => {
                    name.push(c);
                }
                _ => {}
            }
        }
    }

    fn handle_new_dir_key(&mut self, key: KeyEvent) {
        if let Mode::NewDir(ref mut name) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let name = name.clone();
                    self.create_dir(&name);
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    name.pop();
                }
                KeyCode::Char(c) => {
                    name.push(c);
                }
                _ => {}
            }
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Mode::Confirm(action) = &self.mode {
                    match action {
                        ConfirmAction::Delete(paths) => {
                            let paths = paths.clone();
                            self.do_delete(&paths);
                        }
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
            self.mode = Mode::Normal;
        }
    }

    fn handle_bookmarks_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                let bookmarks: Vec<_> = self.config.bookmarks.values().collect();
                if let Some(path) = bookmarks.get(idx) {
                    let path = PathBuf::from(path);
                    self.active_pane_mut().navigate(&path);
                }
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_sort_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') => {
                self.active_pane_mut().sort_method = crate::entry::SortMethod::Name;
                self.active_pane_mut().refresh();
                self.mode = Mode::Normal;
            }
            KeyCode::Char('s') => {
                self.active_pane_mut().sort_method = crate::entry::SortMethod::Size;
                self.active_pane_mut().refresh();
                self.mode = Mode::Normal;
            }
            KeyCode::Char('d') => {
                self.active_pane_mut().sort_method = crate::entry::SortMethod::Modified;
                self.active_pane_mut().refresh();
                self.mode = Mode::Normal;
            }
            KeyCode::Char('t') => {
                self.active_pane_mut().sort_method = crate::entry::SortMethod::Type;
                self.active_pane_mut().refresh();
                self.mode = Mode::Normal;
            }
            KeyCode::Char('r') => {
                self.active_pane_mut().toggle_sort_direction();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn copy_to_other_pane(&mut self) {
        let files = self.active_pane().get_selected_files();
        if files.is_empty() {
            return;
        }

        let dest = self.inactive_pane().path.clone();

        for src in &files {
            let file_name = src.file_name().unwrap_or_default();
            let dest_path = dest.join(file_name);

            if src.is_dir() {
                if let Err(e) = copy_dir_recursive(src, &dest_path) {
                    self.message = Some(format!("Copy failed: {}", e));
                    return;
                }
            } else if let Err(e) = fs::copy(src, &dest_path) {
                self.message = Some(format!("Copy failed: {}", e));
                return;
            }
        }

        self.active_pane_mut().clear_selection();
        self.left_pane.refresh();
        self.right_pane.refresh();
        self.message = Some(format!("Copied {} item(s)", files.len()));
    }

    fn move_to_other_pane(&mut self) {
        let files = self.active_pane().get_selected_files();
        if files.is_empty() {
            return;
        }

        let dest = self.inactive_pane().path.clone();

        for src in &files {
            let file_name = src.file_name().unwrap_or_default();
            let dest_path = dest.join(file_name);

            if let Err(e) = fs::rename(src, &dest_path) {
                self.message = Some(format!("Move failed: {}", e));
                return;
            }
        }

        self.active_pane_mut().clear_selection();
        self.left_pane.refresh();
        self.right_pane.refresh();
        self.message = Some(format!("Moved {} item(s)", files.len()));
    }

    fn delete_selected(&mut self) {
        let files = self.active_pane().get_selected_files();
        if files.is_empty() {
            return;
        }

        if self.config.display.confirm_delete {
            self.mode = Mode::Confirm(ConfirmAction::Delete(files));
        } else {
            self.do_delete(&files);
        }
    }

    fn do_delete(&mut self, files: &[PathBuf]) {
        for path in files {
            let result = if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            };

            if let Err(e) = result {
                self.message = Some(format!("Delete failed: {}", e));
                return;
            }
        }

        self.active_pane_mut().clear_selection();
        self.left_pane.refresh();
        self.right_pane.refresh();
        self.message = Some(format!("Deleted {} item(s)", files.len()));
    }

    fn do_rename(&mut self, new_name: &str) {
        if new_name.is_empty() {
            return;
        }

        if let Some(entry) = self.active_pane().current_entry() {
            let old_path = entry.path.clone();
            let new_path = old_path.parent().unwrap().join(new_name);

            if let Err(e) = fs::rename(&old_path, &new_path) {
                self.message = Some(format!("Rename failed: {}", e));
            } else {
                self.left_pane.refresh();
                self.right_pane.refresh();
                self.message = Some(format!("Renamed to {}", new_name));
            }
        }
    }

    fn create_file(&mut self, name: &str) {
        if name.is_empty() {
            return;
        }

        let path = self.active_pane().path.join(name);

        if let Err(e) = fs::File::create(&path) {
            self.message = Some(format!("Create failed: {}", e));
        } else {
            self.active_pane_mut().refresh();
            self.message = Some(format!("Created {}", name));
        }
    }

    fn create_dir(&mut self, name: &str) {
        if name.is_empty() {
            return;
        }

        let path = self.active_pane().path.join(name);

        if let Err(e) = fs::create_dir(&path) {
            self.message = Some(format!("Create failed: {}", e));
        } else {
            self.active_pane_mut().refresh();
            self.message = Some(format!("Created {}", name));
        }
    }

    fn add_bookmark(&mut self) {
        let path = self.active_pane().path.display().to_string();
        let name = self.active_pane().path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bookmark")
            .to_string();

        self.config.bookmarks.insert(name.clone(), path.clone());
        let _ = self.config.save();
        self.message = Some(format!("Bookmarked: {}", name));
    }

    /// Get status line text
    pub fn status_text(&self) -> String {
        let pane = self.active_pane();
        let selected = pane.selection.len();
        let total = pane.entries.len().saturating_sub(1); // Exclude ".."

        if selected > 0 {
            let size = pane.selected_size();
            format!("{} selected, {}", selected, format_bytes(size))
        } else {
            format!("{} items", total)
        }
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}
