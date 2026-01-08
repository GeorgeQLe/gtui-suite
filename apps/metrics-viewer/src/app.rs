use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Query,
    Dashboard,
    Alerts,
    MetricBrowser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    QueryEdit,
    TimeRangeSelect,
    DashboardSelect,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,

    // Query
    pub query: String,
    pub query_cursor: usize,
    pub query_history: Vec<String>,
    pub history_index: Option<usize>,

    // Results
    pub results: Vec<MetricResult>,
    pub selected_result: usize,

    // Time range
    pub time_range: TimeRange,
    pub time_range_presets: Vec<TimeRangePreset>,
    pub selected_preset: usize,

    // Dashboards
    pub dashboards: Vec<Dashboard>,
    pub active_dashboard: Option<usize>,
    pub selected_panel: usize,

    // Alerts
    pub alerts: Vec<Alert>,
    pub selected_alert: usize,

    // Metrics browser
    pub metric_names: Vec<String>,
    pub selected_metric: usize,
    pub metric_filter: String,

    // UI state
    pub auto_refresh: bool,
    pub scroll_offset: usize,

    // Status
    pub status_message: Option<String>,
    pub connected: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let time_range = TimeRange::last(chrono::Duration::hours(1));
        let time_range_presets = TimeRangePreset::all();

        Self {
            config,
            view: View::Query,
            input_mode: InputMode::Normal,
            query: String::new(),
            query_cursor: 0,
            query_history: Vec::new(),
            history_index: None,
            results: Vec::new(),
            selected_result: 0,
            time_range,
            time_range_presets,
            selected_preset: 2, // 1h default
            dashboards: Vec::new(),
            active_dashboard: None,
            selected_panel: 0,
            alerts: Vec::new(),
            selected_alert: 0,
            metric_names: Vec::new(),
            selected_metric: 0,
            metric_filter: String::new(),
            auto_refresh: false,
            scroll_offset: 0,
            status_message: None,
            connected: false,
        }
    }

    pub async fn refresh(&mut self) {
        // Load demo metric names
        self.metric_names = vec![
            "up".to_string(),
            "node_cpu_seconds_total".to_string(),
            "node_memory_MemTotal_bytes".to_string(),
            "node_memory_MemAvailable_bytes".to_string(),
            "node_disk_read_bytes_total".to_string(),
            "node_disk_written_bytes_total".to_string(),
            "http_requests_total".to_string(),
            "http_request_duration_seconds".to_string(),
            "process_cpu_seconds_total".to_string(),
            "process_resident_memory_bytes".to_string(),
        ];

        // Load demo alerts
        self.alerts = vec![
            {
                let mut alert = Alert::new("HighCPU", AlertState::Firing);
                alert.severity = "critical".to_string();
                alert.annotations.insert(
                    "summary".to_string(),
                    "CPU usage above 90%".to_string(),
                );
                alert
            },
            {
                let mut alert = Alert::new("LowDiskSpace", AlertState::Pending);
                alert.severity = "warning".to_string();
                alert.annotations.insert(
                    "summary".to_string(),
                    "Disk space below 20%".to_string(),
                );
                alert
            },
        ];

        // Load demo dashboards
        self.dashboards = vec![
            {
                let mut dashboard = Dashboard::new("System Overview");
                dashboard.panels = vec![
                    Panel::new(
                        "CPU Usage",
                        PanelType::Graph,
                        "100 - avg(rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100",
                    ),
                    Panel::new(
                        "Memory Usage",
                        PanelType::Stat,
                        "(1 - node_memory_MemAvailable_bytes/node_memory_MemTotal_bytes) * 100",
                    ),
                    {
                        let mut panel = Panel::new(
                            "Disk Usage",
                            PanelType::Gauge,
                            "(1 - node_filesystem_avail_bytes/node_filesystem_size_bytes) * 100",
                        );
                        panel.unit = Some("%".to_string());
                        panel
                    },
                ];
                dashboard
            },
            {
                let mut dashboard = Dashboard::new("HTTP Metrics");
                dashboard.panels = vec![
                    Panel::new("Request Rate", PanelType::Graph, "rate(http_requests_total[5m])"),
                    Panel::new("Error Rate", PanelType::Stat, "rate(http_requests_total{status=~\"5..\"}[5m])"),
                ];
                dashboard
            },
        ];

        self.connected = true;
    }

    pub async fn execute_query(&mut self) {
        if self.query.is_empty() {
            return;
        }

        // Add to history
        if self.query_history.last() != Some(&self.query) {
            self.query_history.push(self.query.clone());
        }
        self.history_index = None;

        // Generate demo results based on query
        let now = Utc::now().timestamp() as f64;
        let step = 60.0; // 1 minute

        self.results = vec![
            {
                let mut result = MetricResult::with_labels(vec![
                    ("__name__", "demo_metric"),
                    ("instance", "localhost:9090"),
                ]);
                for i in 0..60 {
                    let t = now - (60.0 - i as f64) * step;
                    let v = 50.0 + 30.0 * (i as f64 * 0.1).sin() + (i as f64 * 0.5);
                    result.values.push((t, v));
                }
                result
            },
            {
                let mut result = MetricResult::with_labels(vec![
                    ("__name__", "demo_metric"),
                    ("instance", "localhost:9091"),
                ]);
                for i in 0..60 {
                    let t = now - (60.0 - i as f64) * step;
                    let v = 40.0 + 20.0 * (i as f64 * 0.15).cos() + (i as f64 * 0.3);
                    result.values.push((t, v));
                }
                result
            },
        ];

        self.status_message = Some(format!("Query returned {} series", self.results.len()));
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::QueryEdit => self.handle_query_edit_key(key).await,
            InputMode::TimeRangeSelect => self.handle_time_range_key(key),
            InputMode::DashboardSelect => self.handle_dashboard_select_key(key),
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') => return true,

            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
            }

            KeyCode::Char('e') => {
                self.input_mode = InputMode::QueryEdit;
            }

            KeyCode::Enter => {
                if self.view == View::Query && !self.query.is_empty() {
                    self.execute_query().await;
                } else if self.view == View::MetricBrowser {
                    if let Some(metric) = self.filtered_metrics().get(self.selected_metric) {
                        self.query = metric.to_string();
                        self.query_cursor = self.query.len();
                        self.view = View::Query;
                    }
                }
            }

            KeyCode::Char('t') => {
                self.input_mode = InputMode::TimeRangeSelect;
            }

            KeyCode::Char('r') => {
                self.refresh().await;
                self.status_message = Some("Refreshed".to_string());
            }

            KeyCode::Char('R') => {
                self.auto_refresh = !self.auto_refresh;
                self.status_message = Some(format!(
                    "Auto-refresh: {}",
                    if self.auto_refresh { "on" } else { "off" }
                ));
            }

            KeyCode::Char('a') => {
                self.view = View::Alerts;
                self.selected_alert = 0;
            }

            KeyCode::Char('d') => {
                self.input_mode = InputMode::DashboardSelect;
            }

            KeyCode::Char('m') => {
                self.view = View::MetricBrowser;
                self.selected_metric = 0;
            }

            KeyCode::Tab => {
                self.view = match self.view {
                    View::Query => View::Dashboard,
                    View::Dashboard => View::Alerts,
                    View::Alerts => View::MetricBrowser,
                    View::MetricBrowser => View::Query,
                };
            }

            KeyCode::Esc => {
                self.view = View::Query;
            }

            KeyCode::Char('/') => {
                if self.view == View::MetricBrowser {
                    self.metric_filter.clear();
                    self.input_mode = InputMode::QueryEdit; // Reuse for filter
                }
            }

            _ => {}
        }

        false
    }

    async fn handle_query_edit_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.execute_query().await;
            }
            KeyCode::Backspace => {
                if self.query_cursor > 0 {
                    self.query.remove(self.query_cursor - 1);
                    self.query_cursor -= 1;
                }
            }
            KeyCode::Delete => {
                if self.query_cursor < self.query.len() {
                    self.query.remove(self.query_cursor);
                }
            }
            KeyCode::Left => {
                self.query_cursor = self.query_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                self.query_cursor = (self.query_cursor + 1).min(self.query.len());
            }
            KeyCode::Home => {
                self.query_cursor = 0;
            }
            KeyCode::End => {
                self.query_cursor = self.query.len();
            }
            KeyCode::Up => {
                self.history_up();
            }
            KeyCode::Down => {
                self.history_down();
            }
            KeyCode::Char(c) => {
                self.query.insert(self.query_cursor, c);
                self.query_cursor += 1;
            }
            _ => {}
        }
        false
    }

    fn handle_time_range_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_preset =
                    (self.selected_preset + 1).min(self.time_range_presets.len() - 1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_preset = self.selected_preset.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(preset) = self.time_range_presets.get(self.selected_preset) {
                    self.time_range = TimeRange::last(preset.duration);
                    self.status_message = Some(format!("Time range: {}", preset.label));
                }
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        false
    }

    fn handle_dashboard_select_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(idx) = self.active_dashboard {
                    self.active_dashboard = Some((idx + 1).min(self.dashboards.len() - 1));
                } else if !self.dashboards.is_empty() {
                    self.active_dashboard = Some(0);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(idx) = self.active_dashboard {
                    self.active_dashboard = Some(idx.saturating_sub(1));
                }
            }
            KeyCode::Enter => {
                if self.active_dashboard.is_some() {
                    self.view = View::Dashboard;
                    self.selected_panel = 0;
                }
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        false
    }

    fn navigate_down(&mut self) {
        match self.view {
            View::Query => {
                if self.selected_result < self.results.len().saturating_sub(1) {
                    self.selected_result += 1;
                }
            }
            View::Dashboard => {
                if let Some(dashboard) = self.current_dashboard() {
                    if self.selected_panel < dashboard.panels.len().saturating_sub(1) {
                        self.selected_panel += 1;
                    }
                }
            }
            View::Alerts => {
                if self.selected_alert < self.alerts.len().saturating_sub(1) {
                    self.selected_alert += 1;
                }
            }
            View::MetricBrowser => {
                if self.selected_metric < self.filtered_metrics().len().saturating_sub(1) {
                    self.selected_metric += 1;
                }
            }
        }
    }

    fn navigate_up(&mut self) {
        match self.view {
            View::Query => {
                self.selected_result = self.selected_result.saturating_sub(1);
            }
            View::Dashboard => {
                self.selected_panel = self.selected_panel.saturating_sub(1);
            }
            View::Alerts => {
                self.selected_alert = self.selected_alert.saturating_sub(1);
            }
            View::MetricBrowser => {
                self.selected_metric = self.selected_metric.saturating_sub(1);
            }
        }
    }

    fn history_up(&mut self) {
        if self.query_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            Some(idx) => idx.saturating_sub(1),
            None => self.query_history.len() - 1,
        };

        self.history_index = Some(new_index);
        self.query = self.query_history[new_index].clone();
        self.query_cursor = self.query.len();
    }

    fn history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx + 1 < self.query_history.len() {
                self.history_index = Some(idx + 1);
                self.query = self.query_history[idx + 1].clone();
            } else {
                self.history_index = None;
                self.query.clear();
            }
            self.query_cursor = self.query.len();
        }
    }

    pub fn current_dashboard(&self) -> Option<&Dashboard> {
        self.active_dashboard
            .and_then(|idx| self.dashboards.get(idx))
    }

    pub fn filtered_metrics(&self) -> Vec<&String> {
        if self.metric_filter.is_empty() {
            self.metric_names.iter().collect()
        } else {
            let filter = self.metric_filter.to_lowercase();
            self.metric_names
                .iter()
                .filter(|m| m.to_lowercase().contains(&filter))
                .collect()
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Query => format!(
                "Query | range:{} | e:edit t:time r:refresh m:metrics d:dashboard q:quit",
                self.time_range.display()
            ),
            View::Dashboard => format!(
                "{} | Tab:next panel d:select dashboard q:back",
                self.current_dashboard()
                    .map(|d| d.name.as_str())
                    .unwrap_or("No dashboard")
            ),
            View::Alerts => format!(
                "{} alerts | j/k:nav q:back",
                self.alerts.len()
            ),
            View::MetricBrowser => format!(
                "{} metrics | Enter:select /:filter q:back",
                self.filtered_metrics().len()
            ),
        }
    }
}
