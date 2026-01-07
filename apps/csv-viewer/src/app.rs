use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;

use crate::data::CsvData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Table,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search(String),
    Filter(String),
    Sort,
}

pub struct App {
    pub data: CsvData,
    pub view: View,
    pub mode: Mode,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub filtered_rows: Option<Vec<usize>>,
    pub search_results: Vec<(usize, usize)>,
    pub search_index: usize,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            data: CsvData::new(),
            view: View::Table,
            mode: Mode::Normal,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            sort_column: None,
            sort_ascending: true,
            filtered_rows: None,
            search_results: Vec::new(),
            search_index: 0,
            message: None,
            error: None,
        }
    }

    pub fn load(&mut self, path: PathBuf) -> Result<()> {
        self.data = CsvData::load(path)?;
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_row = 0;
        self.scroll_col = 0;
        self.filtered_rows = None;
        Ok(())
    }

    pub fn visible_rows(&self) -> usize {
        match &self.filtered_rows {
            Some(indices) => indices.len(),
            None => self.data.row_count(),
        }
    }

    pub fn get_display_row(&self, index: usize) -> Option<usize> {
        match &self.filtered_rows {
            Some(indices) => indices.get(index).copied(),
            None => Some(index),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, visible_height: usize) -> bool {
        self.message = None;
        self.error = None;

        match &self.mode {
            Mode::Normal => self.handle_normal_key(key, visible_height),
            Mode::Search(_) => self.handle_search_key(key),
            Mode::Filter(_) => self.handle_filter_key(key),
            Mode::Sort => self.handle_sort_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent, visible_height: usize) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Esc => {
                if self.view == View::Help {
                    self.view = View::Table;
                } else {
                    self.filtered_rows = None;
                    self.search_results.clear();
                }
            }

            // Navigation
            KeyCode::Right | KeyCode::Char('l') => {
                if self.cursor_col < self.data.col_count().saturating_sub(1) {
                    self.cursor_col += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor_row < self.visible_rows().saturating_sub(1) {
                    self.cursor_row += 1;
                    self.ensure_visible(visible_height);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.ensure_visible(visible_height);
                }
            }
            KeyCode::PageDown => {
                self.cursor_row = (self.cursor_row + visible_height).min(self.visible_rows().saturating_sub(1));
                self.ensure_visible(visible_height);
            }
            KeyCode::PageUp => {
                self.cursor_row = self.cursor_row.saturating_sub(visible_height);
                self.ensure_visible(visible_height);
            }
            KeyCode::Home => {
                self.cursor_row = 0;
                self.scroll_row = 0;
            }
            KeyCode::End => {
                self.cursor_row = self.visible_rows().saturating_sub(1);
                self.ensure_visible(visible_height);
            }

            // Search
            KeyCode::Char('/') => {
                self.mode = Mode::Search(String::new());
            }

            // Filter
            KeyCode::Char('f') => {
                self.mode = Mode::Filter(String::new());
            }

            // Sort
            KeyCode::Char('s') => {
                self.mode = Mode::Sort;
                self.message = Some(format!("Sort by column {} (1-{}, Enter to confirm, a/d for asc/desc)",
                    self.cursor_col + 1, self.data.col_count()));
            }

            // Next/prev search result
            KeyCode::Char('n') => {
                if !self.search_results.is_empty() {
                    self.search_index = (self.search_index + 1) % self.search_results.len();
                    if let Some((row, col)) = self.search_results.get(self.search_index) {
                        self.cursor_row = *row;
                        self.cursor_col = *col;
                        self.ensure_visible(visible_height);
                    }
                }
            }
            KeyCode::Char('N') => {
                if !self.search_results.is_empty() {
                    self.search_index = if self.search_index == 0 {
                        self.search_results.len() - 1
                    } else {
                        self.search_index - 1
                    };
                    if let Some((row, col)) = self.search_results.get(self.search_index) {
                        self.cursor_row = *row;
                        self.cursor_col = *col;
                        self.ensure_visible(visible_height);
                    }
                }
            }

            _ => {}
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        if let Mode::Search(ref mut query) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let q = query.clone();
                    self.search_results = self.data.search(&q);
                    self.search_index = 0;
                    if let Some((row, col)) = self.search_results.first() {
                        self.cursor_row = *row;
                        self.cursor_col = *col;
                        self.message = Some(format!("Found {} matches", self.search_results.len()));
                    } else {
                        self.error = Some("No matches found".to_string());
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Backspace => { query.pop(); }
                KeyCode::Char(c) => query.push(c),
                _ => {}
            }
        }
        false
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> bool {
        if let Mode::Filter(ref mut query) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let q = query.clone();
                    self.filtered_rows = Some(self.data.filter(self.cursor_col, &q));
                    self.cursor_row = 0;
                    self.scroll_row = 0;
                    self.message = Some(format!("Filtered: {} rows", self.visible_rows()));
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Backspace => { query.pop(); }
                KeyCode::Char(c) => query.push(c),
                _ => {}
            }
        }
        false
    }

    fn handle_sort_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.data.sort_by_column(self.cursor_col, self.sort_ascending);
                self.sort_column = Some(self.cursor_col);
                self.message = Some(format!("Sorted by column {}", self.cursor_col + 1));
                self.mode = Mode::Normal;
            }
            KeyCode::Char('a') => {
                self.sort_ascending = true;
                self.message = Some("Ascending sort".to_string());
            }
            KeyCode::Char('d') => {
                self.sort_ascending = false;
                self.message = Some("Descending sort".to_string());
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let col = c.to_digit(10).unwrap() as usize;
                if col > 0 && col <= self.data.col_count() {
                    self.cursor_col = col - 1;
                }
            }
            KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }
        false
    }

    fn ensure_visible(&mut self, visible_height: usize) {
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + visible_height {
            self.scroll_row = self.cursor_row - visible_height + 1;
        }
    }
}
