use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use regex::Regex;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use crate::config::Config;
use crate::log_entry::{LogEntry, LogLevel};
use crate::parser::LogParser;
use crate::watcher::FileWatcher;

/// Application mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
    Filter,
    Help,
}

/// Application state
pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub entries: Vec<LogEntry>,
    pub scroll_offset: usize,
    pub selected: usize,
    pub follow_mode: bool,
    pub show_line_numbers: bool,
    pub wrap_lines: bool,

    // Search state
    pub search_query: String,
    pub search_regex: Option<Regex>,
    pub search_matches: Vec<usize>,
    pub current_match: usize,

    // Filter state
    pub level_filter: Option<LogLevel>,
    pub filter_input: String,

    // Bookmarks
    pub bookmarks: HashSet<usize>,

    // File handling
    pub file_path: Option<PathBuf>,
    file_position: u64,
    watcher: Option<FileWatcher>,
    parser: LogParser,

    // UI state
    pub viewport_height: u16,
    pub message: Option<String>,
}

impl App {
    pub fn new(file_path: Option<&str>) -> Result<Self> {
        let config = Config::load()?;
        let parser = LogParser::new();

        let mut app = Self {
            config,
            mode: Mode::Normal,
            entries: Vec::new(),
            scroll_offset: 0,
            selected: 0,
            follow_mode: true,
            show_line_numbers: true,
            wrap_lines: false,
            search_query: String::new(),
            search_regex: None,
            search_matches: Vec::new(),
            current_match: 0,
            level_filter: None,
            filter_input: String::new(),
            bookmarks: HashSet::new(),
            file_path: file_path.map(PathBuf::from),
            file_position: 0,
            watcher: None,
            parser,
            viewport_height: 24,
            message: None,
        };

        if let Some(path) = file_path {
            app.load_file(path)?;
            app.watcher = FileWatcher::new(path).ok();
        } else {
            // Demo mode with sample entries
            app.load_demo_entries();
        }

        Ok(app)
    }

    fn load_file(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        self.entries.clear();

        for (line_num, line) in reader.lines().enumerate() {
            if let Ok(line) = line {
                let entry = self.parser.parse(&line, line_num + 1);
                self.entries.push(entry);
            }
        }

        // Track file position for follow mode
        let file = File::open(path)?;
        self.file_position = file.metadata()?.len();
        self.file_path = Some(PathBuf::from(path));

        if self.follow_mode && !self.entries.is_empty() {
            self.scroll_to_bottom();
        }

        self.message = Some(format!("Loaded {} lines from {}", self.entries.len(), path));
        Ok(())
    }

    fn load_demo_entries(&mut self) {
        let demo_lines = vec![
            r#"2024-01-15 10:30:00 INFO  Application started"#,
            r#"2024-01-15 10:30:01 DEBUG Loading configuration from /etc/app/config.toml"#,
            r#"2024-01-15 10:30:02 INFO  Connected to database at localhost:5432"#,
            r#"2024-01-15 10:30:03 WARN  Cache miss for key: user_session_abc123"#,
            r#"2024-01-15 10:30:04 INFO  HTTP server listening on 0.0.0.0:8080"#,
            r#"2024-01-15 10:30:05 DEBUG Request received: GET /api/users"#,
            r#"2024-01-15 10:30:06 INFO  Processed request in 45ms"#,
            r#"2024-01-15 10:30:07 ERROR Failed to connect to external service: timeout"#,
            r#"2024-01-15 10:30:08 WARN  Retrying connection (attempt 1/3)"#,
            r#"2024-01-15 10:30:09 INFO  Connection restored"#,
            r#"2024-01-15 10:30:10 DEBUG Processing batch job: export_reports"#,
            r#"2024-01-15 10:30:11 INFO  Exported 150 reports successfully"#,
            r#"2024-01-15 10:30:12 TRACE Memory usage: 256MB / 1024MB"#,
            r#"2024-01-15 10:30:13 DEBUG Garbage collection completed in 12ms"#,
            r#"2024-01-15 10:30:14 INFO  User login: john.doe@example.com"#,
            r#"2024-01-15 10:30:15 WARN  Rate limit approaching for IP 192.168.1.100"#,
            r#"2024-01-15 10:30:16 ERROR Invalid authentication token"#,
            r#"2024-01-15 10:30:17 INFO  Session expired for user: jane.smith"#,
            r#"2024-01-15 10:30:18 DEBUG Cleaning up stale connections"#,
            r#"2024-01-15 10:30:19 INFO  Scheduled maintenance in 1 hour"#,
            r#"{"timestamp":"2024-01-15T10:30:20Z","level":"info","message":"JSON log entry","user":"admin"}"#,
            r#"{"timestamp":"2024-01-15T10:30:21Z","level":"error","message":"Database query failed","query":"SELECT * FROM users","duration_ms":5000}"#,
        ];

        for (i, line) in demo_lines.iter().enumerate() {
            let entry = self.parser.parse(line, i + 1);
            self.entries.push(entry);
        }

        self.message = Some("Demo mode: showing sample log entries. Pass a file path to view real logs.".to_string());
    }

