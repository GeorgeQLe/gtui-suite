use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct ReflogEntry {
    pub index: usize,
    pub short_id: String,
    pub action: String,
    pub message: String,
    pub timestamp: String,
}

pub struct App {
    pub entries: Vec<ReflogEntry>,
    pub selected: usize,
    pub show_details: bool,
    pub search: String,
    pub searching: bool,
    pub filtered_indices: Vec<usize>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let entries = create_demo_reflog();
        let filtered_indices: Vec<usize> = (0..entries.len()).collect();
        Self {
            entries,
            selected: 0,
            show_details: false,
            search: String::new(),
            searching: false,
            filtered_indices,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        if self.searching {
            return self.handle_search_key(key);
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                } else {
                    return true;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.filtered_indices.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.selected = self.filtered_indices.len().saturating_sub(1);
            }
            KeyCode::Enter => {
                self.show_details = !self.show_details;
            }
            KeyCode::Char('/') => {
                self.searching = true;
                self.search.clear();
            }
            KeyCode::Char('r') => {
                if let Some(&idx) = self.filtered_indices.get(self.selected) {
                    if let Some(entry) = self.entries.get(idx) {
                        self.status_message = Some(format!(
                            "Would reset to HEAD@{{{}}}: {}",
                            entry.index, entry.short_id
                        ));
                    }
                }
            }
            KeyCode::Char('c') => {
                if let Some(&idx) = self.filtered_indices.get(self.selected) {
                    if let Some(entry) = self.entries.get(idx) {
                        self.status_message = Some(format!(
                            "Would checkout {}: {}",
                            entry.short_id, entry.message
                        ));
                    }
                }
            }
            KeyCode::Char('y') => {
                if let Some(&idx) = self.filtered_indices.get(self.selected) {
                    if let Some(entry) = self.entries.get(idx) {
                        self.status_message = Some(format!("Copied: {}", entry.short_id));
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.searching = false;
                self.search.clear();
                self.update_filter();
            }
            KeyCode::Enter => {
                self.searching = false;
            }
            KeyCode::Backspace => {
                self.search.pop();
                self.update_filter();
            }
            KeyCode::Char(c) => {
                self.search.push(c);
                self.update_filter();
            }
            _ => {}
        }
        false
    }

    fn update_filter(&mut self) {
        if self.search.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
        } else {
            let search_lower = self.search.to_lowercase();
            self.filtered_indices = self.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.message.to_lowercase().contains(&search_lower)
                        || e.action.to_lowercase().contains(&search_lower)
                        || e.short_id.to_lowercase().contains(&search_lower)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.selected = self.selected.min(self.filtered_indices.len().saturating_sub(1));
    }

    pub fn selected_entry(&self) -> Option<&ReflogEntry> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&idx| self.entries.get(idx))
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        if self.searching {
            return format!("Search: {}_ | Esc:cancel Enter:confirm", self.search);
        }
        if self.show_details {
            return "Esc:back r:reset c:checkout y:copy".to_string();
        }
        "j/k:nav /:search Enter:details r:reset c:checkout q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_reflog() -> Vec<ReflogEntry> {
    vec![
        ReflogEntry {
            index: 0,
            short_id: "a1b2c3d".to_string(),
            action: "commit".to_string(),
            message: "Add new feature implementation".to_string(),
            timestamp: "2 hours ago".to_string(),
        },
        ReflogEntry {
            index: 1,
            short_id: "e4f5g6h".to_string(),
            action: "commit".to_string(),
            message: "Fix bug in user authentication".to_string(),
            timestamp: "5 hours ago".to_string(),
        },
        ReflogEntry {
            index: 2,
            short_id: "i7j8k9l".to_string(),
            action: "checkout".to_string(),
            message: "moving from feature-branch to main".to_string(),
            timestamp: "1 day ago".to_string(),
        },
        ReflogEntry {
            index: 3,
            short_id: "m0n1o2p".to_string(),
            action: "merge".to_string(),
            message: "Merge branch 'feature-branch'".to_string(),
            timestamp: "1 day ago".to_string(),
        },
        ReflogEntry {
            index: 4,
            short_id: "q3r4s5t".to_string(),
            action: "reset".to_string(),
            message: "moving to HEAD~1".to_string(),
            timestamp: "2 days ago".to_string(),
        },
        ReflogEntry {
            index: 5,
            short_id: "u6v7w8x".to_string(),
            action: "commit (amend)".to_string(),
            message: "Update README with installation instructions".to_string(),
            timestamp: "2 days ago".to_string(),
        },
        ReflogEntry {
            index: 6,
            short_id: "y9z0a1b".to_string(),
            action: "rebase".to_string(),
            message: "rebase finished: returning to refs/heads/main".to_string(),
            timestamp: "3 days ago".to_string(),
        },
        ReflogEntry {
            index: 7,
            short_id: "c2d3e4f".to_string(),
            action: "commit".to_string(),
            message: "Initial commit".to_string(),
            timestamp: "1 week ago".to_string(),
        },
    ]
}
