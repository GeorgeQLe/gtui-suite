use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Feed {
    pub title: String,
    pub url: String,
    pub unread_count: usize,
    pub articles: Vec<Article>,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub published: DateTime<Utc>,
    pub summary: String,
    pub content: String,
    pub read: bool,
    pub starred: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Feeds,
    Articles,
    Reader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Unread,
    Starred,
}

pub struct App {
    pub feeds: Vec<Feed>,
    pub selected_feed: usize,
    pub selected_article: usize,
    pub view: View,
    pub filter: Filter,
    pub scroll_offset: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            feeds: Vec::new(),
            selected_feed: 0,
            selected_article: 0,
            view: View::Feeds,
            filter: Filter::All,
            scroll_offset: 0,
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        self.feeds = create_demo_feeds();
    }

    pub fn filtered_articles(&self) -> Vec<&Article> {
        if let Some(feed) = self.feeds.get(self.selected_feed) {
            feed.articles
                .iter()
                .filter(|a| match self.filter {
                    Filter::All => true,
                    Filter::Unread => !a.read,
                    Filter::Starred => a.starred,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::Feeds => self.handle_feeds_key(key),
            View::Articles => self.handle_articles_key(key),
            View::Reader => self.handle_reader_key(key),
        }
    }

    fn handle_feeds_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_feed < self.feeds.len().saturating_sub(1) {
                    self.selected_feed += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_feed = self.selected_feed.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if !self.feeds.is_empty() {
                    self.view = View::Articles;
                    self.selected_article = 0;
                }
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing feeds...".to_string());
            }
            KeyCode::Char('a') => {
                self.mark_all_read();
            }
            _ => {}
        }
        false
    }

    fn handle_articles_key(&mut self, key: KeyEvent) -> bool {
        let filtered = self.filtered_articles();
        let max_idx = filtered.len().saturating_sub(1);

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => {
                self.view = View::Feeds;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_article < max_idx {
                    self.selected_article += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_article = self.selected_article.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if !filtered.is_empty() {
                    self.view = View::Reader;
                    self.scroll_offset = 0;
                    self.mark_current_read();
                }
            }
            KeyCode::Char('m') => {
                self.toggle_read();
            }
            KeyCode::Char('s') => {
                self.toggle_starred();
            }
            KeyCode::Char('f') => {
                self.cycle_filter();
            }
            KeyCode::Char('o') => {
                self.status_message = Some("Opening in browser...".to_string());
            }
            _ => {}
        }
        false
    }

    fn handle_reader_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => {
                self.view = View::Articles;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_offset += 1;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('d') => {
                self.scroll_offset += 10;
            }
            KeyCode::Char('u') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            KeyCode::Char('s') => {
                self.toggle_starred();
            }
            KeyCode::Char('n') => {
                self.next_article();
            }
            KeyCode::Char('p') => {
                self.prev_article();
            }
            _ => {}
        }
        false
    }

    fn mark_current_read(&mut self) {
        if let Some(feed) = self.feeds.get_mut(self.selected_feed) {
            let filtered_indices: Vec<usize> = feed
                .articles
                .iter()
                .enumerate()
                .filter(|(_, a)| match self.filter {
                    Filter::All => true,
                    Filter::Unread => !a.read,
                    Filter::Starred => a.starred,
                })
                .map(|(i, _)| i)
                .collect();

            if let Some(&idx) = filtered_indices.get(self.selected_article) {
                if let Some(article) = feed.articles.get_mut(idx) {
                    if !article.read {
                        article.read = true;
                        feed.unread_count = feed.unread_count.saturating_sub(1);
                    }
                }
            }
        }
    }

    fn toggle_read(&mut self) {
        if let Some(feed) = self.feeds.get_mut(self.selected_feed) {
            let filtered_indices: Vec<usize> = feed
                .articles
                .iter()
                .enumerate()
                .filter(|(_, a)| match self.filter {
                    Filter::All => true,
                    Filter::Unread => !a.read,
                    Filter::Starred => a.starred,
                })
                .map(|(i, _)| i)
                .collect();

            if let Some(&idx) = filtered_indices.get(self.selected_article) {
                if let Some(article) = feed.articles.get_mut(idx) {
                    article.read = !article.read;
                    if article.read {
                        feed.unread_count = feed.unread_count.saturating_sub(1);
                    } else {
                        feed.unread_count += 1;
                    }
                }
            }
        }
    }

    fn toggle_starred(&mut self) {
        if let Some(feed) = self.feeds.get_mut(self.selected_feed) {
            let filtered_indices: Vec<usize> = feed
                .articles
                .iter()
                .enumerate()
                .filter(|(_, a)| match self.filter {
                    Filter::All => true,
                    Filter::Unread => !a.read,
                    Filter::Starred => a.starred,
                })
                .map(|(i, _)| i)
                .collect();

            if let Some(&idx) = filtered_indices.get(self.selected_article) {
                if let Some(article) = feed.articles.get_mut(idx) {
                    article.starred = !article.starred;
                }
            }
        }
    }

    fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            Filter::All => Filter::Unread,
            Filter::Unread => Filter::Starred,
            Filter::Starred => Filter::All,
        };
        self.selected_article = 0;
    }

    fn mark_all_read(&mut self) {
        if let Some(feed) = self.feeds.get_mut(self.selected_feed) {
            for article in &mut feed.articles {
                article.read = true;
            }
            feed.unread_count = 0;
            self.status_message = Some("Marked all as read".to_string());
        }
    }

    fn next_article(&mut self) {
        let filtered = self.filtered_articles();
        if self.selected_article < filtered.len().saturating_sub(1) {
            self.selected_article += 1;
            self.scroll_offset = 0;
            self.mark_current_read();
        }
    }

    fn prev_article(&mut self) {
        if self.selected_article > 0 {
            self.selected_article -= 1;
            self.scroll_offset = 0;
            self.mark_current_read();
        }
    }

    pub fn current_article(&self) -> Option<&Article> {
        let filtered = self.filtered_articles();
        filtered.get(self.selected_article).copied()
    }

    pub fn total_unread(&self) -> usize {
        self.feeds.iter().map(|f| f.unread_count).sum()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Feeds => format!(
                "{} feeds, {} unread | Enter:open r:refresh a:mark all read",
                self.feeds.len(),
                self.total_unread()
            ),
            View::Articles => format!(
                "Filter: {:?} | m:toggle read s:star f:filter o:open in browser",
                self.filter
            ),
            View::Reader => "j/k:scroll n/p:next/prev s:star Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_feeds() -> Vec<Feed> {
    vec![
        Feed {
            title: "Rust Blog".to_string(),
            url: "https://blog.rust-lang.org/feed.xml".to_string(),
            unread_count: 3,
            articles: vec![
                Article {
                    title: "Announcing Rust 1.75".to_string(),
                    link: "https://blog.rust-lang.org/2024/01/01/Rust-1.75.html".to_string(),
                    published: Utc::now() - chrono::Duration::hours(5),
                    summary: "The Rust team is happy to announce a new version of Rust, 1.75.0.".to_string(),
                    content: "The Rust team is happy to announce a new version of Rust, 1.75.0.\n\nThis release includes several new features and improvements:\n\n- Async fn in traits (stabilized)\n- Return position impl Trait in traits\n- Various performance improvements\n\nAs always, you can upgrade via rustup:\n\n```\nrustup update stable\n```".to_string(),
                    read: false,
                    starred: true,
                },
                Article {
                    title: "This Year in Rust".to_string(),
                    link: "https://blog.rust-lang.org/2024/01/02/year.html".to_string(),
                    published: Utc::now() - chrono::Duration::days(1),
                    summary: "A retrospective of Rust's achievements this year.".to_string(),
                    content: "It's been an incredible year for Rust!\n\nHere are some highlights:\n\n1. Async fn in traits\n2. Generic associated types\n3. Type alias impl trait\n4. Growing ecosystem".to_string(),
                    read: false,
                    starred: false,
                },
                Article {
                    title: "Rust Foundation Update".to_string(),
                    link: "https://foundation.rust-lang.org/update".to_string(),
                    published: Utc::now() - chrono::Duration::days(3),
                    summary: "Updates from the Rust Foundation.".to_string(),
                    content: "The Rust Foundation has been busy this quarter...".to_string(),
                    read: false,
                    starred: false,
                },
            ],
        },
        Feed {
            title: "Hacker News".to_string(),
            url: "https://news.ycombinator.com/rss".to_string(),
            unread_count: 5,
            articles: vec![
                Article {
                    title: "Show HN: A new approach to terminal UIs".to_string(),
                    link: "https://news.ycombinator.com/item?id=12345".to_string(),
                    published: Utc::now() - chrono::Duration::hours(2),
                    summary: "I built a new TUI framework in Rust".to_string(),
                    content: "After years of working with ncurses, I decided to build something better...".to_string(),
                    read: false,
                    starred: false,
                },
                Article {
                    title: "The future of programming languages".to_string(),
                    link: "https://news.ycombinator.com/item?id=12346".to_string(),
                    published: Utc::now() - chrono::Duration::hours(6),
                    summary: "Where are programming languages headed?".to_string(),
                    content: "In this essay, I explore emerging trends in PL design...".to_string(),
                    read: true,
                    starred: true,
                },
            ],
        },
        Feed {
            title: "This Week in Rust".to_string(),
            url: "https://this-week-in-rust.org/rss.xml".to_string(),
            unread_count: 1,
            articles: vec![
                Article {
                    title: "This Week in Rust 520".to_string(),
                    link: "https://this-week-in-rust.org/blog/2024/01/03/this-week-in-rust-520/".to_string(),
                    published: Utc::now() - chrono::Duration::hours(12),
                    summary: "The weekly newsletter for Rust developers.".to_string(),
                    content: "Hello and welcome to another issue of This Week in Rust!\n\n## Crate of the Week\n\nThis week's crate is `ratatui`.\n\n## Updates from Rust Community\n\n...".to_string(),
                    read: false,
                    starred: false,
                },
            ],
        },
    ]
}
