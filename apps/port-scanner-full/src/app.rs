use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::net::{IpAddr, Ipv4Addr};

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Targets,
    Scanning,
    Results,
    Details,
    Profiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Input,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,

    // Target configuration
    pub targets: Vec<ScanTarget>,
    pub selected_target: usize,
    pub target_input: String,

    // Scan configuration
    pub scan_type: ScanType,
    pub profile: ScanProfile,
    pub timing: TimingTemplate,
    pub version_detection: bool,
    pub os_detection: bool,

    // Scan state
    pub scanning: bool,
    pub progress: Option<ScanProgress>,
    pub current_scan: Option<ScanResult>,

    // Results
    pub scan_history: Vec<ScanResult>,
    pub selected_result: usize,
    pub selected_port: usize,

    // UI state
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Targets,
            input_mode: InputMode::Normal,
            targets: Vec::new(),
            selected_target: 0,
            target_input: String::new(),
            scan_type: ScanType::ConnectScan,
            profile: ScanProfile::Quick,
            timing: TimingTemplate::T3,
            version_detection: true,
            os_detection: false,
            scanning: false,
            progress: None,
            current_scan: None,
            scan_history: Vec::new(),
            selected_result: 0,
            selected_port: 0,
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        // Add demo target
        if self.targets.is_empty() {
            self.targets.push(ScanTarget::new("localhost").with_top_ports(100));
        }

        // Add demo scan history
        if self.scan_history.is_empty() {
            self.scan_history.push(create_demo_scan_result());
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Global shortcuts
        if let KeyCode::Char('q') = key.code {
            if is_ctrl || self.input_mode == InputMode::Normal {
                return true;
            }
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Input => self.handle_input_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match self.view {
            View::Targets => self.handle_targets_key(key),
            View::Scanning => self.handle_scanning_key(key),
            View::Results => self.handle_results_key(key),
            View::Details => self.handle_details_key(key),
            View::Profiles => self.handle_profiles_key(key),
        }
    }

    fn handle_targets_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_target < self.targets.len().saturating_sub(1) {
                    self.selected_target += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_target = self.selected_target.saturating_sub(1);
            }
            KeyCode::Char('a') => {
                self.input_mode = InputMode::Input;
                self.target_input.clear();
            }
            KeyCode::Char('d') => {
                if !self.targets.is_empty() {
                    self.targets.remove(self.selected_target);
                    if self.selected_target >= self.targets.len() && self.selected_target > 0 {
                        self.selected_target -= 1;
                    }
                }
            }
            KeyCode::Char('s') => {
                self.start_scan();
            }
            KeyCode::Char('p') => {
                self.view = View::Profiles;
            }
            KeyCode::Char('r') => {
                self.view = View::Results;
            }
            KeyCode::Tab => {
                self.view = View::Results;
            }
            _ => {}
        }
        false
    }

    fn handle_scanning_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('S') | KeyCode::Esc => {
                self.stop_scan();
            }
            _ => {}
        }
        false
    }

    fn handle_results_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_result < self.scan_history.len().saturating_sub(1) {
                    self.selected_result += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_result = self.selected_result.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !self.scan_history.is_empty() {
                    self.view = View::Details;
                    self.selected_port = 0;
                }
            }
            KeyCode::Tab => {
                self.view = View::Targets;
            }
            KeyCode::Char('x') => {
                self.export_results();
            }
            _ => {}
        }
        false
    }

    fn handle_details_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(result) = self.scan_history.get(self.selected_result) {
                    if self.selected_port < result.ports.len().saturating_sub(1) {
                        self.selected_port += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_port = self.selected_port.saturating_sub(1);
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.view = View::Results;
            }
            _ => {}
        }
        false
    }

    fn handle_profiles_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('1') => {
                self.profile = ScanProfile::Quick;
                self.view = View::Targets;
            }
            KeyCode::Char('2') => {
                self.profile = ScanProfile::Standard;
                self.view = View::Targets;
            }
            KeyCode::Char('3') => {
                self.profile = ScanProfile::Comprehensive;
                self.view = View::Targets;
            }
            KeyCode::Char('4') => {
                self.profile = ScanProfile::Stealth;
                self.view = View::Targets;
            }
            KeyCode::Esc => {
                self.view = View::Targets;
            }
            _ => {}
        }
        false
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.target_input.clear();
            }
            KeyCode::Enter => {
                if !self.target_input.is_empty() {
                    let target = ScanTarget::new(&self.target_input)
                        .with_top_ports(self.profile.port_count());
                    self.targets.push(target);
                    self.target_input.clear();
                    self.input_mode = InputMode::Normal;
                }
            }
            KeyCode::Backspace => {
                self.target_input.pop();
            }
            KeyCode::Char(c) => {
                self.target_input.push(c);
            }
            _ => {}
        }
        false
    }

    fn start_scan(&mut self) {
        if self.targets.is_empty() {
            self.status_message = Some("No targets configured".to_string());
            return;
        }

        let target = &self.targets[self.selected_target];
        let mut result = ScanResult::new(&target.host, self.scan_type, self.profile);
        result.ip = Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        // Simulate scan results
        for port in target.ports.iter().take(20) {
            let state = if *port == 22 || *port == 80 || *port == 443 || *port == 8080 {
                PortState::Open
            } else if *port % 7 == 0 {
                PortState::Filtered
            } else {
                PortState::Closed
            };

            let mut port_result = PortResult::new(*port, Protocol::Tcp, state);
            if state == PortState::Open {
                if let Some(service_name) = get_common_service(*port) {
                    let mut service = ServiceInfo::new(service_name);
                    match *port {
                        22 => {
                            service.product = Some("OpenSSH".to_string());
                            service.version = Some("8.9".to_string());
                        }
                        80 | 8080 => {
                            service.product = Some("nginx".to_string());
                            service.version = Some("1.24.0".to_string());
                        }
                        443 => {
                            service.product = Some("nginx".to_string());
                            service.version = Some("1.24.0".to_string());
                            service.extra_info = Some("TLS 1.3".to_string());
                        }
                        _ => {}
                    }
                    port_result.service = Some(service);
                }
                port_result.response_time_ms = Some(1 + (*port as u64 % 10));
            }
            result.ports.push(port_result);
        }

        // Sort by port number
        result.ports.sort_by_key(|p| p.port);

        // Add OS detection if enabled
        if self.os_detection {
            result.os_detection = Some(OsDetection::new("Linux", "Linux", 95));
        }

        result.completed_at = Some(chrono::Utc::now());
        result.hostname = Some("localhost".to_string());

        self.scan_history.insert(0, result);
        self.selected_result = 0;
        self.view = View::Results;
        self.status_message = Some("Scan completed".to_string());
    }

    fn stop_scan(&mut self) {
        self.scanning = false;
        self.progress = None;
        self.view = View::Targets;
        self.status_message = Some("Scan stopped".to_string());
    }

    fn export_results(&mut self) {
        if let Some(result) = self.scan_history.get(self.selected_result) {
            let filename = format!(
                "scan_{}_{}.json",
                result.target.replace('.', "_"),
                result.started_at.format("%Y%m%d_%H%M%S")
            );
            self.status_message = Some(format!("Exported to {}", filename));
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Targets => format!(
                "{} targets | Profile: {} | s:scan a:add d:delete p:profile Tab:results",
                self.targets.len(),
                self.profile.as_str()
            ),
            View::Scanning => {
                if let Some(ref progress) = self.progress {
                    format!(
                        "Scanning... {:.1}% ({} open) | {:.0} pps | S:stop",
                        progress.percent_complete(),
                        progress.open_found,
                        progress.packets_per_second
                    )
                } else {
                    "Scanning... | S:stop".to_string()
                }
            }
            View::Results => format!(
                "{} scans | Enter:details x:export Tab:targets",
                self.scan_history.len()
            ),
            View::Details => "j/k:navigate Esc:back".to_string(),
            View::Profiles => "1-4:select profile Esc:cancel".to_string(),
        }
    }

    pub fn current_result(&self) -> Option<&ScanResult> {
        self.scan_history.get(self.selected_result)
    }
}

fn create_demo_scan_result() -> ScanResult {
    let mut result = ScanResult::new("192.168.1.1", ScanType::ConnectScan, ScanProfile::Quick);
    result.ip = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    result.hostname = Some("gateway".to_string());

    let demo_ports = vec![
        (22, PortState::Open, Some("ssh")),
        (53, PortState::Open, Some("dns")),
        (80, PortState::Open, Some("http")),
        (443, PortState::Open, Some("https")),
        (8080, PortState::Filtered, Some("http-proxy")),
    ];

    for (port, state, service_name) in demo_ports {
        let mut port_result = PortResult::new(port, Protocol::Tcp, state);
        if let Some(name) = service_name {
            port_result.service = Some(ServiceInfo::new(name));
        }
        if state == PortState::Open {
            port_result.response_time_ms = Some(2);
        }
        result.ports.push(port_result);
    }

    result.os_detection = Some(OsDetection::new("Linux 5.x", "Linux", 92));
    result.completed_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));

    result
}
