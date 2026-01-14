use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Details,
    Add,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    EditHostname,
    EditPort,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,
    pub hosts: Vec<SslHost>,
    pub selected: usize,
    pub hostname_buffer: String,
    pub port_buffer: String,
    pub status_message: Option<String>,
    pub sort_by: SortBy,
    pub filter: FilterBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    ExpiryDate,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterBy {
    All,
    Valid,
    Warning,
    Critical,
    Expired,
    Error,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::List,
            input_mode: InputMode::Normal,
            hosts: Vec::new(),
            selected: 0,
            hostname_buffer: String::new(),
            port_buffer: "443".to_string(),
            status_message: None,
            sort_by: SortBy::ExpiryDate,
            filter: FilterBy::All,
        }
    }

    pub async fn refresh(&mut self) {
        self.hosts = create_demo_hosts();
        self.sort_hosts();
    }

    fn sort_hosts(&mut self) {
        match self.sort_by {
            SortBy::Name => {
                self.hosts.sort_by(|a, b| a.hostname.cmp(&b.hostname));
            }
            SortBy::ExpiryDate => {
                self.hosts.sort_by(|a, b| {
                    let a_days = a.days_until_expiry().unwrap_or(i64::MAX);
                    let b_days = b.days_until_expiry().unwrap_or(i64::MAX);
                    a_days.cmp(&b_days)
                });
            }
            SortBy::Status => {
                self.hosts.sort_by(|a, b| {
                    let status_order = |s: HostStatus| -> u8 {
                        match s {
                            HostStatus::Expired => 0,
                            HostStatus::Critical => 1,
                            HostStatus::Error => 2,
                            HostStatus::Warning => 3,
                            HostStatus::Valid => 4,
                            HostStatus::Unknown => 5,
                        }
                    };
                    status_order(a.status()).cmp(&status_order(b.status()))
                });
            }
        }
    }

    pub fn filtered_hosts(&self) -> Vec<&SslHost> {
        self.hosts
            .iter()
            .filter(|h| match self.filter {
                FilterBy::All => true,
                FilterBy::Valid => h.status() == HostStatus::Valid,
                FilterBy::Warning => h.status() == HostStatus::Warning,
                FilterBy::Critical => h.status() == HostStatus::Critical,
                FilterBy::Expired => h.status() == HostStatus::Expired,
                FilterBy::Error => h.status() == HostStatus::Error,
            })
            .collect()
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') if self.input_mode == InputMode::Normal && self.view == View::List => {
                return true
            }
            _ => {}
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::EditHostname | InputMode::EditPort => self.handle_edit_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match self.view {
            View::List => self.handle_list_key(key),
            View::Details => self.handle_details_key(key),
            View::Add => self.handle_add_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        let filtered = self.filtered_hosts();
        let max_idx = filtered.len().saturating_sub(1);

        match key.code {
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
            KeyCode::Char('a') => {
                self.start_add();
            }
            KeyCode::Char('d') => {
                self.delete_selected();
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing certificates...".to_string());
            }
            KeyCode::Char('s') => {
                self.cycle_sort();
            }
            KeyCode::Char('f') => {
                self.cycle_filter();
            }
            KeyCode::Char('1') => self.filter = FilterBy::All,
            KeyCode::Char('2') => self.filter = FilterBy::Valid,
            KeyCode::Char('3') => self.filter = FilterBy::Warning,
            KeyCode::Char('4') => self.filter = FilterBy::Critical,
            KeyCode::Char('5') => self.filter = FilterBy::Expired,
            KeyCode::Char('6') => self.filter = FilterBy::Error,
            _ => {}
        }
        false
    }

    fn handle_details_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.view = View::List;
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing certificate...".to_string());
            }
            _ => {}
        }
        false
    }

    fn handle_add_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
                self.hostname_buffer.clear();
                self.port_buffer = "443".to_string();
            }
            KeyCode::Tab => {
                self.input_mode = if self.input_mode == InputMode::EditHostname {
                    InputMode::EditPort
                } else {
                    InputMode::EditHostname
                };
            }
            KeyCode::Enter if self.input_mode == InputMode::Normal => {
                self.input_mode = InputMode::EditHostname;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_host();
            }
            _ => {}
        }
        false
    }

    fn handle_edit_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                self.input_mode = if self.input_mode == InputMode::EditHostname {
                    InputMode::EditPort
                } else {
                    InputMode::EditHostname
                };
            }
            KeyCode::Backspace => {
                let buffer = match self.input_mode {
                    InputMode::EditHostname => &mut self.hostname_buffer,
                    InputMode::EditPort => &mut self.port_buffer,
                    _ => return false,
                };
                buffer.pop();
            }
            KeyCode::Char(c) => {
                let buffer = match self.input_mode {
                    InputMode::EditHostname => &mut self.hostname_buffer,
                    InputMode::EditPort => &mut self.port_buffer,
                    _ => return false,
                };
                buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn start_add(&mut self) {
        self.view = View::Add;
        self.hostname_buffer.clear();
        self.port_buffer = "443".to_string();
        self.input_mode = InputMode::EditHostname;
    }

    fn save_host(&mut self) {
        if self.hostname_buffer.is_empty() {
            self.status_message = Some("Hostname is required".to_string());
            return;
        }

        let port: u16 = self.port_buffer.parse().unwrap_or(443);
        let host = SslHost::new(&self.hostname_buffer, port);

        self.hosts.push(host);
        self.sort_hosts();
        self.status_message = Some(format!("Added {}:{}", self.hostname_buffer, port));
        self.hostname_buffer.clear();
        self.port_buffer = "443".to_string();
        self.view = View::List;
        self.input_mode = InputMode::Normal;
    }

    fn delete_selected(&mut self) {
        let filtered = self.filtered_hosts();
        if self.selected < filtered.len() {
            let hostname = filtered[self.selected].hostname.clone();
            self.hosts.retain(|h| h.hostname != hostname);
            if self.selected >= self.hosts.len() {
                self.selected = self.hosts.len().saturating_sub(1);
            }
            self.status_message = Some(format!("Deleted {}", hostname));
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Name => SortBy::ExpiryDate,
            SortBy::ExpiryDate => SortBy::Status,
            SortBy::Status => SortBy::Name,
        };
        self.sort_hosts();
        self.status_message = Some(format!("Sorted by {:?}", self.sort_by));
    }

    fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            FilterBy::All => FilterBy::Valid,
            FilterBy::Valid => FilterBy::Warning,
            FilterBy::Warning => FilterBy::Critical,
            FilterBy::Critical => FilterBy::Expired,
            FilterBy::Expired => FilterBy::Error,
            FilterBy::Error => FilterBy::All,
        };
        self.selected = 0;
    }

    pub fn selected_host(&self) -> Option<&SslHost> {
        let filtered = self.filtered_hosts();
        filtered.get(self.selected).copied()
    }

    pub fn stats(&self) -> HostStats {
        let mut stats = HostStats::default();
        for host in &self.hosts {
            match host.status() {
                HostStatus::Valid => stats.valid += 1,
                HostStatus::Warning => stats.warning += 1,
                HostStatus::Critical => stats.critical += 1,
                HostStatus::Expired => stats.expired += 1,
                HostStatus::Error => stats.error += 1,
                HostStatus::Unknown => stats.unknown += 1,
            }
        }
        stats.total = self.hosts.len();
        stats
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List => format!(
                "{} hosts | a:add d:delete r:refresh s:sort f:filter 1-6:quick filter",
                self.hosts.len()
            ),
            View::Details => "r:refresh Esc:back".to_string(),
            View::Add => "Tab:next field Ctrl+s:save Esc:cancel".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct HostStats {
    pub total: usize,
    pub valid: usize,
    pub warning: usize,
    pub critical: usize,
    pub expired: usize,
    pub error: usize,
    pub unknown: usize,
}
