//! Application state and logic.

use crate::config::Config;
use crate::db::Database;
use crate::models::{Note, NoteId, SearchResult, ViewMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub db: Database,
    pub config: Config,
    pub notes: Vec<Note>,
    pub selected_index: usize,
    pub current_note: Option<Note>,
    pub backlinks: Vec<Note>,
    pub forward_links: Vec<(String, Option<Note>)>,
    pub mode: Mode,
    pub view: ViewMode,
    pub pane: Pane,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub editor_content: Vec<String>,
    pub editor_cursor: (usize, usize),
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
    List,
    Editor,
    Backlinks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    NewNote,
    Rename,
    Search,
    InsertLink,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "notes.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;
        let notes = db.list_notes()?;

        Ok(Self {
            db,
            config,
            notes,
            selected_index: 0,
            current_note: None,
            backlinks: Vec::new(),
            forward_links: Vec::new(),
            mode: Mode::Normal,
            view: ViewMode::List,
            pane: Pane::List,
            input_buffer: String::new(),
            input_mode: InputMode::None,
            editor_content: Vec::new(),
            editor_cursor: (0, 0),
            search_results: Vec::new(),
            message: None,
            show_help: false,
        })
    }

    pub fn can_quit(&self) -> bool {
        self.mode == Mode::Normal && self.input_mode == InputMode::None
    }

    pub fn refresh(&mut self) {
        if let Ok(notes) = self.db.list_notes() {
            self.notes = notes;
        }
        if self.selected_index >= self.notes.len() && !self.notes.is_empty() {
            self.selected_index = self.notes.len() - 1;
        }
    }

    fn load_note_context(&mut self) {
        if let Some(note) = &self.current_note {
            self.backlinks = self.db.get_backlinks(note.id).unwrap_or_default();
            self.forward_links = self.db.get_forward_links(note.id).unwrap_or_default();
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
                if !self.notes.is_empty() {
                    self.selected_index = self.notes.len() - 1;
                }
            }

            // Pane switching
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::List => Pane::Editor,
                    Pane::Editor => Pane::Backlinks,
                    Pane::Backlinks => Pane::List,
                };
            }

            // View switching
            KeyCode::Char('1') => self.view = ViewMode::List,
            KeyCode::Char('2') => self.view = ViewMode::Backlinks,
            KeyCode::Char('3') => self.view = ViewMode::Graph,

            // Actions
            KeyCode::Enter => self.open_selected(),
            KeyCode::Char('n') => {
                self.input_mode = InputMode::NewNote;
                self.input_buffer.clear();
            }
            KeyCode::Char('e') => {
                if self.current_note.is_some() {
                    self.mode = Mode::Editing;
                    self.pane = Pane::Editor;
                }
            }
            KeyCode::Char('r') => {
                if let Some(note) = &self.current_note {
                    self.input_mode = InputMode::Rename;
                    self.input_buffer = note.title.clone();
                }
            }
            KeyCode::Char('d') => self.delete_selected(),

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.input_buffer.clear();
                self.mode = Mode::Search;
            }

            // Save
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current_note();
            }

            // Help
            KeyCode::Char('?') => self.show_help = true,

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
                self.pane = Pane::List;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current_note();
            }
            KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Quick insert link
                self.input_mode = InputMode::InsertLink;
                self.input_buffer.clear();
            }
            KeyCode::Up => {
                if self.editor_cursor.0 > 0 {
                    self.editor_cursor.0 -= 1;
                    self.clamp_cursor();
                }
            }
            KeyCode::Down => {
                if self.editor_cursor.0 < self.editor_content.len().saturating_sub(1) {
                    self.editor_cursor.0 += 1;
                    self.clamp_cursor();
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

    fn clamp_cursor(&mut self) {
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
                    let note = Note::new(&self.input_buffer);
                    if let Ok(id) = self.db.insert_note(&note) {
                        self.refresh();
                        self.open_note_by_id(id);
                        self.message = Some("Note created".to_string());
                    }
                }
            }
            InputMode::Rename => {
                if !self.input_buffer.is_empty() {
                    if let Some(note) = &mut self.current_note {
                        note.title = self.input_buffer.clone();
                        let _ = self.db.update_note(note);
                        self.refresh();
                        self.message = Some("Renamed".to_string());
                    }
                }
            }
            InputMode::Search => {
                if let Some(result) = self.search_results.first() {
                    self.open_note_by_id(result.note_id);
                }
                self.mode = Mode::Normal;
            }
            InputMode::InsertLink => {
                if !self.input_buffer.is_empty() {
                    let link = format!("[[{}]]", self.input_buffer);
                    if let Some(line) = self.editor_content.get_mut(self.editor_cursor.0) {
                        line.insert_str(self.editor_cursor.1, &link);
                        self.editor_cursor.1 += link.len();
                    }
                }
            }
            InputMode::None => {}
        }

        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }

    fn move_selection(&mut self, delta: i32) {
        match self.pane {
            Pane::List => {
                let len = self.notes.len();
                if len == 0 { return; }
                let new_idx = self.selected_index as i32 + delta;
                self.selected_index = new_idx.clamp(0, len as i32 - 1) as usize;
            }
            Pane::Backlinks => {
                // Could implement backlinks navigation here
            }
            Pane::Editor => {}
        }
    }

    fn open_selected(&mut self) {
        if let Some(note) = self.notes.get(self.selected_index) {
            let id = note.id;
            self.open_note_by_id(id);
        }
    }

    fn open_note_by_id(&mut self, id: NoteId) {
        if let Ok(Some(note)) = self.db.get_note(id) {
            self.editor_content = note.content.lines().map(|s| s.to_string()).collect();
            if self.editor_content.is_empty() {
                self.editor_content.push(String::new());
            }
            self.editor_cursor = (0, 0);
            self.current_note = Some(note);
            self.load_note_context();
            self.pane = Pane::Editor;
        }
    }

    pub fn open_note_by_title(&mut self, title: &str) {
        if let Ok(Some(note)) = self.db.get_note_by_title(title) {
            let id = note.id;
            self.open_note_by_id(id);
        } else {
            // Create new note with this title
            let note = Note::new(title);
            if let Ok(id) = self.db.insert_note(&note) {
                self.refresh();
                self.open_note_by_id(id);
                self.message = Some(format!("Created note: {}", title));
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(note) = self.notes.get(self.selected_index) {
            let id = note.id;
            if self.db.delete_note(id).is_ok() {
                if self.current_note.as_ref().map(|n| n.id) == Some(id) {
                    self.current_note = None;
                    self.editor_content.clear();
                    self.backlinks.clear();
                    self.forward_links.clear();
                }
                self.refresh();
                self.message = Some("Deleted".to_string());
            }
        }
    }

    fn save_current_note(&mut self) {
        if let Some(note) = &mut self.current_note {
            note.content = self.editor_content.join("\n");
            if self.db.update_note(note).is_ok() {
                self.load_note_context();
                self.message = Some("Saved".to_string());
            }
        }
    }

    fn perform_search(&mut self) {
        if self.input_buffer.is_empty() {
            self.search_results.clear();
        } else if let Ok(results) = self.db.search(&self.input_buffer) {
            self.search_results = results;
        }
    }

    pub fn note_count(&self) -> usize {
        self.db.note_count().unwrap_or(0)
    }

    pub fn link_count(&self) -> usize {
        self.db.link_count().unwrap_or(0)
    }
}
