use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub added: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Folder {
    pub name: String,
    pub bookmarks: Vec<usize>,
    pub expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Folders,
    Bookmarks,
    Search,
}

pub struct App {
    pub bookmarks: Vec<Bookmark>,
    pub folders: Vec<Folder>,
    pub selected_folder: usize,
    pub selected_bookmark: usize,
    pub view: View,
    pub search_query: String,
    pub filtered_bookmarks: Vec<usize>,
    pub status_message: Option<String>,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Self {
        Self {
            bookmarks: Vec::new(),
            folders: Vec::new(),
            selected_folder: 0,
            selected_bookmark: 0,
            view: View::Folders,
            search_query: String::new(),
            filtered_bookmarks: Vec::new(),
            status_message: None,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn load_bookmarks(&mut self) {
        self.bookmarks = create_demo_bookmarks();
        self.folders = create_demo_folders(&self.bookmarks);
        self.update_filtered();
    }

    fn update_filtered(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_bookmarks = (0..self.bookmarks.len()).collect();
        } else {
            let mut scored: Vec<(usize, i64)> = self
                .bookmarks
                .iter()
                .enumerate()
                .filter_map(|(i, b)| {
                    let title_score = self.matcher.fuzzy_match(&b.title, &self.search_query);
                    let url_score = self.matcher.fuzzy_match(&b.url, &self.search_query);
                    let tag_score = b
                        .tags
                        .iter()
                        .filter_map(|t| self.matcher.fuzzy_match(t, &self.search_query))
                        .max();

                    let best = [title_score, url_score, tag_score]
                        .into_iter()
                        .flatten()
                        .max();

                    best.map(|s| (i, s))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_bookmarks = scored.into_iter().map(|(i, _)| i).collect();
        }
        self.selected_bookmark = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::Folders => self.handle_folders_key(key),
            View::Bookmarks => self.handle_bookmarks_key(key),
            View::Search => self.handle_search_key(key),
        }
    }

    fn handle_folders_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_folder < self.folders.len().saturating_sub(1) {
                    self.selected_folder += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_folder = self.selected_folder.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                self.view = View::Bookmarks;
                self.selected_bookmark = 0;
            }
            KeyCode::Char('/') => {
                self.view = View::Search;
                self.search_query.clear();
            }
            KeyCode::Tab => {
                self.view = View::Bookmarks;
            }
            _ => {}
        }
        false
    }

    fn handle_bookmarks_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                self.view = View::Folders;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let visible = self.visible_bookmarks();
                if self.selected_bookmark < visible.len().saturating_sub(1) {
                    self.selected_bookmark += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_bookmark = self.selected_bookmark.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(bookmark) = self.selected_bookmark() {
                    self.status_message = Some(format!("Opening: {}", bookmark.url));
                }
            }
            KeyCode::Char('y') => {
                if let Some(bookmark) = self.selected_bookmark() {
                    self.status_message = Some(format!("Copied: {}", bookmark.url));
                }
            }
            KeyCode::Char('/') => {
                self.view = View::Search;
                self.search_query.clear();
            }
            KeyCode::Tab => {
                self.view = View::Folders;
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::Bookmarks;
                self.search_query.clear();
                self.update_filtered();
            }
            KeyCode::Enter => {
                self.view = View::Bookmarks;
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

    pub fn visible_bookmarks(&self) -> Vec<&Bookmark> {
        if self.view == View::Search || !self.search_query.is_empty() {
            self.filtered_bookmarks
                .iter()
                .filter_map(|&i| self.bookmarks.get(i))
                .collect()
        } else if let Some(folder) = self.folders.get(self.selected_folder) {
            folder
                .bookmarks
                .iter()
                .filter_map(|&i| self.bookmarks.get(i))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn selected_bookmark(&self) -> Option<&Bookmark> {
        let visible = self.visible_bookmarks();
        visible.get(self.selected_bookmark).copied()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Folders => "Enter:open /:search Tab:switch".to_string(),
            View::Bookmarks => "Enter:open y:copy /:search Esc:back".to_string(),
            View::Search => format!("Search: {}_", self.search_query),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_bookmarks() -> Vec<Bookmark> {
    vec![
        Bookmark {
            title: "Rust Programming Language".to_string(),
            url: "https://www.rust-lang.org/".to_string(),
            folder: "Programming".to_string(),
            tags: vec!["rust".to_string(), "language".to_string()],
            added: Utc::now(),
        },
        Bookmark {
            title: "GitHub".to_string(),
            url: "https://github.com/".to_string(),
            folder: "Development".to_string(),
            tags: vec!["git".to_string(), "code".to_string()],
            added: Utc::now(),
        },
        Bookmark {
            title: "Hacker News".to_string(),
            url: "https://news.ycombinator.com/".to_string(),
            folder: "News".to_string(),
            tags: vec!["news".to_string(), "tech".to_string()],
            added: Utc::now(),
        },
        Bookmark {
            title: "Reddit".to_string(),
            url: "https://www.reddit.com/".to_string(),
            folder: "Social".to_string(),
            tags: vec!["social".to_string(), "forum".to_string()],
            added: Utc::now(),
        },
        Bookmark {
            title: "Stack Overflow".to_string(),
            url: "https://stackoverflow.com/".to_string(),
            folder: "Development".to_string(),
            tags: vec!["qa".to_string(), "programming".to_string()],
            added: Utc::now(),
        },
        Bookmark {
            title: "MDN Web Docs".to_string(),
            url: "https://developer.mozilla.org/".to_string(),
            folder: "Documentation".to_string(),
            tags: vec!["web".to_string(), "docs".to_string()],
            added: Utc::now(),
        },
        Bookmark {
            title: "Crates.io".to_string(),
            url: "https://crates.io/".to_string(),
            folder: "Programming".to_string(),
            tags: vec!["rust".to_string(), "packages".to_string()],
            added: Utc::now(),
        },
        Bookmark {
            title: "Docs.rs".to_string(),
            url: "https://docs.rs/".to_string(),
            folder: "Documentation".to_string(),
            tags: vec!["rust".to_string(), "docs".to_string()],
            added: Utc::now(),
        },
    ]
}

fn create_demo_folders(bookmarks: &[Bookmark]) -> Vec<Folder> {
    let mut folder_map: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();

    for (i, bookmark) in bookmarks.iter().enumerate() {
        folder_map
            .entry(bookmark.folder.clone())
            .or_default()
            .push(i);
    }

    folder_map
        .into_iter()
        .map(|(name, bookmarks)| Folder {
            name,
            bookmarks,
            expanded: true,
        })
        .collect()
}
