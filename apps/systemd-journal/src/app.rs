use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub timestamp: DateTime<Utc>,
    pub unit: String,
    pub priority: Priority,
    pub message: String,
    pub pid: Option<u32>,
    pub uid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Emergency => "EMERG",
            Priority::Alert => "ALERT",
            Priority::Critical => "CRIT",
            Priority::Error => "ERROR",
            Priority::Warning => "WARN",
            Priority::Notice => "NOTICE",
            Priority::Info => "INFO",
            Priority::Debug => "DEBUG",
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            Priority::Emergency => 0,
            Priority::Alert => 1,
            Priority::Critical => 2,
            Priority::Error => 3,
            Priority::Warning => 4,
            Priority::Notice => 5,
            Priority::Info => 6,
            Priority::Debug => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Entries,
    Details,
    Units,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterPriority {
    All,
    Error,
    Warning,
    Info,
}

pub struct App {
    pub entries: Vec<JournalEntry>,
    pub units: Vec<String>,
    pub selected: usize,
    pub selected_unit: usize,
    pub view: View,
    pub filter_priority: FilterPriority,
    pub filter_unit: Option<String>,
    pub search_query: String,
    pub is_searching: bool,
    pub follow_mode: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            units: Vec::new(),
            selected: 0,
            selected_unit: 0,
            view: View::Entries,
            filter_priority: FilterPriority::All,
            filter_unit: None,
            search_query: String::new(),
            is_searching: false,
            follow_mode: false,
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        self.entries = create_demo_entries();
        self.units = self
            .entries
            .iter()
            .map(|e| e.unit.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        self.units.sort();
    }

    pub fn filtered_entries(&self) -> Vec<&JournalEntry> {
        self.entries
            .iter()
            .filter(|e| {
                let priority_ok = match self.filter_priority {
                    FilterPriority::All => true,
                    FilterPriority::Error => e.priority.level() <= 3,
                    FilterPriority::Warning => e.priority.level() <= 4,
                    FilterPriority::Info => e.priority.level() <= 6,
                };

                let unit_ok = self
                    .filter_unit
                    .as_ref()
                    .map(|u| &e.unit == u)
                    .unwrap_or(true);

                let search_ok = self.search_query.is_empty()
                    || e.message.to_lowercase().contains(&self.search_query.to_lowercase())
                    || e.unit.to_lowercase().contains(&self.search_query.to_lowercase());

                priority_ok && unit_ok && search_ok
            })
            .collect()
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        if self.is_searching {
            return self.handle_search_key(key);
        }

        match self.view {
            View::Entries => self.handle_entries_key(key),
            View::Details => self.handle_details_key(key),
            View::Units => self.handle_units_key(key),
        }
    }

