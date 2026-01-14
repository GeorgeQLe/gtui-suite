use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Search,
    Detail,
}

pub struct App {
    pub entries: Vec<HistoryEntry>,
    pub filtered_entries: Vec<usize>,
    pub selected: usize,
    pub view: View,
    pub search_query: String,
    pub scroll_offset: usize,
    pub status_message: Option<String>,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            selected: 0,
            view: View::List,
            search_query: String::new(),
            scroll_offset: 0,
            status_message: None,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn load_history(&mut self) {
        self.entries = create_demo_history();
        self.update_filtered();
    }

    fn update_filtered(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_entries = (0..self.entries.len()).collect();
        } else {
            let mut scored: Vec<(usize, i64)> = self
                .entries
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    self.matcher
                        .fuzzy_match(&e.command, &self.search_query)
                        .map(|s| (i, s))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_entries = scored.into_iter().map(|(i, _)| i).collect();
        }
        self.selected = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Search => self.handle_search_key(key),
            View::Detail => self.handle_detail_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.filtered_entries.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.view = View::Detail;
            }
            KeyCode::Char('/') => {
                self.view = View::Search;
            }
            KeyCode::Char('y') => {
                if let Some(entry) = self.selected_entry() {
                    self.status_message = Some(format!("Copied: {}", entry.command));
                }
            }
            KeyCode::Char('r') => {
                if let Some(entry) = self.selected_entry() {
                    self.status_message = Some(format!("Run: {}", entry.command));
                }
            }
            KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::Char('G') => {
                self.selected = self.filtered_entries.len().saturating_sub(1);
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
                self.search_query.clear();
                self.update_filtered();
            }
            KeyCode::Enter => {
                self.view = View::List;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_filtered();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_filtered();
            }
            _ => {}
        }
        false
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::List;
            }
            KeyCode::Char('y') => {
                if let Some(entry) = self.selected_entry() {
                    self.status_message = Some(format!("Copied: {}", entry.command));
                }
            }
            _ => {}
        }
        false
    }

    pub fn selected_entry(&self) -> Option<&HistoryEntry> {
        self.filtered_entries
            .get(self.selected)
            .and_then(|&i| self.entries.get(i))
    }

    pub fn visible_entries(&self) -> Vec<&HistoryEntry> {
        self.filtered_entries
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .collect()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List => format!(
                "{} commands | Enter:detail y:copy r:run /:search",
                self.filtered_entries.len()
            ),
            View::Search => format!("Search: {}_", self.search_query),
            View::Detail => "y:copy Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_history() -> Vec<HistoryEntry> {
    vec![
        HistoryEntry {
            command: "git status".to_string(),
            timestamp: Some(Utc::now()),
            count: 42,
        },
        HistoryEntry {
            command: "git add .".to_string(),
            timestamp: Some(Utc::now()),
            count: 38,
        },
        HistoryEntry {
            command: "git commit -m \"update\"".to_string(),
            timestamp: Some(Utc::now()),
            count: 35,
        },
        HistoryEntry {
            command: "cargo build".to_string(),
            timestamp: Some(Utc::now()),
            count: 30,
        },
        HistoryEntry {
            command: "cargo test".to_string(),
            timestamp: Some(Utc::now()),
            count: 28,
        },
        HistoryEntry {
            command: "cargo run".to_string(),
            timestamp: Some(Utc::now()),
            count: 25,
        },
        HistoryEntry {
            command: "ls -la".to_string(),
            timestamp: Some(Utc::now()),
            count: 50,
        },
        HistoryEntry {
            command: "cd ..".to_string(),
            timestamp: Some(Utc::now()),
            count: 45,
        },
        HistoryEntry {
            command: "vim main.rs".to_string(),
            timestamp: Some(Utc::now()),
            count: 20,
        },
        HistoryEntry {
            command: "docker ps".to_string(),
            timestamp: Some(Utc::now()),
            count: 15,
        },
        HistoryEntry {
            command: "docker-compose up -d".to_string(),
            timestamp: Some(Utc::now()),
            count: 12,
        },
        HistoryEntry {
            command: "ssh server".to_string(),
            timestamp: Some(Utc::now()),
            count: 10,
        },
        HistoryEntry {
            command: "cat /etc/hosts".to_string(),
            timestamp: Some(Utc::now()),
            count: 8,
        },
        HistoryEntry {
            command: "grep -r \"TODO\" .".to_string(),
            timestamp: Some(Utc::now()),
            count: 6,
        },
        HistoryEntry {
            command: "htop".to_string(),
            timestamp: Some(Utc::now()),
            count: 5,
        },
    ]
}
