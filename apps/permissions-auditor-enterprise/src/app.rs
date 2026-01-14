use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Systems,
    Compliance,
    Findings,
    Reports,
    SystemDetails,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,
    pub systems: Vec<System>,
    pub scans: Vec<SystemScan>,
    pub selected_system: usize,
    pub selected_finding: usize,
    pub search_query: String,
    pub severity_filter: Option<Severity>,
    pub stats: ComplianceStats,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Dashboard,
            input_mode: InputMode::Normal,
            systems: Vec::new(),
            scans: Vec::new(),
            selected_system: 0,
            selected_finding: 0,
            search_query: String::new(),
            severity_filter: None,
            stats: ComplianceStats::new(),
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        self.systems = create_demo_systems();
        self.scans = create_demo_scans(&self.systems);

        for system in &mut self.systems {
            if let Some(scan) = self.scans.iter().find(|s| s.system_id == system.id) {
                system.compliance_score = Some(scan.compliance_score);
                system.last_scan = Some(scan.scan_time);
            }
        }

        self.stats.update(&self.systems, &self.scans);
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') if self.input_mode == InputMode::Normal => return true,
            _ => {}
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Search => self.handle_search_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('1') => self.view = View::Dashboard,
            KeyCode::Char('2') => self.view = View::Systems,
            KeyCode::Char('3') => self.view = View::Compliance,
            KeyCode::Char('4') => self.view = View::Findings,
            KeyCode::Char('5') => self.view = View::Reports,
            KeyCode::Char('d') => self.view = View::Dashboard,
            KeyCode::Char('y') => self.view = View::Systems,
            KeyCode::Char('c') => self.view = View::Compliance,
            KeyCode::Char('f') => self.view = View::Findings,
            KeyCode::Char('r') => self.view = View::Reports,
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => self.navigate_down(),
            KeyCode::Char('k') | KeyCode::Up => self.navigate_up(),
            KeyCode::Enter => self.select_item(),
            KeyCode::Esc => {
                if self.view == View::SystemDetails {
                    self.view = View::Systems;
                }
            }
            KeyCode::Char('s') => self.start_scan(),
            KeyCode::Char('S') => self.scan_all_systems(),
            KeyCode::Tab => self.next_view(),
            KeyCode::BackTab => self.prev_view(),
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Enter => {
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
            View::Systems | View::SystemDetails => {
                let max = self.filtered_systems().len().saturating_sub(1);
                if self.selected_system < max {
                    self.selected_system += 1;
                }
            }
            View::Findings => {
                let max = self.all_findings().len().saturating_sub(1);
                if self.selected_finding < max {
                    self.selected_finding += 1;
                }
            }
            _ => {}
        }
    }

    fn navigate_up(&mut self) {
        match self.view {
            View::Systems | View::SystemDetails => {
                self.selected_system = self.selected_system.saturating_sub(1);
            }
            View::Findings => {
                self.selected_finding = self.selected_finding.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn select_item(&mut self) {
        match self.view {
            View::Systems => {
                self.view = View::SystemDetails;
                self.selected_finding = 0;
            }
            _ => {}
        }
    }

    fn next_view(&mut self) {
        self.view = match self.view {
            View::Dashboard => View::Systems,
            View::Systems => View::Compliance,
            View::Compliance => View::Findings,
            View::Findings => View::Reports,
            View::Reports => View::Dashboard,
            View::SystemDetails => View::Systems,
        };
    }

    fn prev_view(&mut self) {
        self.view = match self.view {
            View::Dashboard => View::Reports,
            View::Systems => View::Dashboard,
            View::Compliance => View::Systems,
            View::Findings => View::Compliance,
            View::Reports => View::Findings,
            View::SystemDetails => View::Systems,
        };
    }

    fn start_scan(&mut self) {
        let system_name = if let Some(system) = self.systems.get(self.selected_system) {
            system.name.clone()
        } else {
            return;
        };

        self.status_message = Some(format!("Starting scan on {}...", system_name));

        let mut scan = {
            let system = self.systems.get_mut(self.selected_system).unwrap();
            system.status = SystemStatus::Scanning;
            system.status = SystemStatus::Online;
            let mut scan = SystemScan::new(system);
            let checks = get_cis_checks();

            for check in checks {
                let status = if rand_bool(0.8) {
                    FindingStatus::Pass
                } else {
                    FindingStatus::Fail
                };
                scan.findings.push(Finding::new(check, status));
            }

            scan.calculate_score();
            scan.completed_at = Some(Utc::now());
            system.compliance_score = Some(scan.compliance_score);
            system.last_scan = Some(scan.scan_time);
            scan
        };

        self.scans.push(scan);
        self.stats.update(&self.systems, &self.scans);
        self.status_message = Some(format!("Scan complete for {}", system_name));
    }

    fn scan_all_systems(&mut self) {
        self.status_message = Some(format!("Scanning {} systems...", self.systems.len()));

        for system in &mut self.systems {
            system.status = SystemStatus::Scanning;
        }

        let mut new_scans = Vec::new();

        for i in 0..self.systems.len() {
            let scan = {
                let system = self.systems.get_mut(i).unwrap();
                system.status = SystemStatus::Online;
                let mut scan = SystemScan::new(system);
                let checks = get_cis_checks();

                for check in checks {
                    let status = if rand_bool(0.75) {
                        FindingStatus::Pass
                    } else {
                        FindingStatus::Fail
                    };
                    scan.findings.push(Finding::new(check, status));
                }

                scan.calculate_score();
                scan.completed_at = Some(Utc::now());
                system.compliance_score = Some(scan.compliance_score);
                system.last_scan = Some(scan.scan_time);
                scan
            };

            new_scans.push(scan);
        }

        self.scans.extend(new_scans);
        self.stats.update(&self.systems, &self.scans);
        self.status_message = Some("All scans complete".to_string());
    }

    pub fn filtered_systems(&self) -> Vec<&System> {
        if self.search_query.is_empty() {
            self.systems.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.systems
                .iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&query)
                        || s.hostname.to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    pub fn all_findings(&self) -> Vec<&Finding> {
        let findings: Vec<&Finding> = self
            .scans
            .iter()
            .flat_map(|s| &s.findings)
            .filter(|f| {
                if let Some(severity) = self.severity_filter {
                    f.check.severity == severity
                } else {
                    true
                }
            })
            .collect();
        findings
    }

    pub fn system_scan(&self) -> Option<&SystemScan> {
        if let Some(system) = self.systems.get(self.selected_system) {
            self.scans.iter().rev().find(|s| s.system_id == system.id)
        } else {
            None
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        let view_text = match self.view {
            View::Dashboard => "Dashboard",
            View::Systems => "Systems",
            View::Compliance => "Compliance",
            View::Findings => "Findings",
            View::Reports => "Reports",
            View::SystemDetails => "System Details",
        };

        format!(
            "{} | 1-5:views Tab:next s:scan S:scan-all /:search q:quit",
            view_text
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_systems() -> Vec<System> {
    vec![
        {
            let mut s = System::new("web-server-01", "web01.example.com");
            s.ip_address = Some("192.168.1.10".to_string());
            s.os = Some("Ubuntu".to_string());
            s.os_version = Some("22.04 LTS".to_string());
            s.status = SystemStatus::Online;
            s
        },
        {
            let mut s = System::new("db-server-01", "db01.example.com");
            s.ip_address = Some("192.168.1.20".to_string());
            s.os = Some("RHEL".to_string());
            s.os_version = Some("8.8".to_string());
            s.status = SystemStatus::Online;
            s
        },
        {
            let mut s = System::new("app-server-01", "app01.example.com");
            s.ip_address = Some("192.168.1.30".to_string());
            s.os = Some("Debian".to_string());
            s.os_version = Some("12".to_string());
            s.status = SystemStatus::Online;
            s
        },
        {
            let mut s = System::new("cache-server-01", "cache01.example.com");
            s.ip_address = Some("192.168.1.40".to_string());
            s.os = Some("Ubuntu".to_string());
            s.os_version = Some("20.04 LTS".to_string());
            s.status = SystemStatus::Offline;
            s
        },
    ]
}

fn create_demo_scans(systems: &[System]) -> Vec<SystemScan> {
    let checks = get_cis_checks();

    systems
        .iter()
        .take(2)
        .map(|system| {
            let mut scan = SystemScan::new(system);

            for check in &checks {
                let status = if rand_bool(0.8) {
                    FindingStatus::Pass
                } else {
                    FindingStatus::Fail
                };
                scan.findings.push(Finding::new(check.clone(), status));
            }

            scan.calculate_score();
            scan.completed_at = Some(Utc::now());
            scan
        })
        .collect()
}

fn rand_bool(probability: f64) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64 / u32::MAX as f64) < probability
}