    fn handle_entries_key(&mut self, key: KeyEvent) -> bool {
        let filtered = self.filtered_entries();
        let max_idx = filtered.len().saturating_sub(1);

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < max_idx {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !filtered.is_empty() {
                    self.view = View::Details;
                }
            }
            KeyCode::Char('u') => {
                self.view = View::Units;
                self.selected_unit = 0;
            }
            KeyCode::Char('p') => {
                self.cycle_priority_filter();
            }
            KeyCode::Char('/') => {
                self.is_searching = true;
                self.search_query.clear();
            }
            KeyCode::Char('f') => {
                self.follow_mode = !self.follow_mode;
                self.status_message = Some(format!(
                    "Follow mode: {}",
                    if self.follow_mode { "ON" } else { "OFF" }
                ));
            }
            KeyCode::Char('c') => {
                self.filter_unit = None;
                self.search_query.clear();
                self.filter_priority = FilterPriority::All;
                self.selected = 0;
            }
            KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::Char('G') => {
                self.selected = max_idx;
            }
            _ => {}
        }
        false
    }

    fn handle_details_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Entries;
            }
            _ => {}
        }
        false
    }

    fn handle_units_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Entries;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_unit < self.units.len().saturating_sub(1) {
                    self.selected_unit += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_unit = self.selected_unit.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(unit) = self.units.get(self.selected_unit) {
                    self.filter_unit = Some(unit.clone());
                    self.view = View::Entries;
                    self.selected = 0;
                }
            }
            KeyCode::Char('a') => {
                self.filter_unit = None;
                self.view = View::Entries;
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.is_searching = false;
            }
            KeyCode::Enter => {
                self.is_searching = false;
                self.selected = 0;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        false
    }

    fn cycle_priority_filter(&mut self) {
        self.filter_priority = match self.filter_priority {
            FilterPriority::All => FilterPriority::Error,
            FilterPriority::Error => FilterPriority::Warning,
            FilterPriority::Warning => FilterPriority::Info,
            FilterPriority::Info => FilterPriority::All,
        };
        self.selected = 0;
    }

    pub fn selected_entry(&self) -> Option<&JournalEntry> {
        let filtered = self.filtered_entries();
        filtered.get(self.selected).copied()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        if self.is_searching {
            return format!("Search: {}_", self.search_query);
        }

        let filter_info = match self.filter_priority {
            FilterPriority::All => "all",
            FilterPriority::Error => "errors",
            FilterPriority::Warning => "warnings",
            FilterPriority::Info => "info",
        };

        format!(
            "{} entries | Filter: {} | u:units p:priority /:search f:follow c:clear",
            self.filtered_entries().len(),
            filter_info
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_entries() -> Vec<JournalEntry> {
    let now = Utc::now();

    vec![
        JournalEntry {
            timestamp: now - chrono::Duration::seconds(5),
            unit: "sshd.service".to_string(),
            priority: Priority::Info,
            message: "Accepted publickey for user from 192.168.1.100".to_string(),
            pid: Some(1234),
            uid: Some(0),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::seconds(30),
            unit: "nginx.service".to_string(),
            priority: Priority::Warning,
            message: "upstream timed out (110: Connection timed out)".to_string(),
            pid: Some(5678),
            uid: Some(33),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::minutes(1),
            unit: "systemd".to_string(),
            priority: Priority::Info,
            message: "Started Daily apt download activities.".to_string(),
            pid: Some(1),
            uid: Some(0),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::minutes(2),
            unit: "kernel".to_string(),
            priority: Priority::Error,
            message: "EXT4-fs error: unable to read inode block".to_string(),
            pid: None,
            uid: None,
        },
        JournalEntry {
            timestamp: now - chrono::Duration::minutes(5),
            unit: "docker.service".to_string(),
            priority: Priority::Info,
            message: "Container abc123 started".to_string(),
            pid: Some(9012),
            uid: Some(0),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::minutes(10),
            unit: "cron.service".to_string(),
            priority: Priority::Info,
            message: "(*) CMD (/usr/local/bin/backup.sh)".to_string(),
            pid: Some(3456),
            uid: Some(0),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::minutes(15),
            unit: "postgresql.service".to_string(),
            priority: Priority::Critical,
            message: "database system was shut down unexpectedly".to_string(),
            pid: Some(7890),
            uid: Some(26),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::minutes(20),
            unit: "NetworkManager.service".to_string(),
            priority: Priority::Notice,
            message: "NetworkManager state is now CONNECTED_GLOBAL".to_string(),
            pid: Some(456),
            uid: Some(0),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::hours(1),
            unit: "sshd.service".to_string(),
            priority: Priority::Warning,
            message: "Failed password for invalid user admin from 10.0.0.5".to_string(),
            pid: Some(1235),
            uid: Some(0),
        },
        JournalEntry {
            timestamp: now - chrono::Duration::hours(2),
            unit: "systemd".to_string(),
            priority: Priority::Info,
            message: "Finished Daily apt download activities.".to_string(),
            pid: Some(1),
            uid: Some(0),
        },
    ]
}
