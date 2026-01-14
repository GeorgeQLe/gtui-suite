use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel { Info, Warn, Error, Debug }

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub container: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub color: u8,
}

pub struct App {
    pub containers: Vec<Container>,
    pub logs: Vec<LogEntry>,
    pub selected_container: usize,
    pub scroll_offset: usize,
    pub follow: bool,
    pub filter_level: Option<LogLevel>,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            containers: vec![
                Container { id: "abc123".into(), name: "nginx".into(), enabled: true, color: 1 },
                Container { id: "def456".into(), name: "api".into(), enabled: true, color: 2 },
                Container { id: "ghi789".into(), name: "db".into(), enabled: true, color: 3 },
                Container { id: "jkl012".into(), name: "redis".into(), enabled: false, color: 4 },
            ],
            logs: vec![
                LogEntry { timestamp: Local::now(), container: "nginx".into(), level: LogLevel::Info, message: "Starting nginx server...".into() },
                LogEntry { timestamp: Local::now(), container: "api".into(), level: LogLevel::Info, message: "Connected to database".into() },
                LogEntry { timestamp: Local::now(), container: "nginx".into(), level: LogLevel::Info, message: "Listening on port 80".into() },
                LogEntry { timestamp: Local::now(), container: "db".into(), level: LogLevel::Warn, message: "High memory usage detected".into() },
                LogEntry { timestamp: Local::now(), container: "api".into(), level: LogLevel::Error, message: "Failed to connect to redis".into() },
                LogEntry { timestamp: Local::now(), container: "api".into(), level: LogLevel::Info, message: "Retrying connection...".into() },
                LogEntry { timestamp: Local::now(), container: "nginx".into(), level: LogLevel::Info, message: "GET /api/health 200 5ms".into() },
                LogEntry { timestamp: Local::now(), container: "api".into(), level: LogLevel::Debug, message: "Processing request id=12345".into() },
            ],
            selected_container: 0,
            scroll_offset: 0,
            follow: true,
            filter_level: None,
            tick_count: 0,
        }
    }

    pub fn filtered_logs(&self) -> Vec<&LogEntry> {
        self.logs.iter()
            .filter(|log| {
                let container_enabled = self.containers.iter().find(|c| c.name == log.container).map(|c| c.enabled).unwrap_or(true);
                let level_match = self.filter_level.as_ref().map(|f| *f == log.level).unwrap_or(true);
                container_enabled && level_match
            })
            .collect()
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        if self.tick_count % 5 == 0 {
            let containers = ["nginx", "api", "db"];
            let levels = [LogLevel::Info, LogLevel::Warn, LogLevel::Debug];
            let messages = ["Request processed", "Health check passed", "Cache hit", "Connection established"];
            let container = containers[self.tick_count as usize % containers.len()];
            let level = levels[self.tick_count as usize % levels.len()].clone();
            let message = messages[self.tick_count as usize % messages.len()];

            self.logs.push(LogEntry {
                timestamp: Local::now(),
                container: container.into(),
                level,
                message: message.into(),
            });

            if self.logs.len() > 500 { self.logs.remove(0); }
            if self.follow { self.scroll_offset = self.filtered_logs().len().saturating_sub(1); }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let max = self.filtered_logs().len().saturating_sub(1);
                if self.scroll_offset < max { self.scroll_offset += 1; self.follow = false; }
            },
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                self.follow = false;
            },
            KeyCode::Char('f') => self.follow = !self.follow,
            KeyCode::Char('c') => { self.logs.clear(); self.scroll_offset = 0; },
            KeyCode::Tab => {
                self.selected_container = (self.selected_container + 1) % self.containers.len();
            },
            KeyCode::Char(' ') => {
                if let Some(container) = self.containers.get_mut(self.selected_container) {
                    container.enabled = !container.enabled;
                }
            },
            KeyCode::Char('1') => self.filter_level = None,
            KeyCode::Char('2') => self.filter_level = Some(LogLevel::Info),
            KeyCode::Char('3') => self.filter_level = Some(LogLevel::Warn),
            KeyCode::Char('4') => self.filter_level = Some(LogLevel::Error),
            KeyCode::Char('G') => {
                self.scroll_offset = self.filtered_logs().len().saturating_sub(1);
                self.follow = true;
            },
            KeyCode::Char('g') => { self.scroll_offset = 0; self.follow = false; },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        let filter_str = match &self.filter_level {
            None => "ALL",
            Some(LogLevel::Info) => "INFO",
            Some(LogLevel::Warn) => "WARN",
            Some(LogLevel::Error) => "ERROR",
            Some(LogLevel::Debug) => "DEBUG",
        };
        format!("j/k:scroll f:follow({}) tab:container space:toggle 1-4:filter[{}] c:clear q:quit",
            if self.follow { "on" } else { "off" }, filter_str)
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
