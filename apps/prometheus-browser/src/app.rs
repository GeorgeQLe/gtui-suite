use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub help: String,
    pub labels: Vec<String>,
}

#[derive(Clone)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Unknown,
}

impl MetricType {
    pub fn as_str(&self) -> &str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Summary => "summary",
            MetricType::Unknown => "unknown",
        }
    }
}

#[derive(Clone)]
pub struct QueryResult {
    pub metric: String,
    pub labels: Vec<(String, String)>,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct Target {
    pub job: String,
    pub instance: String,
    pub health: TargetHealth,
    pub scrape_url: String,
    pub last_scrape: String,
    pub scrape_duration: String,
}

#[derive(Clone, PartialEq)]
pub enum TargetHealth {
    Up,
    Down,
    Unknown,
}

impl TargetHealth {
    pub fn as_str(&self) -> &str {
        match self {
            TargetHealth::Up => "up",
            TargetHealth::Down => "down",
            TargetHealth::Unknown => "unknown",
        }
    }
}

pub enum Tab {
    Query,
    Metrics,
    Targets,
}

pub struct App {
    pub metrics: Vec<Metric>,
    pub targets: Vec<Target>,
    pub query_results: Vec<QueryResult>,
    pub query_input: String,
    pub selected_metric: usize,
    pub selected_target: usize,
    pub selected_result: usize,
    pub current_tab: Tab,
    pub show_help: bool,
    pub query_history: Vec<String>,
    pub history_index: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        Self {
            metrics: vec![
                Metric {
                    name: "up".to_string(),
                    metric_type: MetricType::Gauge,
                    help: "Whether the target is up".to_string(),
                    labels: vec!["job".to_string(), "instance".to_string()],
                },
                Metric {
                    name: "node_cpu_seconds_total".to_string(),
                    metric_type: MetricType::Counter,
                    help: "Seconds the CPUs spent in each mode".to_string(),
                    labels: vec!["cpu".to_string(), "mode".to_string()],
                },
                Metric {
                    name: "node_memory_MemTotal_bytes".to_string(),
                    metric_type: MetricType::Gauge,
                    help: "Total memory in bytes".to_string(),
                    labels: vec!["instance".to_string()],
                },
                Metric {
                    name: "node_memory_MemAvailable_bytes".to_string(),
                    metric_type: MetricType::Gauge,
                    help: "Available memory in bytes".to_string(),
                    labels: vec!["instance".to_string()],
                },
                Metric {
                    name: "http_requests_total".to_string(),
                    metric_type: MetricType::Counter,
                    help: "Total HTTP requests".to_string(),
                    labels: vec!["method".to_string(), "status".to_string(), "path".to_string()],
                },
                Metric {
                    name: "http_request_duration_seconds".to_string(),
                    metric_type: MetricType::Histogram,
                    help: "HTTP request duration in seconds".to_string(),
                    labels: vec!["method".to_string(), "path".to_string()],
                },
                Metric {
                    name: "process_resident_memory_bytes".to_string(),
                    metric_type: MetricType::Gauge,
                    help: "Resident memory size in bytes".to_string(),
                    labels: vec!["job".to_string()],
                },
                Metric {
                    name: "go_goroutines".to_string(),
                    metric_type: MetricType::Gauge,
                    help: "Number of goroutines".to_string(),
                    labels: vec!["job".to_string()],
                },
            ],
            targets: vec![
                Target {
                    job: "prometheus".to_string(),
                    instance: "localhost:9090".to_string(),
                    health: TargetHealth::Up,
                    scrape_url: "http://localhost:9090/metrics".to_string(),
                    last_scrape: "2024-01-15 10:30:00".to_string(),
                    scrape_duration: "15.2ms".to_string(),
                },
                Target {
                    job: "node".to_string(),
                    instance: "server1:9100".to_string(),
                    health: TargetHealth::Up,
                    scrape_url: "http://server1:9100/metrics".to_string(),
                    last_scrape: "2024-01-15 10:30:00".to_string(),
                    scrape_duration: "45.8ms".to_string(),
                },
                Target {
                    job: "node".to_string(),
                    instance: "server2:9100".to_string(),
                    health: TargetHealth::Up,
                    scrape_url: "http://server2:9100/metrics".to_string(),
                    last_scrape: "2024-01-15 10:30:00".to_string(),
                    scrape_duration: "38.1ms".to_string(),
                },
                Target {
                    job: "api".to_string(),
                    instance: "api-server:8080".to_string(),
                    health: TargetHealth::Down,
                    scrape_url: "http://api-server:8080/metrics".to_string(),
                    last_scrape: "2024-01-15 10:25:00".to_string(),
                    scrape_duration: "0ms".to_string(),
                },
            ],
            query_results: Vec::new(),
            query_input: String::new(),
            selected_metric: 0,
            selected_target: 0,
            selected_result: 0,
            current_tab: Tab::Query,
            show_help: false,
            query_history: vec![
                "up".to_string(),
                "rate(http_requests_total[5m])".to_string(),
                "node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes * 100".to_string(),
            ],
            history_index: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => {
                self.show_help = true;
                return false;
            }
            KeyCode::Tab => {
                self.current_tab = match self.current_tab {
                    Tab::Query => Tab::Metrics,
                    Tab::Metrics => Tab::Targets,
                    Tab::Targets => Tab::Query,
                };
                return false;
            }
            KeyCode::Char('1') => {
                self.current_tab = Tab::Query;
                return false;
            }
            KeyCode::Char('2') => {
                self.current_tab = Tab::Metrics;
                return false;
            }
            KeyCode::Char('3') => {
                self.current_tab = Tab::Targets;
                return false;
            }
            _ => {}
        }

