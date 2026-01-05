//! Application state and logic.

use crate::config::Config;
use crate::models::{NodeId, SearchResult, TreeItem};
use crate::storage::Storage;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub storage: Storage,
    pub config: Config,
    pub tree_items: Vec<TreeItem>,
    pub selected_index: usize,
    pub mode: Mode,
    pub pane: Pane,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub editor_content: Vec<String>,
    pub editor_cursor: (usize, usize), // (line, col)
    pub current_note: Option<NodeId>,
    pub search_results: Vec<SearchResult>,
    pub message: Option<String>,
    pub show_help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Editing,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Tree,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    NewNote,
    NewFolder,
    Rename,
    Search,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let storage = Storage::open(config.notes_dir.clone())?;
        let tree_items = storage.build_tree();

        Ok(Self {
            storage,
            config,
            tree_items,
            selected_index: 0,
            mode: Mode::Normal,
            pane: Pane::Tree,
            input_buffer: String::new(),
            input_mode: InputMode::None,
            editor_content: Vec::new(),
            editor_cursor: (0, 0),
            current_note: None,
            search_results: Vec::new(),
            message: None,
            show_help: false,
        })
    }

    pub fn can_quit(&self) -> bool {
        self.mode == Mode::Normal && self.input_mode == InputMode::None
    }

    pub fn refresh_tree(&mut self) {
        self.tree_items = self.storage.build_tree();
        if self.selected_index >= self.tree_items.len() && !self.tree_items.is_empty() {
            self.selected_index = self.tree_items.len() - 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.message = None;

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.input_mode != InputMode::None {
            self.handle_input_key(key);
            return;
        }

        if self.mode == Mode::Editing {
            self.handle_editor_key(key);
            return;
        }

        match key.code {
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') => self.selected_index = 0,
            KeyCode::Char('G') => {
                if !self.tree_items.is_empty() {
                    self.selected_index = self.tree_items.len() - 1;
                }
            }

            // Pane switching
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::Tree => Pane::Editor,
                    Pane::Editor => Pane::Tree,
                };
            }

            // Tree actions
            KeyCode::Enter => self.activate_selected(),
            KeyCode::Char('l') | KeyCode::Right => self.expand_or_enter(),
            KeyCode::Char('h') | KeyCode::Left => self.collapse_or_parent(),

            // Create
            KeyCode::Char('n') => {
                self.input_mode = InputMode::NewNote;
                self.input_buffer.clear();
            }
            KeyCode::Char('N') => {
                self.input_mode = InputMode::NewFolder;
                self.input_buffer.clear();
            }

            // Edit
            KeyCode::Char('r') => {
                if let Some(item) = self.tree_items.get(self.selected_index) {
                    self.input_mode = InputMode::Rename;
                    self.input_buffer = item.name.clone();
                }
            }
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('e') => {
                if self.current_note.is_some() {
                    self.mode = Mode::Editing;
                    self.pane = Pane::Editor;
                }
            }

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.input_buffer.clear();
                self.mode = Mode::Search;
            }

            // Refresh
            KeyCode::Char('R') => {
                if self.storage.refresh().is_ok() {
                    self.refresh_tree();
                    self.message = Some("Refreshed".to_string());
                }
            }

            // Help
            KeyCode::Char('?') => self.show_help = true,

            // Save
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current_note();
            }

            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_results.clear();
            }

            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::None;
                self.input_buffer.clear();
                if self.mode == Mode::Search {
                    self.mode = Mode::Normal;
                    self.search_results.clear();
                }
            }
            KeyCode::Enter => self.finish_input(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
                if self.input_mode == InputMode::Search {
                    self.perform_search();
                }
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                if self.input_mode == InputMode::Search {
                    self.perform_search();
                }
            }
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.pane = Pane::Tree;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current_note();
            }
            KeyCode::Up => {
                if self.editor_cursor.0 > 0 {
                    self.editor_cursor.0 -= 1;
                    self.clamp_cursor_col();
                }
            }
            KeyCode::Down => {
                if self.editor_cursor.0 < self.editor_content.len().saturating_sub(1) {
                    self.editor_cursor.0 += 1;
                    self.clamp_cursor_col();
                }
            }
            KeyCode::Left => {
                if self.editor_cursor.1 > 0 {
                    self.editor_cursor.1 -= 1;
                }
            }
            KeyCode::Right => {
                if let Some(line) = self.editor_content.get(self.editor_cursor.0) {
                    if self.editor_cursor.1 < line.len() {
                        self.editor_cursor.1 += 1;
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(line) = self.editor_content.get_mut(self.editor_cursor.0) {
                    let rest = line.split_off(self.editor_cursor.1);
                    self.editor_content.insert(self.editor_cursor.0 + 1, rest);
                    self.editor_cursor.0 += 1;
                    self.editor_cursor.1 = 0;
                }
            }
            KeyCode::Backspace => {
                if self.editor_cursor.1 > 0 {
                    if let Some(line) = self.editor_content.get_mut(self.editor_cursor.0) {
                        line.remove(self.editor_cursor.1 - 1);
                        self.editor_cursor.1 -= 1;
                    }
                } else if self.editor_cursor.0 > 0 {
                    let current = self.editor_content.remove(self.editor_cursor.0);
                    self.editor_cursor.0 -= 1;
                    if let Some(prev) = self.editor_content.get_mut(self.editor_cursor.0) {
                        self.editor_cursor.1 = prev.len();
                        prev.push_str(&current);
                    }
                }
            }
            KeyCode::Char(c) => {
                if self.editor_content.is_empty() {
                    self.editor_content.push(String::new());
                }
                if let Some(line) = self.editor_content.get_mut(self.editor_cursor.0) {
                    line.insert(self.editor_cursor.1, c);
                    self.editor_cursor.1 += 1;
                }
            }
            _ => {}
        }
    }

    fn clamp_cursor_col(&mut self) {
        if let Some(line) = self.editor_content.get(self.editor_cursor.0) {
            if self.editor_cursor.1 > line.len() {
                self.editor_cursor.1 = line.len();
            }
        }
    }

    fn finish_input(&mut self) {
        match self.input_mode {
            InputMode::NewNote => {
                if !self.input_buffer.is_empty() {
                    let parent_id = self.get_current_folder_id();
                    if let Ok(id) = self.storage.create_note(&self.input_buffer, &parent_id) {
                        self.refresh_tree();
                        self.open_note(&id);
                        self.message = Some("Note created".to_string());
                    }
                }
            }
            InputMode::NewFolder => {
                if !self.input_buffer.is_empty() {
                    let parent_id = self.get_current_folder_id();
                    if self.storage.create_folder(&self.input_buffer, &parent_id).is_ok() {
                        self.refresh_tree();
                        self.message = Some("Folder created".to_string());
                    }
                }
            }
            InputMode::Rename => {
                if !self.input_buffer.is_empty() {
                    if let Some(item) = self.tree_items.get(self.selected_index) {
                        let id = item.id.clone();
                        if self.storage.rename_node(&id, &self.input_buffer).is_ok() {
                            self.refresh_tree();
                            self.message = Some("Renamed".to_string());
                        }
                    }
                }
            }
            InputMode::Search => {
                if let Some(result) = self.search_results.first() {
                    let id = result.note_id.clone();
                    self.open_note(&id);
                }
                self.mode = Mode::Normal;
            }
            InputMode::None => {}
        }

        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }

    fn get_current_folder_id(&self) -> NodeId {
        if let Some(item) = self.tree_items.get(self.selected_index) {
            if item.is_folder {
                return item.id.clone();
            }
            // Find parent folder
            if let Some(note) = self.storage.get_note(&item.id) {
                return note.parent_id.clone();
            }
        }
        self.storage.root_id().clone()
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.tree_items.len();
        if len == 0 {
            return;
        }
        let new_idx = self.selected_index as i32 + delta;
        self.selected_index = new_idx.clamp(0, len as i32 - 1) as usize;
    }

    fn activate_selected(&mut self) {
        if let Some(item) = self.tree_items.get(self.selected_index) {
            let id = item.id.clone();
            if item.is_folder {
                self.storage.toggle_folder(&id);
                self.refresh_tree();
            } else {
                self.open_note(&id);
            }
        }
    }

    fn expand_or_enter(&mut self) {
        if let Some(item) = self.tree_items.get(self.selected_index) {
            let id = item.id.clone();
            if item.is_folder {
                if !item.expanded {
                    self.storage.expand_folder(&id);
                    self.refresh_tree();
                } else if self.selected_index + 1 < self.tree_items.len() {
                    self.selected_index += 1;
                }
            } else {
                self.open_note(&id);
            }
        }
    }

    fn collapse_or_parent(&mut self) {
        if let Some(item) = self.tree_items.get(self.selected_index) {
            let id = item.id.clone();
            if item.is_folder && item.expanded {
                self.storage.toggle_folder(&id);
                self.refresh_tree();
            } else {
                // Go to parent
                if item.depth > 1 && self.selected_index > 0 {
                    for i in (0..self.selected_index).rev() {
                        if self.tree_items[i].depth < item.depth && self.tree_items[i].is_folder {
                            self.selected_index = i;
                            break;
                        }
                    }
                }
            }
        }
    }

    fn open_note(&mut self, id: &NodeId) {
        if let Some(note) = self.storage.get_note(id) {
            self.current_note = Some(id.clone());
            self.editor_content = note.content.lines().map(|s| s.to_string()).collect();
            if self.editor_content.is_empty() {
                self.editor_content.push(String::new());
            }
            self.editor_cursor = (0, 0);
            self.pane = Pane::Editor;
        }
    }

    fn delete_selected(&mut self) {
        if let Some(item) = self.tree_items.get(self.selected_index) {
            let id = item.id.clone();
            if self.storage.delete_node(&id).is_ok() {
                if self.current_note.as_ref() == Some(&id) {
                    self.current_note = None;
                    self.editor_content.clear();
                }
                self.refresh_tree();
                self.message = Some("Deleted".to_string());
            }
        }
    }

    fn save_current_note(&mut self) {
        if let Some(id) = &self.current_note {
            let content = self.editor_content.join("\n");
            if self.storage.save_note(id, &content).is_ok() {
                self.message = Some("Saved".to_string());
            }
        }
    }

    fn perform_search(&mut self) {
        if self.input_buffer.is_empty() {
            self.search_results.clear();
        } else {
            self.search_results = self.storage.search(&self.input_buffer);
        }
    }

    pub fn selected_item(&self) -> Option<&TreeItem> {
        self.tree_items.get(self.selected_index)
    }

    pub fn current_note_title(&self) -> Option<String> {
        self.current_note.as_ref().and_then(|id| {
            self.storage.get_note(id).map(|n| n.title.clone())
        })
    }
}
