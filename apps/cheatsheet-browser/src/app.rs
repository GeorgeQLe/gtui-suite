//! Application state and logic.

use crate::cheatsheets::{bundled_cheatsheets, load_user_cheatsheets, CheatSheet};
use crate::config::Config;
use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct App {
    pub config: Config,
    pub cheatsheets: Vec<CheatSheet>,
    pub filtered: Vec<usize>,
    pub selected_index: usize,
    pub view: View,
    pub search: String,
    pub searching: bool,
    pub scroll_offset: usize,
    pub show_help: bool,
    matcher: SkimMatcherV2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();

        let mut cheatsheets = Vec::new();

        // Load bundled cheatsheets
        if config.sources.bundled {
            cheatsheets.extend(bundled_cheatsheets());
        }

        // Load user cheatsheets
        if let Some(path) = config.sources.user_path.as_ref() {
            cheatsheets.extend(load_user_cheatsheets(path));
        } else if let Some(path) = Config::user_cheatsheets_path() {
            cheatsheets.extend(load_user_cheatsheets(&path));
        }

        // Sort by category then topic
        cheatsheets.sort_by(|a, b| (&a.category, &a.topic).cmp(&(&b.category, &b.topic)));

        let filtered: Vec<usize> = (0..cheatsheets.len()).collect();

        Self {
            config,
            cheatsheets,
            filtered,
            selected_index: 0,
            view: View::List,
            search: String::new(),
            searching: false,
            scroll_offset: 0,
            show_help: false,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn selected_cheatsheet(&self) -> Option<&CheatSheet> {
        self.filtered
            .get(self.selected_index)
            .and_then(|&i| self.cheatsheets.get(i))
    }

    pub fn filtered_cheatsheets(&self) -> Vec<&CheatSheet> {
        self.filtered
            .iter()
            .filter_map(|&i| self.cheatsheets.get(i))
            .collect()
    }

    fn filter_cheatsheets(&mut self) {
        if self.search.is_empty() {
            self.filtered = (0..self.cheatsheets.len()).collect();
        } else {
            let mut matches: Vec<(usize, i64)> = self
                .cheatsheets
                .iter()
                .enumerate()
                .filter_map(|(i, sheet)| {
                    let topic_score = self.matcher.fuzzy_match(&sheet.topic, &self.search);
                    let cat_score = self.matcher.fuzzy_match(&sheet.category, &self.search);
                    let content_score = self.matcher.fuzzy_match(&sheet.content, &self.search);

                    let max_score = [topic_score, cat_score, content_score]
                        .into_iter()
                        .flatten()
                        .max();

                    max_score.map(|score| (i, score))
                })
                .collect();

            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = matches.into_iter().map(|(i, _)| i).collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered.len() {
            self.selected_index = 0;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.searching {
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    self.search.clear();
                    self.filter_cheatsheets();
                }
                KeyCode::Enter => {
                    self.searching = false;
                }
                KeyCode::Backspace => {
                    self.search.pop();
                    self.filter_cheatsheets();
                }
                KeyCode::Char(c) => {
                    self.search.push(c);
                    self.filter_cheatsheets();
                }
                _ => {}
            }
            return;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Detail => self.handle_detail_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.filtered.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.filtered.len() - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                self.selected_index = 0;
            }
            KeyCode::Char('G') => {
                if !self.filtered.is_empty() {
                    self.selected_index = self.filtered.len() - 1;
                }
            }
            KeyCode::Enter => {
                if self.selected_cheatsheet().is_some() {
                    self.view = View::Detail;
                    self.scroll_offset = 0;
                }
            }
            KeyCode::Char('/') => {
                self.searching = true;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Esc => {
                if !self.search.is_empty() {
                    self.search.clear();
                    self.filter_cheatsheets();
                }
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('d') => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
            }
            KeyCode::Char('u') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            KeyCode::Char('g') => {
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                if let Some(sheet) = self.selected_cheatsheet() {
                    let lines = sheet.content.lines().count();
                    self.scroll_offset = lines.saturating_sub(20);
                }
            }
            KeyCode::Char('n') => {
                // Next section (find next # heading)
                if let Some(sheet) = self.selected_cheatsheet() {
                    let lines: Vec<&str> = sheet.content.lines().collect();
                    for (i, line) in lines.iter().enumerate().skip(self.scroll_offset + 1) {
                        if line.starts_with('#') {
                            self.scroll_offset = i;
                            break;
                        }
                    }
                }
            }
            KeyCode::Char('p') => {
                // Previous section
                if let Some(sheet) = self.selected_cheatsheet() {
                    let lines: Vec<&str> = sheet.content.lines().collect();
                    for i in (0..self.scroll_offset).rev() {
                        if lines.get(i).map_or(false, |l| l.starts_with('#')) {
                            self.scroll_offset = i;
                            break;
                        }
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.view = View::List;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            _ => {}
        }
    }
}