        match &self.current_tab {
            Tab::Query => self.handle_query_key(key),
            Tab::Metrics => self.handle_metrics_key(key),
            Tab::Targets => self.handle_targets_key(key),
        }
    }

    fn handle_query_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.query_input.push('q');
            }
            KeyCode::Char(c) => {
                self.query_input.push(c);
                self.history_index = None;
            }
            KeyCode::Backspace => {
                self.query_input.pop();
            }
            KeyCode::Enter => {
                self.execute_query();
            }
            KeyCode::Up => {
                if !self.query_results.is_empty() {
                    self.selected_result = self.selected_result.saturating_sub(1);
                } else {
                    // Navigate history
                    let history_len = self.query_history.len();
                    if history_len > 0 {
                        self.history_index = Some(match self.history_index {
                            None => history_len - 1,
                            Some(i) => i.saturating_sub(1),
                        });
                        self.query_input = self.query_history[self.history_index.unwrap()].clone();
                    }
                }
            }
            KeyCode::Down => {
                if !self.query_results.is_empty() {
                    if self.selected_result < self.query_results.len().saturating_sub(1) {
                        self.selected_result += 1;
                    }
                } else if let Some(i) = self.history_index {
                    if i < self.query_history.len() - 1 {
                        self.history_index = Some(i + 1);
                        self.query_input = self.query_history[self.history_index.unwrap()].clone();
                    } else {
                        self.history_index = None;
                        self.query_input.clear();
                    }
                }
            }
            KeyCode::Esc => {
                self.query_results.clear();
                self.query_input.clear();
            }
            _ => {}
        }
        false
    }

    fn handle_metrics_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_metric < self.metrics.len().saturating_sub(1) {
                    self.selected_metric += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_metric = self.selected_metric.saturating_sub(1);
            }
            KeyCode::Enter => {
                // Use metric in query
                self.query_input = self.metrics[self.selected_metric].name.clone();
                self.current_tab = Tab::Query;
            }
            _ => {}
        }
        false
    }

    fn handle_targets_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_target < self.targets.len().saturating_sub(1) {
                    self.selected_target += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_target = self.selected_target.saturating_sub(1);
            }
            _ => {}
        }
        false
    }

    fn execute_query(&mut self) {
        if self.query_input.is_empty() {
            return;
        }

        // Add to history
        if !self.query_history.contains(&self.query_input) {
            self.query_history.push(self.query_input.clone());
        }

        // Simulate query results
        self.selected_result = 0;
        self.query_results = vec![
            QueryResult {
                metric: self.query_input.clone(),
                labels: vec![
                    ("instance".to_string(), "server1:9100".to_string()),
                    ("job".to_string(), "node".to_string()),
                ],
                value: 0.85,
                timestamp: 1705312200,
            },
            QueryResult {
                metric: self.query_input.clone(),
                labels: vec![
                    ("instance".to_string(), "server2:9100".to_string()),
                    ("job".to_string(), "node".to_string()),
                ],
                value: 0.72,
                timestamp: 1705312200,
            },
            QueryResult {
                metric: self.query_input.clone(),
                labels: vec![
                    ("instance".to_string(), "server3:9100".to_string()),
                    ("job".to_string(), "node".to_string()),
                ],
                value: 0.91,
                timestamp: 1705312200,
            },
        ];
    }
}
