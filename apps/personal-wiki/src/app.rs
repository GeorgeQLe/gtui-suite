//! Application state and logic.

use crate::config::Config;
use crate::db::Database;
use crate::models::{Category, Page, Revision, WikiStats};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub db: Database,
    pub config: Config,
    pub pages: Vec<Page>,
    pub categories: Vec<Category>,
    pub selected_index: usize,
    pub current_page: Option<Page>,
    pub backlinks: Vec<Page>,
    pub revisions: Vec<Revision>,
    pub history: Vec<i64>,
    pub stats: WikiStats,
    pub mode: Mode,
    pub view: View,
    pub pane: Pane,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub editor_content: Vec<String>,
    pub editor_cursor: (usize, usize),
    pub search_results: Vec<Page>,
    pub wanted_pages: Vec<(String, usize)>,
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
pub enum View {
    AllPages,
    RecentChanges,
    Categories,
    Orphans,
    Wanted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    List,
    Editor,
    Sidebar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    NewPage,
    GoTo,
    Search,
    EditSummary,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "wiki.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;
        let pages = db.list_pages()?;
        let categories = db.list_categories()?;
        let stats = db.get_stats()?;

        Ok(Self {
            db,
            config,
            pages,
            categories,
            selected_index: 0,
            current_page: None,
            backlinks: Vec::new(),
            revisions: Vec::new(),
            history: Vec::new(),
            stats,
            mode: Mode::Normal,
            view: View::AllPages,
            pane: Pane::List,
            input_buffer: String::new(),
            input_mode: InputMode::None,
            editor_content: Vec::new(),
            editor_cursor: (0, 0),
            search_results: Vec::new(),
            wanted_pages: Vec::new(),
            message: None,
            show_help: false,
        })
    }

    pub fn can_quit(&self) -> bool {
        self.mode == Mode::Normal && self.input_mode == InputMode::None
    }

    pub fn refresh(&mut self) {
        match self.view {
            View::AllPages => {
                self.pages = self.db.list_pages().unwrap_or_default();
            }
            View::RecentChanges => {
                self.pages = self.db.list_recent_pages(self.config.display.recent_limit).unwrap_or_default();
            }
            View::Categories => {
                self.categories = self.db.list_categories().unwrap_or_default();
            }
            View::Orphans => {
                self.pages = self.db.get_orphan_pages().unwrap_or_default();
            }
            View::Wanted => {
                self.wanted_pages = self.db.get_wanted_pages().unwrap_or_default();
            }
        }
        self.stats = self.db.get_stats().unwrap_or_default();

        if self.selected_index >= self.pages.len() && !self.pages.is_empty() {
            self.selected_index = self.pages.len() - 1;
        }
    }

    fn load_page_context(&mut self) {
        if let Some(page) = &self.current_page {
            self.backlinks = self.db.get_backlinks(page.id).unwrap_or_default();
            self.revisions = self.db.get_revisions(page.id).unwrap_or_default();
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
                if !self.pages.is_empty() {
                    self.selected_index = self.pages.len() - 1;
                }
            }

            // Pane switching
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::List => Pane::Editor,
                    Pane::Editor => Pane::Sidebar,
                    Pane::Sidebar => Pane::List,
                };
            }

            // View switching
            KeyCode::Char('1') => { self.view = View::AllPages; self.refresh(); }
            KeyCode::Char('2') => { self.view = View::RecentChanges; self.refresh(); }
            KeyCode::Char('3') => { self.view = View::Categories; self.refresh(); }
            KeyCode::Char('4') => { self.view = View::Orphans; self.refresh(); }
            KeyCode::Char('5') => { self.view = View::Wanted; self.refresh(); }

            // Actions
            KeyCode::Enter => self.open_selected(),
            KeyCode::Char('n') => {
                self.input_mode = InputMode::NewPage;
                self.input_buffer.clear();
            }
            KeyCode::Char('o') => {
                self.input_mode = InputMode::GoTo;
                self.input_buffer.clear();
            }
            KeyCode::Char('e') => {
                if self.current_page.is_some() {
                    self.mode = Mode::Editing;
                    self.pane = Pane::Editor;
                }
            }
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('b') => self.go_back(),
            KeyCode::Char('r') => {
                if let Ok(Some(page)) = self.db.get_random_page() {
                    self.open_page(page.id);
                }
            }

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.input_buffer.clear();
                self.mode = Mode::Search;
            }

            // Save
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_mode = InputMode::EditSummary;
                self.input_buffer.clear();
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
                self.input_mode = InputMode::EditSummary;
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
            InputMode::NewPage => {
                if !self.input_buffer.is_empty() {
                    let page = Page::new(&self.input_buffer);
                    if let Ok(id) = self.db.insert_page(&page) {
                        self.refresh();
                        self.open_page(id);
                        self.message = Some(format!("Created: {}", self.input_buffer));
                    }
                }
            }
            InputMode::GoTo => {
                if !self.input_buffer.is_empty() {
                    if let Ok(Some(page)) = self.db.get_page_by_title(&self.input_buffer) {
                        self.open_page(page.id);
                    } else {
                        // Create new page
                        let page = Page::new(&self.input_buffer);
                        if let Ok(id) = self.db.insert_page(&page) {
                            self.refresh();
                            self.open_page(id);
                            self.message = Some(format!("Created: {}", self.input_buffer));
                        }
                    }
                }
            }
            InputMode::Search => {
                if let Some(page) = self.search_results.first() {
                    let id = page.id;
                    self.open_page(id);
                }
                self.mode = Mode::Normal;
            }
            InputMode::EditSummary => {
                self.save_current_page(&self.input_buffer.clone());
            }
            InputMode::None => {}
        }

        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.pages.len();
        if len == 0 { return; }
        let new_idx = self.selected_index as i32 + delta;
        self.selected_index = new_idx.clamp(0, len as i32 - 1) as usize;
    }

    fn open_selected(&mut self) {
        if let Some(page) = self.pages.get(self.selected_index) {
            let id = page.id;
            self.open_page(id);
        }
    }

    fn open_page(&mut self, id: i64) {
        // Push current page to history
        if let Some(current) = &self.current_page {
            self.history.push(current.id);
        }

        if let Ok(Some(page)) = self.db.get_page(id) {
            // Handle redirect
            if let Some(ref redirect_to) = page.redirect_to {
                if let Ok(Some(target)) = self.db.get_page_by_title(redirect_to) {
                    self.message = Some(format!("Redirected from: {}", page.title));
                    self.open_page(target.id);
                    return;
                }
            }

            self.editor_content = page.content.lines().map(|s| s.to_string()).collect();
            if self.editor_content.is_empty() {
                self.editor_content.push(String::new());
            }
            self.editor_cursor = (0, 0);
            self.current_page = Some(page);
            self.load_page_context();
            self.pane = Pane::Editor;
        }
    }

    fn go_back(&mut self) {
        if let Some(id) = self.history.pop() {
            if let Ok(Some(page)) = self.db.get_page(id) {
                self.editor_content = page.content.lines().map(|s| s.to_string()).collect();
                if self.editor_content.is_empty() {
                    self.editor_content.push(String::new());
                }
                self.editor_cursor = (0, 0);
                self.current_page = Some(page);
                self.load_page_context();
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(page) = self.pages.get(self.selected_index) {
            let id = page.id;
            if self.db.delete_page(id).is_ok() {
                if self.current_page.as_ref().map(|p| p.id) == Some(id) {
                    self.current_page = None;
                    self.editor_content.clear();
                    self.backlinks.clear();
                    self.revisions.clear();
                }
                self.refresh();
                self.message = Some("Deleted".to_string());
            }
        }
    }

    fn save_current_page(&mut self, summary: &str) {
        if let Some(page) = &mut self.current_page {
            page.content = self.editor_content.join("\n");
            if self.db.update_page(page, summary).is_ok() {
                self.load_page_context();
                self.refresh();
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
}
