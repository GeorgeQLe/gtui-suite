//! Application state and logic.

use crate::config::Config;
use crate::db::Database;
use crate::models::{LinkType, Zettel, ZettelType, ZkStats};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub db: Database,
    pub config: Config,
    pub zettels: Vec<Zettel>,
    pub selected_index: usize,
    pub current_zettel: Option<Zettel>,
    pub outgoing_links: Vec<(Zettel, LinkType)>,
    pub incoming_links: Vec<(Zettel, LinkType)>,
    pub stats: ZkStats,
    pub tags: Vec<(String, usize)>,
    pub mode: Mode,
    pub view: View,
    pub pane: Pane,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub editor_content: Vec<String>,
    pub editor_cursor: (usize, usize),
    pub search_results: Vec<Zettel>,
    pub filter_type: Option<ZettelType>,
    pub filter_tag: Option<String>,
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
    List,
    Tags,
    Types,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    List,
    Editor,
    Links,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    NewZettel,
    AddTag,
    AddLink,
    Search,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "zettelkasten.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;
        let zettels = db.list_zettels()?;
        let stats = db.get_stats()?;
        let tags = db.get_all_tags()?;

        Ok(Self {
            db,
            config,
            zettels,
            selected_index: 0,
            current_zettel: None,
            outgoing_links: Vec::new(),
            incoming_links: Vec::new(),
            stats,
            tags,
            mode: Mode::Normal,
            view: View::List,
            pane: Pane::List,
            input_buffer: String::new(),
            input_mode: InputMode::None,
            editor_content: Vec::new(),
            editor_cursor: (0, 0),
            search_results: Vec::new(),
            filter_type: None,
            filter_tag: None,
            message: None,
            show_help: false,
        })
    }

    pub fn can_quit(&self) -> bool {
        self.mode == Mode::Normal && self.input_mode == InputMode::None
    }

    pub fn refresh(&mut self) {
        self.zettels = if let Some(t) = self.filter_type {
            self.db.list_by_type(t).unwrap_or_default()
        } else if let Some(ref tag) = self.filter_tag {
            self.db.list_by_tag(tag).unwrap_or_default()
        } else {
            self.db.list_zettels().unwrap_or_default()
        };
        self.stats = self.db.get_stats().unwrap_or_default();
        self.tags = self.db.get_all_tags().unwrap_or_default();

        if self.selected_index >= self.zettels.len() && !self.zettels.is_empty() {
            self.selected_index = self.zettels.len() - 1;
        }
    }

    fn load_zettel_context(&mut self) {
        if let Some(z) = &self.current_zettel {
            self.outgoing_links = self.db.get_outgoing_links(&z.id).unwrap_or_default();
            self.incoming_links = self.db.get_incoming_links(&z.id).unwrap_or_default();
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
                if !self.zettels.is_empty() {
                    self.selected_index = self.zettels.len() - 1;
                }
            }

            // Pane/view switching
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::List => Pane::Editor,
                    Pane::Editor => Pane::Links,
                    Pane::Links => Pane::List,
                };
            }
            KeyCode::Char('1') => self.view = View::List,
            KeyCode::Char('2') => self.view = View::Tags,
            KeyCode::Char('3') => self.view = View::Types,

            // Actions
            KeyCode::Enter => self.open_selected(),
            KeyCode::Char('n') => {
                self.input_mode = InputMode::NewZettel;
                self.input_buffer.clear();
            }
            KeyCode::Char('e') => {
                if self.current_zettel.is_some() {
                    self.mode = Mode::Editing;
                    self.pane = Pane::Editor;
                }
            }
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('t') => {
                if self.current_zettel.is_some() {
                    self.input_mode = InputMode::AddTag;
                    self.input_buffer.clear();
                }
            }
            KeyCode::Char('l') => {
                if self.current_zettel.is_some() {
                    self.input_mode = InputMode::AddLink;
                    self.input_buffer.clear();
                }
            }
            KeyCode::Char('T') => self.cycle_zettel_type(),

            // Filtering
            KeyCode::Char('f') => self.toggle_type_filter(),
            KeyCode::Char('F') => {
                self.filter_type = None;
                self.filter_tag = None;
                self.refresh();
                self.message = Some("Filters cleared".to_string());
            }

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.input_buffer.clear();
                self.mode = Mode::Search;
            }

            // Save
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current_zettel();
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
                self.save_current_zettel();
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
            InputMode::NewZettel => {
                if !self.input_buffer.is_empty() {
                    let zettel = Zettel::new(&self.input_buffer, ZettelType::Fleeting);
                    if let Ok(db_id) = self.db.insert_zettel(&zettel) {
                        self.refresh();
                        self.open_zettel_by_db_id(db_id);
                        self.message = Some(format!("Created: {}", zettel.id));
                    }
                }
            }
            InputMode::AddTag => {
                if !self.input_buffer.is_empty() {
                    if let Some(z) = &mut self.current_zettel {
                        if !z.tags.contains(&self.input_buffer) {
                            z.tags.push(self.input_buffer.clone());
                            let _ = self.db.update_zettel(z);
                            self.refresh();
                            self.message = Some(format!("Added tag: {}", self.input_buffer));
                        }
                    }
                }
            }
            InputMode::AddLink => {
                if !self.input_buffer.is_empty() {
                    if let Some(z) = &self.current_zettel {
                        if self.db.get_zettel_by_id(&self.input_buffer).ok().flatten().is_some() {
                            let _ = self.db.add_link(&z.id, &self.input_buffer, LinkType::Reference);
                            self.load_zettel_context();
                            self.message = Some(format!("Linked to: {}", self.input_buffer));
                        } else {
                            self.message = Some("Zettel not found".to_string());
                        }
                    }
                }
            }
            InputMode::Search => {
                if let Some(z) = self.search_results.first() {
                    let db_id = z.db_id;
                    self.open_zettel_by_db_id(db_id);
                }
                self.mode = Mode::Normal;
            }
            InputMode::None => {}
        }

        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.zettels.len();
        if len == 0 { return; }
        let new_idx = self.selected_index as i32 + delta;
        self.selected_index = new_idx.clamp(0, len as i32 - 1) as usize;
    }

    fn open_selected(&mut self) {
        if let Some(z) = self.zettels.get(self.selected_index) {
            let db_id = z.db_id;
            self.open_zettel_by_db_id(db_id);
        }
    }

    fn open_zettel_by_db_id(&mut self, db_id: i64) {
        if let Ok(Some(z)) = self.db.get_zettel(db_id) {
            self.editor_content = z.content.lines().map(|s| s.to_string()).collect();
            if self.editor_content.is_empty() {
                self.editor_content.push(String::new());
            }
            self.editor_cursor = (0, 0);
            self.current_zettel = Some(z);
            self.load_zettel_context();
            self.pane = Pane::Editor;
        }
    }

    fn delete_selected(&mut self) {
        if let Some(z) = self.zettels.get(self.selected_index) {
            let db_id = z.db_id;
            if self.db.delete_zettel(db_id).is_ok() {
                if self.current_zettel.as_ref().map(|z| z.db_id) == Some(db_id) {
                    self.current_zettel = None;
                    self.editor_content.clear();
                    self.outgoing_links.clear();
                    self.incoming_links.clear();
                }
                self.refresh();
                self.message = Some("Deleted".to_string());
            }
        }
    }

    fn save_current_zettel(&mut self) {
        if let Some(z) = &mut self.current_zettel {
            z.content = self.editor_content.join("\n");
            if self.db.update_zettel(z).is_ok() {
                self.message = Some("Saved".to_string());
            }
        }
    }

    fn cycle_zettel_type(&mut self) {
        if let Some(z) = &mut self.current_zettel {
            z.zettel_type = match z.zettel_type {
                ZettelType::Fleeting => ZettelType::Literature,
                ZettelType::Literature => ZettelType::Permanent,
                ZettelType::Permanent => ZettelType::Hub,
                ZettelType::Hub => ZettelType::Fleeting,
            };
            let _ = self.db.update_zettel(z);
            let type_label = z.zettel_type.label().to_string();
            self.refresh();
            self.message = Some(format!("Type: {}", type_label));
        }
    }

    fn toggle_type_filter(&mut self) {
        self.filter_type = match self.filter_type {
            None => Some(ZettelType::Fleeting),
            Some(ZettelType::Fleeting) => Some(ZettelType::Literature),
            Some(ZettelType::Literature) => Some(ZettelType::Permanent),
            Some(ZettelType::Permanent) => Some(ZettelType::Hub),
            Some(ZettelType::Hub) => None,
        };
        self.refresh();
        let filter_name = self.filter_type.map(|t| t.label()).unwrap_or("All");
        self.message = Some(format!("Filter: {}", filter_name));
    }

    fn perform_search(&mut self) {
        if self.input_buffer.is_empty() {
            self.search_results.clear();
        } else if let Ok(results) = self.db.search(&self.input_buffer) {
            self.search_results = results;
        }
    }
}
