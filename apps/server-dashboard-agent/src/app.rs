use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sysinfo::{Disks, System};

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    ServerDetail,
    Alerts,
    AlertRules,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,

    // Servers
    pub servers: Vec<Server>,
    pub selected_server: usize,
    pub current_server: Option<Server>,

    // Alerts
    pub alerts: Vec<Alert>,
    pub selected_alert: usize,
    pub alert_rules: Vec<AlertRule>,
    pub selected_rule: usize,

    // History
    pub history: Vec<MetricHistory>,
    pub selected_history: usize,

    // UI state
    pub show_graphs: bool,
    pub search_query: String,
    pub scroll_offset: usize,

    // Local system info
    pub system: System,

    // Status
    pub status_message: Option<String>,
    pub connected: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let show_graphs = config.display.show_graphs;

        Self {
            config,
            view: View::Dashboard,
            input_mode: InputMode::Normal,
            servers: Vec::new(),
            selected_server: 0,
            current_server: None,
            alerts: Vec::new(),
            selected_alert: 0,
            alert_rules: Vec::new(),
            selected_rule: 0,
            history: Vec::new(),
            selected_history: 0,
            show_graphs,
            search_query: String::new(),
            scroll_offset: 0,
            system: System::new_all(),
            status_message: None,
            connected: false,
        }
    }

    pub async fn refresh(&mut self) {
        // Update system info
        self.system.refresh_all();

        // Get local hostname
        let hostname = System::host_name().unwrap_or_else(|| "localhost".to_string());

        // Generate demo servers including local
        self.servers = vec![
            {
                let mut server = Server::new(&hostname);
                server.ip_address = "127.0.0.1".to_string();
                server.status = ServerStatus::Online;
                server.last_seen = Utc::now();
                server.uptime_secs = System::uptime();
                server.metrics = self.get_local_metrics();
                server
            },
            {
                let mut server = Server::new("web-server-1");
                server.ip_address = "192.168.1.10".to_string();
                server.status = ServerStatus::Online;
                server.last_seen = Utc::now() - chrono::Duration::seconds(15);
                server.uptime_secs = 86400 * 30;
                server.metrics = ServerMetrics {
                    cpu_usage: 45.0,
                    memory_total: 16 * 1024 * 1024 * 1024,
                    memory_used: 12 * 1024 * 1024 * 1024,
                    disk_total: 500 * 1024 * 1024 * 1024,
                    disk_used: 200 * 1024 * 1024 * 1024,
                    load_1: 2.5,
                    load_5: 2.0,
                    load_15: 1.8,
                    process_count: 150,
                    ..Default::default()
                };
                server
            },
            {
                let mut server = Server::new("db-server-1");
                server.ip_address = "192.168.1.20".to_string();
                server.status = ServerStatus::Warning;
                server.last_seen = Utc::now() - chrono::Duration::seconds(5);
                server.uptime_secs = 86400 * 90;
                server.metrics = ServerMetrics {
                    cpu_usage: 85.0,
                    memory_total: 64 * 1024 * 1024 * 1024,
                    memory_used: 58 * 1024 * 1024 * 1024,
                    disk_total: 2000 * 1024 * 1024 * 1024,
                    disk_used: 1800 * 1024 * 1024 * 1024,
                    load_1: 8.0,
                    load_5: 7.5,
                    load_15: 6.0,
                    process_count: 200,
                    ..Default::default()
                };
                server
            },
            {
                let mut server = Server::new("cache-server-1");
                server.ip_address = "192.168.1.30".to_string();
                server.status = ServerStatus::Critical;
                server.last_seen = Utc::now() - chrono::Duration::seconds(120);
                server.uptime_secs = 86400 * 7;
                server.metrics = ServerMetrics {
                    cpu_usage: 95.0,
                    memory_total: 32 * 1024 * 1024 * 1024,
                    memory_used: 31 * 1024 * 1024 * 1024,
                    disk_total: 100 * 1024 * 1024 * 1024,
                    disk_used: 95 * 1024 * 1024 * 1024,
                    load_1: 12.0,
                    load_5: 10.0,
                    load_15: 8.0,
                    process_count: 300,
                    ..Default::default()
                };
                server
            },
        ];

        // Generate demo alerts
        self.alerts = vec![
            {
                let mut alert = Alert::new("db-server-1", "cpu_usage", 85.0, 80.0);
                alert.severity = AlertSeverity::Warning;
                alert.message = "CPU usage above 80%".to_string();
                alert
            },
            {
                let mut alert = Alert::new("cache-server-1", "memory_usage", 96.9, 90.0);
                alert.severity = AlertSeverity::Critical;
                alert.message = "Memory usage above 90%".to_string();
                alert
            },
            {
                let mut alert = Alert::new("cache-server-1", "disk_usage", 95.0, 85.0);
                alert.severity = AlertSeverity::Critical;
                alert.message = "Disk usage above 85%".to_string();
                alert
            },
        ];

        // Generate demo alert rules
        self.alert_rules = vec![
            AlertRule::new("High CPU", "cpu_usage", AlertCondition::GreaterThan, 80.0),
            AlertRule::new("High Memory", "memory_usage", AlertCondition::GreaterThan, 90.0),
            AlertRule::new("High Disk", "disk_usage", AlertCondition::GreaterThan, 85.0),
            AlertRule::new("High Load", "load_1", AlertCondition::GreaterThan, 10.0),
        ];

        // Generate demo history
        self.history = vec![
            {
                let mut h = MetricHistory::new("cpu_usage");
                for i in 0..60 {
                    let t = Utc::now() - chrono::Duration::minutes(60 - i);
                    let v = 40.0 + 30.0 * (i as f64 * 0.1).sin() + (i as f64 * 0.2);
                    h.add(t, v);
                }
                h
            },
            {
                let mut h = MetricHistory::new("memory_usage");
                for i in 0..60 {
                    let t = Utc::now() - chrono::Duration::minutes(60 - i);
                    let v = 60.0 + 10.0 * (i as f64 * 0.15).cos();
                    h.add(t, v);
                }
                h
            },
        ];

        self.connected = true;

        if self.selected_server >= self.servers.len() {
            self.selected_server = self.servers.len().saturating_sub(1);
        }
    }

    fn get_local_metrics(&self) -> ServerMetrics {
        let cpu_usage = self.system.global_cpu_usage() as f64;
        let memory_total = self.system.total_memory();
        let memory_used = self.system.used_memory();

        // Get disk info
        let disks = Disks::new_with_refreshed_list();
        let (disk_total, disk_used) = disks.list().first().map(|d| {
            (d.total_space(), d.total_space() - d.available_space())
        }).unwrap_or((0, 0));

        // Get load averages
        let load = System::load_average();

        ServerMetrics {
            cpu_usage,
            memory_total,
            memory_used,
            disk_total,
            disk_used,
            load_1: load.one,
            load_5: load.five,
            load_15: load.fifteen,
            process_count: self.system.processes().len() as u32,
            ..Default::default()
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') => {
                match self.view {
                    View::ServerDetail | View::History => {
                        self.view = View::Dashboard;
                        self.current_server = None;
                    }
                    View::AlertRules => {
                        self.view = View::Alerts;
                    }
                    View::Alerts => {
                        self.view = View::Dashboard;
                    }
                    View::Dashboard => return true,
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
            }
            KeyCode::Char('g') => {
                self.go_to_top();
            }
            KeyCode::Char('G') => {
                self.go_to_bottom();
            }

            KeyCode::Enter => {
                self.open_details();
            }

            KeyCode::Char('a') => {
                self.view = View::Alerts;
                self.selected_alert = 0;
            }

            KeyCode::Char('A') => {
                self.view = View::AlertRules;
                self.selected_rule = 0;
            }

            KeyCode::Char('h') => {
                if self.view == View::ServerDetail {
                    self.view = View::History;
                    self.selected_history = 0;
                }
            }

            KeyCode::Char('G') | KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.show_graphs = !self.show_graphs;
                self.status_message = Some(format!(
                    "Graphs: {}",
                    if self.show_graphs { "on" } else { "off" }
                ));
            }

            KeyCode::Char('r') => {
                self.refresh().await;
                self.status_message = Some("Refreshed".to_string());
            }

            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }

            KeyCode::Tab => {
                self.view = match self.view {
                    View::Dashboard => View::Alerts,
                    View::Alerts => View::Dashboard,
                    _ => self.view,
                };
            }

            KeyCode::Esc => {
                self.view = View::Dashboard;
                self.current_server = None;
            }

            _ => {}
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
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

    fn navigate_down(&mut self) {
        match self.view {
            View::Dashboard => {
                let max = self.filtered_servers().len().saturating_sub(1);
                if self.selected_server < max {
                    self.selected_server += 1;
                }
            }
            View::Alerts => {
                if self.selected_alert < self.alerts.len().saturating_sub(1) {
                    self.selected_alert += 1;
                }
            }
            View::AlertRules => {
                if self.selected_rule < self.alert_rules.len().saturating_sub(1) {
                    self.selected_rule += 1;
                }
            }
            View::History => {
                if self.selected_history < self.history.len().saturating_sub(1) {
                    self.selected_history += 1;
                }
            }
            _ => {}
        }
    }

    fn navigate_up(&mut self) {
        match self.view {
            View::Dashboard => {
                self.selected_server = self.selected_server.saturating_sub(1);
            }
            View::Alerts => {
                self.selected_alert = self.selected_alert.saturating_sub(1);
            }
            View::AlertRules => {
                self.selected_rule = self.selected_rule.saturating_sub(1);
            }
            View::History => {
                self.selected_history = self.selected_history.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn go_to_top(&mut self) {
        match self.view {
            View::Dashboard => self.selected_server = 0,
            View::Alerts => self.selected_alert = 0,
            View::AlertRules => self.selected_rule = 0,
            View::History => self.selected_history = 0,
            _ => {}
        }
    }

    fn go_to_bottom(&mut self) {
        match self.view {
            View::Dashboard => {
                self.selected_server = self.filtered_servers().len().saturating_sub(1);
            }
            View::Alerts => {
                self.selected_alert = self.alerts.len().saturating_sub(1);
            }
            View::AlertRules => {
                self.selected_rule = self.alert_rules.len().saturating_sub(1);
            }
            View::History => {
                self.selected_history = self.history.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    fn open_details(&mut self) {
        if self.view == View::Dashboard {
            if let Some(server) = self.filtered_servers().get(self.selected_server) {
                self.current_server = Some((*server).clone());
                self.view = View::ServerDetail;
            }
        }
    }

    pub fn filtered_servers(&self) -> Vec<&Server> {
        if self.search_query.is_empty() {
            self.servers.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.servers
                .iter()
                .filter(|s| {
                    s.hostname.to_lowercase().contains(&query)
                        || s.ip_address.to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    pub fn active_alerts_count(&self) -> usize {
        self.alerts.iter().filter(|a| a.is_active()).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            return msg.clone();
        }

        let active_alerts = self.active_alerts_count();

        match self.view {
            View::Dashboard => format!(
                "{} servers | {} alerts | j/k:nav Enter:details a:alerts r:refresh q:quit",
                self.servers.len(),
                active_alerts
            ),
            View::ServerDetail => "h:history g:toggle graphs q:back".to_string(),
            View::Alerts => format!("{} active alerts | A:rules q:back", active_alerts),
            View::AlertRules => format!("{} rules | q:back", self.alert_rules.len()),
            View::History => "j/k:select metric q:back".to_string(),
        }
    }
}
