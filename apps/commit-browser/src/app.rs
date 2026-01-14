use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
    Search,
}

pub struct App {
    pub commits: Vec<Commit>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub view: View,
    pub search_query: String,
    pub status_message: Option<String>,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            view: View::List,
            search_query: String::new(),
            status_message: None,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn load_commits(&mut self) {
        self.commits = create_demo_commits();
        self.update_filtered();
    }

    fn update_filtered(&mut self) {
        if self.search_query.is_empty() {
            self.filtered = (0..self.commits.len()).collect();
        } else {
            let mut scored: Vec<(usize, i64)> = self
                .commits
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    let msg_score = self.matcher.fuzzy_match(&c.message, &self.search_query);
                    let author_score = self.matcher.fuzzy_match(&c.author, &self.search_query);
                    let hash_score = self.matcher.fuzzy_match(&c.hash, &self.search_query);

                    let best = [msg_score, author_score, hash_score]
                        .into_iter()
                        .flatten()
                        .max();

                    best.map(|s| (i, s))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        }
        self.selected = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Detail => self.handle_detail_key(key),
            View::Search => self.handle_search_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.filtered.len().saturating_sub(1) {
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
                if let Some(commit) = self.selected_commit() {
                    self.status_message = Some(format!("Copied: {}", commit.hash));
                }
            }
            KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::Char('G') => {
                self.selected = self.filtered.len().saturating_sub(1);
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
                if let Some(commit) = self.selected_commit() {
                    self.status_message = Some(format!("Copied: {}", commit.hash));
                }
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

    pub fn selected_commit(&self) -> Option<&Commit> {
        self.filtered
            .get(self.selected)
            .and_then(|&i| self.commits.get(i))
    }

    pub fn visible_commits(&self) -> Vec<&Commit> {
        self.filtered
            .iter()
            .filter_map(|&i| self.commits.get(i))
            .collect()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List => format!(
                "{} commits | Enter:detail /:search y:copy",
                self.filtered.len()
            ),
            View::Detail => "y:copy hash Esc:back".to_string(),
            View::Search => format!("Search: {}_", self.search_query),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_commits() -> Vec<Commit> {
    vec![
        Commit {
            hash: "abc1234567890def".to_string(),
            short_hash: "abc1234".to_string(),
            author: "Alice".to_string(),
            date: Utc::now(),
            message: "feat: Add new user authentication system".to_string(),
            files_changed: 5,
            insertions: 150,
            deletions: 20,
        },
        Commit {
            hash: "def567890abcdef1".to_string(),
            short_hash: "def5678".to_string(),
            author: "Bob".to_string(),
            date: Utc::now(),
            message: "fix: Resolve memory leak in cache handler".to_string(),
            files_changed: 2,
            insertions: 10,
            deletions: 45,
        },
        Commit {
            hash: "ghi9abc0def12345".to_string(),
            short_hash: "ghi9abc".to_string(),
            author: "Charlie".to_string(),
            date: Utc::now(),
            message: "docs: Update README with new API examples".to_string(),
            files_changed: 1,
            insertions: 80,
            deletions: 5,
        },
        Commit {
            hash: "jkl0def123456789".to_string(),
            short_hash: "jkl0def".to_string(),
            author: "Diana".to_string(),
            date: Utc::now(),
            message: "refactor: Clean up database connection pool".to_string(),
            files_changed: 8,
            insertions: 200,
            deletions: 180,
        },
        Commit {
            hash: "mno3456789abcdef".to_string(),
            short_hash: "mno3456".to_string(),
            author: "Alice".to_string(),
            date: Utc::now(),
            message: "test: Add unit tests for auth module".to_string(),
            files_changed: 3,
            insertions: 300,
            deletions: 0,
        },
        Commit {
            hash: "pqr789abcdef0123".to_string(),
            short_hash: "pqr789a".to_string(),
            author: "Bob".to_string(),
            date: Utc::now(),
            message: "chore: Update dependencies".to_string(),
            files_changed: 1,
            insertions: 50,
            deletions: 40,
        },
    ]
}