    pub fn check_updates(&mut self) -> Result<()> {
        if !self.follow_mode {
            return Ok(());
        }

        let should_update = self.watcher.as_mut().map(|w| w.check()).unwrap_or(false);

        if should_update {
            if let Some(ref path) = self.file_path {
                self.read_new_lines(path.clone())?;
            }
        }

        Ok(())
    }

    fn read_new_lines(&mut self, path: PathBuf) -> Result<()> {
        let mut file = File::open(&path)?;
        let current_len = file.metadata()?.len();

        if current_len > self.file_position {
            file.seek(SeekFrom::Start(self.file_position))?;
            let reader = BufReader::new(file);
            let start_line = self.entries.len();

            for (i, line) in reader.lines().enumerate() {
                if let Ok(line) = line {
                    let entry = self.parser.parse(&line, start_line + i + 1);
                    self.entries.push(entry);
                }
            }

            self.file_position = current_len;

            if self.follow_mode {
                self.scroll_to_bottom();
            }
        } else if current_len < self.file_position {
            // File was truncated, reload
            self.load_file(path.to_str().unwrap_or(""))?;
        }

        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Search => self.handle_search_key(key),
            Mode::Filter => self.handle_filter_key(key),
            Mode::Help => self.handle_help_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        self.message = None;

        match key.code {
            // Navigation
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::PageDown | KeyCode::Char('J') => self.page_down(),
            KeyCode::PageUp | KeyCode::Char('K') => self.page_up(),
            KeyCode::Home | KeyCode::Char('g') => self.scroll_to_top(),
            KeyCode::End | KeyCode::Char('G') => self.scroll_to_bottom(),

            // Modes
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('l') => {
                self.mode = Mode::Filter;
                self.filter_input.clear();
            }
            KeyCode::Char('?') => {
                self.mode = Mode::Help;
            }

            // Search navigation
            KeyCode::Char('n') => self.next_match(),
            KeyCode::Char('N') => self.prev_match(),

            // Toggle features
            KeyCode::Char('f') => {
                self.follow_mode = !self.follow_mode;
                self.message = Some(format!(
                    "Follow mode: {}",
                    if self.follow_mode { "ON" } else { "OFF" }
                ));
                if self.follow_mode {
                    self.scroll_to_bottom();
                }
            }
            KeyCode::Char('w') => {
                self.wrap_lines = !self.wrap_lines;
                self.message = Some(format!(
                    "Line wrap: {}",
                    if self.wrap_lines { "ON" } else { "OFF" }
                ));
            }
            KeyCode::Char('#') => {
                self.show_line_numbers = !self.show_line_numbers;
            }

            // Bookmarks
            KeyCode::Char('b') => self.toggle_bookmark(),
            KeyCode::Char('B') => self.clear_bookmarks(),
            KeyCode::Char('\'') => self.jump_to_next_bookmark(),

            // Clear filters
            KeyCode::Char('c') => {
                self.level_filter = None;
                self.search_query.clear();
                self.search_regex = None;
                self.search_matches.clear();
                self.message = Some("Filters cleared".to_string());
            }

            // Level quick filters
            KeyCode::Char('1') => self.set_level_filter(Some(LogLevel::Error)),
            KeyCode::Char('2') => self.set_level_filter(Some(LogLevel::Warn)),
            KeyCode::Char('3') => self.set_level_filter(Some(LogLevel::Info)),
            KeyCode::Char('4') => self.set_level_filter(Some(LogLevel::Debug)),
            KeyCode::Char('5') => self.set_level_filter(Some(LogLevel::Trace)),
            KeyCode::Char('0') => self.set_level_filter(None),

            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.execute_search();
                self.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_query.clear();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.apply_level_filter();
                self.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.filter_input.clear();
            }
            KeyCode::Char(c) => {
                self.filter_input.push(c);
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn move_down(&mut self) {
        self.follow_mode = false;
        let visible = self.visible_entries();
        if self.selected < visible.len().saturating_sub(1) {
            self.selected += 1;
            self.ensure_visible();
        }
    }

    fn move_up(&mut self) {
        self.follow_mode = false;
        if self.selected > 0 {
            self.selected -= 1;
            self.ensure_visible();
        }
    }

    fn page_down(&mut self) {
        self.follow_mode = false;
        let visible = self.visible_entries();
        let page = self.viewport_height.saturating_sub(2) as usize;
        self.selected = (self.selected + page).min(visible.len().saturating_sub(1));
        self.ensure_visible();
    }

    fn page_up(&mut self) {
        self.follow_mode = false;
        let page = self.viewport_height.saturating_sub(2) as usize;
        self.selected = self.selected.saturating_sub(page);
        self.ensure_visible();
    }

    fn scroll_to_top(&mut self) {
        self.follow_mode = false;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    fn scroll_to_bottom(&mut self) {
        let visible = self.visible_entries();
        self.selected = visible.len().saturating_sub(1);
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        let viewport = self.viewport_height.saturating_sub(4) as usize;
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + viewport {
            self.scroll_offset = self.selected.saturating_sub(viewport) + 1;
        }
    }

    fn execute_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_regex = None;
            self.search_matches.clear();
            return;
        }

        match Regex::new(&self.search_query) {
            Ok(regex) => {
                // Collect matches first to avoid borrow issues
                let matches: Vec<usize> = self.visible_entries()
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| regex.is_match(&entry.raw))
                    .map(|(i, _)| i)
                    .collect();

                self.search_matches = matches;
                self.search_regex = Some(regex);
                self.current_match = 0;

                if !self.search_matches.is_empty() {
                    self.selected = self.search_matches[0];
                    self.ensure_visible();
                    self.message = Some(format!(
                        "Found {} matches",
                        self.search_matches.len()
                    ));
                } else {
                    self.message = Some("No matches found".to_string());
                }
            }
            Err(e) => {
                self.message = Some(format!("Invalid regex: {}", e));
            }
        }
    }

    fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        self.selected = self.search_matches[self.current_match];
        self.ensure_visible();
        self.message = Some(format!(
            "Match {}/{}",
            self.current_match + 1,
            self.search_matches.len()
        ));
    }

    fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = if self.current_match == 0 {
            self.search_matches.len() - 1
        } else {
            self.current_match - 1
        };
        self.selected = self.search_matches[self.current_match];
        self.ensure_visible();
        self.message = Some(format!(
            "Match {}/{}",
            self.current_match + 1,
            self.search_matches.len()
        ));
    }

    fn set_level_filter(&mut self, level: Option<LogLevel>) {
        self.level_filter = level;
        self.selected = 0;
        self.scroll_offset = 0;
        self.message = Some(match level {
            Some(l) => format!("Filter: {} and above", l.label()),
            None => "Filter: All levels".to_string(),
        });
    }

    fn apply_level_filter(&mut self) {
        let level = match self.filter_input.to_lowercase().as_str() {
            "e" | "error" => Some(LogLevel::Error),
            "w" | "warn" | "warning" => Some(LogLevel::Warn),
            "i" | "info" => Some(LogLevel::Info),
            "d" | "debug" => Some(LogLevel::Debug),
            "t" | "trace" => Some(LogLevel::Trace),
            _ => None,
        };
        self.set_level_filter(level);
        self.filter_input.clear();
    }

    fn toggle_bookmark(&mut self) {
        let visible = self.visible_entries();
        if let Some(entry) = visible.get(self.selected) {
            let line_num = entry.line_number;
            if self.bookmarks.contains(&line_num) {
                self.bookmarks.remove(&line_num);
                self.message = Some(format!("Bookmark removed: line {}", line_num));
            } else {
                self.bookmarks.insert(line_num);
                self.message = Some(format!("Bookmark added: line {}", line_num));
            }
        }
    }

    fn clear_bookmarks(&mut self) {
        self.bookmarks.clear();
        self.message = Some("All bookmarks cleared".to_string());
    }

    fn jump_to_next_bookmark(&mut self) {
        if self.bookmarks.is_empty() {
            self.message = Some("No bookmarks set".to_string());
            return;
        }

        let visible = self.visible_entries();
        let current_line = visible.get(self.selected).map(|e| e.line_number).unwrap_or(0);

        // Find next bookmark after current position
        let mut sorted: Vec<_> = self.bookmarks.iter().copied().collect();
        sorted.sort();

        let next = sorted.iter().find(|&&b| b > current_line).copied()
            .or_else(|| sorted.first().copied());

        if let Some(target) = next {
            // Find index in visible entries
            for (i, entry) in visible.iter().enumerate() {
                if entry.line_number == target {
                    self.selected = i;
                    self.ensure_visible();
                    self.message = Some(format!("Jumped to bookmark: line {}", target));
                    return;
                }
            }
        }
    }

    pub fn visible_entries(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(ref filter) = self.level_filter {
                    e.level.severity() >= filter.severity()
                } else {
                    true
                }
            })
            .collect()
    }
}
