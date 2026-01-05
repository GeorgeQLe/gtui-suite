//! Application state and logic.

use crate::config::Config;
use crate::db::Database;
use crate::models::{DailyEntry, JournalStats, MonthCalendar};
use chrono::{Datelike, NaiveDate, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub db: Database,
    pub config: Config,
    pub calendar: MonthCalendar,
    pub selected_day: usize,
    pub current_entry: Option<DailyEntry>,
    pub recent_entries: Vec<DailyEntry>,
    pub stats: JournalStats,
    pub mode: Mode,
    pub pane: Pane,
    pub editor_content: Vec<String>,
    pub editor_cursor: (usize, usize),
    pub search_query: String,
    pub search_results: Vec<DailyEntry>,
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
    Calendar,
    Editor,
    List,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "journal.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;

        let today = Utc::now().date_naive();
        let mut calendar = MonthCalendar::new(today.year(), today.month());
        db.populate_calendar(&mut calendar)?;

        let selected_day = today.day() as usize - 1;
        let current_entry = db.get_or_create_entry(today).ok();
        let recent_entries = db.list_entries(20)?;
        let stats = db.get_stats()?;

        let editor_content = current_entry
            .as_ref()
            .map(|e| e.content.lines().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| vec![String::new()]);

        Ok(Self {
            db,
            config,
            calendar,
            selected_day,
            current_entry,
            recent_entries,
            stats,
            mode: Mode::Normal,
            pane: Pane::Calendar,
            editor_content,
            editor_cursor: (0, 0),
            search_query: String::new(),
            search_results: Vec::new(),
            message: None,
            show_help: false,
        })
    }

    pub fn can_quit(&self) -> bool {
        self.mode == Mode::Normal
    }

    pub fn refresh(&mut self) {
        let _ = self.db.populate_calendar(&mut self.calendar);
        self.recent_entries = self.db.list_entries(20).unwrap_or_default();
        self.stats = self.db.get_stats().unwrap_or_default();
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.message = None;

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.mode == Mode::Search {
            self.handle_search_key(key);
            return;
        }

        if self.mode == Mode::Editing {
            self.handle_editor_key(key);
            return;
        }

        match key.code {
            // Pane switching
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::Calendar => Pane::Editor,
                    Pane::Editor => Pane::List,
                    Pane::List => Pane::Calendar,
                };
            }

            // Calendar navigation
            KeyCode::Char('h') | KeyCode::Left => self.move_calendar(-1),
            KeyCode::Char('l') | KeyCode::Right => self.move_calendar(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_calendar(-7),
            KeyCode::Char('j') | KeyCode::Down => self.move_calendar(7),

            // Month navigation
            KeyCode::Char('[') => self.prev_month(),
            KeyCode::Char(']') => self.next_month(),

            // Go to today
            KeyCode::Char('t') => self.goto_today(),

            // Select day
            KeyCode::Enter => self.select_current_day(),

            // Edit mode
            KeyCode::Char('e') => {
                if self.current_entry.is_some() {
                    self.mode = Mode::Editing;
                    self.pane = Pane::Editor;
                }
            }

            // Search
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                self.search_query.clear();
                self.search_results.clear();
            }

            // Save
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current_entry();
            }

            // Help
            KeyCode::Char('?') => self.show_help = true,

            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.pane = Pane::Calendar;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current_entry();
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

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Enter => {
                if let Some(entry) = self.search_results.first() {
                    self.open_entry(entry.date);
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.perform_search();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.perform_search();
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

    fn move_calendar(&mut self, delta: i32) {
        let new_day = self.selected_day as i32 + delta;
        let days_in_month = self.calendar.days.len() as i32;

        if new_day < 0 {
            self.prev_month();
            self.selected_day = self.calendar.days.len() - 1;
        } else if new_day >= days_in_month {
            self.next_month();
            self.selected_day = 0;
        } else {
            self.selected_day = new_day as usize;
        }
    }

    fn prev_month(&mut self) {
        let (year, month) = if self.calendar.month == 1 {
            (self.calendar.year - 1, 12)
        } else {
            (self.calendar.year, self.calendar.month - 1)
        };
        self.calendar = MonthCalendar::new(year, month);
        let _ = self.db.populate_calendar(&mut self.calendar);
        self.selected_day = self.selected_day.min(self.calendar.days.len() - 1);
    }

    fn next_month(&mut self) {
        let (year, month) = if self.calendar.month == 12 {
            (self.calendar.year + 1, 1)
        } else {
            (self.calendar.year, self.calendar.month + 1)
        };
        self.calendar = MonthCalendar::new(year, month);
        let _ = self.db.populate_calendar(&mut self.calendar);
        self.selected_day = self.selected_day.min(self.calendar.days.len() - 1);
    }

    fn goto_today(&mut self) {
        let today = Utc::now().date_naive();
        self.calendar = MonthCalendar::new(today.year(), today.month());
        let _ = self.db.populate_calendar(&mut self.calendar);
        self.selected_day = today.day() as usize - 1;
        self.select_current_day();
    }

    fn select_current_day(&mut self) {
        if let Some(day) = self.calendar.days.get(self.selected_day) {
            self.open_entry(day.date);
        }
    }

    fn open_entry(&mut self, date: NaiveDate) {
        if let Ok(entry) = self.db.get_or_create_entry(date) {
            let content = if entry.content.is_empty() && self.config.template.use_template {
                let template = &self.config.template.daily_template;
                template.replace("{{date}}", &entry.formatted_date())
            } else {
                entry.content.clone()
            };

            self.editor_content = content.lines().map(|s| s.to_string()).collect();
            if self.editor_content.is_empty() {
                self.editor_content.push(String::new());
            }
            self.editor_cursor = (0, 0);
            self.current_entry = Some(entry);
            self.pane = Pane::Editor;
        }
    }

    fn save_current_entry(&mut self) {
        if let Some(entry) = &mut self.current_entry {
            entry.content = self.editor_content.join("\n");
            entry.update_word_count();
            let word_count = entry.word_count;
            if self.db.update_entry(entry).is_ok() {
                self.refresh();
                self.message = Some(format!("Saved ({} words)", word_count));
            }
        }
    }

    fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
        } else if let Ok(results) = self.db.search(&self.search_query) {
            self.search_results = results;
        }
    }

    pub fn selected_date(&self) -> Option<NaiveDate> {
        self.calendar.days.get(self.selected_day).map(|d| d.date)
    }
}
