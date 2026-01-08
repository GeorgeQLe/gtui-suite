use crossterm::event::{KeyCode, KeyEvent};
use chrono::Utc;
use std::net::IpAddr;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::scanner::{get_service_name, Host, Port, PortState, ScanResult, Scanner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Disclaimer,
    Hosts,
    HostDetail,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    TargetInput,
    PortInput,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,
    pub input_buffer: String,

    // Hosts data
    pub hosts: Vec<Host>,
    pub selected_host: usize,
    pub selected_port: usize,

    // Scan state
    pub scanning: bool,
    pub scan_progress: (usize, usize), // (scanned, total)
    scan_receiver: Option<mpsc::Receiver<ScanResult>>,

    // Port configuration
    pub target_ip: String,
    pub port_range: String,

    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let view = if config.disclaimer_accepted {
            View::Hosts
        } else {
            View::Disclaimer
        };

        Self {
            config,
            view,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            hosts: Vec::new(),
            selected_host: 0,
            selected_port: 0,
            scanning: false,
            scan_progress: (0, 0),
            scan_receiver: None,
            target_ip: "127.0.0.1".to_string(),
            port_range: "common".to_string(),
            message: None,
            error: None,
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::TargetInput => self.handle_target_input(key),
            InputMode::PortInput => self.handle_port_input(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match self.view {
            View::Disclaimer => self.handle_disclaimer_key(key),
            View::Hosts => self.handle_hosts_key(key),
            View::HostDetail => self.handle_detail_key(key),
            View::Help => self.handle_help_key(key),
        }
    }

    fn handle_disclaimer_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.config.disclaimer_accepted = true;
                let _ = self.config.save();
                self.view = View::Hosts;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('q') => {
                return true; // Quit
            }
            _ => {}
        }
        false
    }

    fn handle_hosts_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_host < self.hosts.len().saturating_sub(1) {
                    self.selected_host += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_host > 0 {
                    self.selected_host -= 1;
                }
            }
            KeyCode::Enter => {
                if !self.hosts.is_empty() {
                    self.view = View::HostDetail;
                    self.selected_port = 0;
                }
            }

            // Scan controls
            KeyCode::Char('s') => {
                if !self.scanning {
                    self.input_mode = InputMode::TargetInput;
                    self.input_buffer = self.target_ip.clone();
                }
            }
            KeyCode::Char('S') => {
                // Stop scan (just clear receiver)
                self.scan_receiver = None;
                self.scanning = false;
                self.message = Some("Scan stopped".to_string());
            }
            KeyCode::Char('p') => {
                self.input_mode = InputMode::PortInput;
                self.input_buffer = self.port_range.clone();
            }
            KeyCode::Char('c') => {
                // Clear hosts
                self.hosts.clear();
                self.selected_host = 0;
            }

            _ => {}
        }
        false
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.view = View::Hosts;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(host) = self.hosts.get(self.selected_host) {
                    if self.selected_port < host.ports.len().saturating_sub(1) {
                        self.selected_port += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_port > 0 {
                    self.selected_port -= 1;
                }
            }
            KeyCode::Char('r') => {
                // Rescan this host
                if let Some(host) = self.hosts.get(self.selected_host) {
                    let ip = host.ip;
                    self.start_scan(ip);
                }
            }
            _ => {}
        }
        false
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Char('?') => {
                self.view = View::Hosts;
            }
            _ => {}
        }
        false
    }

    fn handle_target_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.target_ip = self.input_buffer.clone();
                self.input_mode = InputMode::Normal;

                // Parse and start scan
                if let Ok(ip) = self.target_ip.parse::<IpAddr>() {
                    self.start_scan(ip);
                } else {
                    self.error = Some(format!("Invalid IP: {}", self.target_ip));
                }
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_port_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.port_range = self.input_buffer.clone();
                self.input_mode = InputMode::Normal;
                self.message = Some(format!("Port range set: {}", self.port_range));
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn start_scan(&mut self, ip: IpAddr) {
        let ports = self.parse_port_range();
        if ports.is_empty() {
            self.error = Some("No valid ports to scan".to_string());
            return;
        }

        let scanner = Scanner::new(
            self.config.scan.timeout_ms,
            self.config.scan.max_concurrent,
        );

        self.scan_progress = (0, ports.len());
        self.scanning = true;
        self.message = Some(format!("Scanning {} ({} ports)...", ip, ports.len()));

        let receiver = scanner.scan_ports(ip, ports);
        self.scan_receiver = Some(receiver);
    }

    fn parse_port_range(&self) -> Vec<u16> {
        let range = &self.port_range;

        if range == "common" {
            return self.config.ports.common.clone();
        }

        if range == "top100" {
            return vec![
                7, 9, 13, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111,
                113, 119, 135, 139, 143, 144, 179, 199, 389, 427, 443, 444, 445,
                465, 513, 514, 515, 543, 544, 548, 554, 587, 631, 646, 873, 990,
                993, 995, 1025, 1026, 1027, 1028, 1029, 1110, 1433, 1720, 1723,
                1755, 1900, 2000, 2001, 2049, 2121, 2717, 3000, 3128, 3306, 3389,
                3986, 4899, 5000, 5009, 5051, 5060, 5101, 5190, 5357, 5432, 5631,
                5666, 5800, 5900, 6000, 6001, 6646, 7070, 8000, 8008, 8009, 8080,
                8081, 8443, 8888, 9100, 9999, 10000, 32768, 49152, 49153, 49154,
            ];
        }

        let mut ports = Vec::new();

        for part in range.split(',') {
            let part = part.trim();

            if part.contains('-') {
                // Range like "1-1024"
                let parts: Vec<&str> = part.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                        for p in start..=end {
                            ports.push(p);
                        }
                    }
                }
            } else if let Ok(p) = part.parse::<u16>() {
                ports.push(p);
            }
        }

        ports
    }

    pub async fn check_scan_progress(&mut self) {
        let mut results = Vec::new();
        let mut completed = false;

        if let Some(ref mut receiver) = self.scan_receiver {
            // Non-blocking receive - collect results first
            while let Ok(result) = receiver.try_recv() {
                if matches!(result, ScanResult::ScanCompleted) {
                    completed = true;
                }
                results.push(result);
            }
        }

        // Process results outside the borrow
        for result in results {
            match result {
                ScanResult::PortScanned { ip, port, state } => {
                    if state == PortState::Open {
                        // Find or create host
                        let host = self.hosts.iter_mut().find(|h| h.ip == ip);
                        let host = match host {
                            Some(h) => h,
                            None => {
                                self.hosts.push(Host {
                                    ip,
                                    hostname: None,
                                    ports: Vec::new(),
                                    last_seen: Utc::now(),
                                });
                                self.hosts.last_mut().unwrap()
                            }
                        };

                        // Add port
                        if !host.ports.iter().any(|p| p.number == port) {
                            host.ports.push(Port {
                                number: port,
                                state,
                                service: get_service_name(port).map(|s| s.to_string()),
                            });
                            host.ports.sort_by_key(|p| p.number);
                        }
                        host.last_seen = Utc::now();
                    }
                }
                ScanResult::Progress { scanned, total } => {
                    self.scan_progress = (scanned, total);
                }
                ScanResult::HostCompleted { ip } => {
                    self.message = Some(format!("Completed scan of {}", ip));
                }
                ScanResult::ScanCompleted => {
                    // Handled after loop
                }
            }
        }

        if completed {
            self.scanning = false;
            self.scan_receiver = None;
            self.message = Some("Scan completed".to_string());
        }
    }

    pub fn current_host(&self) -> Option<&Host> {
        self.hosts.get(self.selected_host)
    }
}
