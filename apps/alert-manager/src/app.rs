use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            AlertSeverity::Critical => "critical",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Info => "info",
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum AlertState {
    Firing,
    Pending,
    Resolved,
}

impl AlertState {
    pub fn as_str(&self) -> &str {
        match self {
            AlertState::Firing => "firing",
            AlertState::Pending => "pending",
            AlertState::Resolved => "resolved",
        }
    }
}

#[derive(Clone)]
pub struct Alert {
    pub name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub labels: Vec<(String, String)>,
    pub annotations: Vec<(String, String)>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub fingerprint: String,
}

#[derive(Clone)]
pub struct Silence {
    pub id: String,
    pub matchers: Vec<String>,
    pub created_by: String,
    pub comment: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub active: bool,
}

pub enum Tab {
    Alerts,
    Silences,
}

pub struct App {
    pub alerts: Vec<Alert>,
    pub silences: Vec<Silence>,
    pub selected_alert: usize,
    pub selected_silence: usize,
    pub current_tab: Tab,
    pub show_help: bool,
    pub show_details: bool,
    pub filter_severity: Option<AlertSeverity>,
}

impl App {
    pub fn new() -> Self {
        let now = Utc::now();

        Self {
            alerts: vec![
                Alert {
                    name: "HighCPUUsage".to_string(),
                    severity: AlertSeverity::Critical,
                    state: AlertState::Firing,
                    labels: vec![
                        ("instance".to_string(), "web-1".to_string()),
                        ("job".to_string(), "node-exporter".to_string()),
                    ],
                    annotations: vec![
                        ("summary".to_string(), "High CPU usage detected".to_string()),
                        ("description".to_string(), "CPU usage is above 90% for 5 minutes".to_string()),
                    ],
                    starts_at: now - chrono::Duration::minutes(15),
                    ends_at: None,
                    fingerprint: "abc123".to_string(),
                },
                Alert {
                    name: "DiskSpaceLow".to_string(),
                    severity: AlertSeverity::Warning,
                    state: AlertState::Firing,
                    labels: vec![
                        ("instance".to_string(), "db-1".to_string()),
                        ("mountpoint".to_string(), "/data".to_string()),
                    ],
                    annotations: vec![
                        ("summary".to_string(), "Disk space running low".to_string()),
                        ("description".to_string(), "Disk usage is above 85%".to_string()),
                    ],
                    starts_at: now - chrono::Duration::hours(2),
                    ends_at: None,
                    fingerprint: "def456".to_string(),
                },
                Alert {
                    name: "MemoryPressure".to_string(),
                    severity: AlertSeverity::Warning,
                    state: AlertState::Pending,
                    labels: vec![
                        ("instance".to_string(), "api-1".to_string()),
                        ("job".to_string(), "node-exporter".to_string()),
                    ],
                    annotations: vec![
                        ("summary".to_string(), "Memory pressure detected".to_string()),
                        ("description".to_string(), "Memory usage is above 80%".to_string()),
                    ],
                    starts_at: now - chrono::Duration::minutes(3),
                    ends_at: None,
                    fingerprint: "ghi789".to_string(),
                },
                Alert {
                    name: "ServiceDown".to_string(),
                    severity: AlertSeverity::Critical,
                    state: AlertState::Resolved,
                    labels: vec![
                        ("service".to_string(), "payment-api".to_string()),
                    ],
                    annotations: vec![
                        ("summary".to_string(), "Service is down".to_string()),
                        ("description".to_string(), "Payment API is not responding".to_string()),
                    ],
                    starts_at: now - chrono::Duration::hours(1),
                    ends_at: Some(now - chrono::Duration::minutes(30)),
                    fingerprint: "jkl012".to_string(),
                },
                Alert {
                    name: "HTTPErrorRate".to_string(),
                    severity: AlertSeverity::Info,
                    state: AlertState::Firing,
                    labels: vec![
                        ("service".to_string(), "frontend".to_string()),
                        ("code".to_string(), "5xx".to_string()),
                    ],
                    annotations: vec![
                        ("summary".to_string(), "Elevated HTTP error rate".to_string()),
                        ("description".to_string(), "5xx error rate above 1%".to_string()),
                    ],
                    starts_at: now - chrono::Duration::minutes(10),
                    ends_at: None,
                    fingerprint: "mno345".to_string(),
                },
            ],
            silences: vec![
                Silence {
                    id: "silence-001".to_string(),
                    matchers: vec!["alertname=MaintenanceWindow".to_string()],
                    created_by: "admin".to_string(),
                    comment: "Planned maintenance window".to_string(),
                    starts_at: now - chrono::Duration::hours(1),
                    ends_at: now + chrono::Duration::hours(2),
                    active: true,
                },
                Silence {
                    id: "silence-002".to_string(),
                    matchers: vec!["instance=test-*".to_string()],
                    created_by: "dev".to_string(),
                    comment: "Silence test environment alerts".to_string(),
                    starts_at: now - chrono::Duration::days(1),
                    ends_at: now + chrono::Duration::days(6),
                    active: true,
                },
            ],
            selected_alert: 0,
            selected_silence: 0,
            current_tab: Tab::Alerts,
            show_help: false,
            show_details: false,
            filter_severity: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        if self.show_details {
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                self.show_details = false;
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Tab => {
                self.current_tab = match self.current_tab {
                    Tab::Alerts => Tab::Silences,
                    Tab::Silences => Tab::Alerts,
                };
            }
            KeyCode::Char('1') => self.current_tab = Tab::Alerts,
            KeyCode::Char('2') => self.current_tab = Tab::Silences,
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Enter => {
                self.show_details = true;
            }
            KeyCode::Char('s') => {
                // Silence alert (demo)
            }
            KeyCode::Char('a') => {
                // Acknowledge alert (demo)
            }
            KeyCode::Char('f') => {
                // Cycle filter
                self.filter_severity = match &self.filter_severity {
                    None => Some(AlertSeverity::Critical),
                    Some(AlertSeverity::Critical) => Some(AlertSeverity::Warning),
                    Some(AlertSeverity::Warning) => Some(AlertSeverity::Info),
                    Some(AlertSeverity::Info) => None,
                };
            }
            _ => {}
        }
        false
    }

    fn move_down(&mut self) {
        match self.current_tab {
            Tab::Alerts => {
                let filtered = self.filtered_alerts();
                if self.selected_alert < filtered.len().saturating_sub(1) {
                    self.selected_alert += 1;
                }
            }
            Tab::Silences => {
                if self.selected_silence < self.silences.len().saturating_sub(1) {
                    self.selected_silence += 1;
                }
            }
        }
    }

    fn move_up(&mut self) {
        match self.current_tab {
            Tab::Alerts => {
                self.selected_alert = self.selected_alert.saturating_sub(1);
            }
            Tab::Silences => {
                self.selected_silence = self.selected_silence.saturating_sub(1);
            }
        }
    }

    pub fn filtered_alerts(&self) -> Vec<&Alert> {
        self.alerts
            .iter()
            .filter(|a| {
                self.filter_severity.is_none()
                    || self.filter_severity.as_ref() == Some(&a.severity)
            })
            .collect()
    }

    pub fn current_alert(&self) -> Option<&Alert> {
        self.filtered_alerts().get(self.selected_alert).copied()
    }
}
