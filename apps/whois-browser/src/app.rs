use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct WhoisResult {
    pub query: String,
    pub timestamp: DateTime<Local>,
    pub registrar: Option<String>,
    pub creation_date: Option<String>,
    pub expiration_date: Option<String>,
    pub name_servers: Vec<String>,
    pub status: Vec<String>,
    pub raw_data: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode { List, Detail }

pub struct App {
    pub history: Vec<WhoisResult>,
    pub selected: usize,
    pub view_mode: ViewMode,
    pub scroll_offset: usize,
    pub input_buffer: String,
    pub is_inputting: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            history: vec![
                WhoisResult {
                    query: "google.com".into(),
                    timestamp: Local::now(),
                    registrar: Some("MarkMonitor Inc.".into()),
                    creation_date: Some("1997-09-15".into()),
                    expiration_date: Some("2028-09-14".into()),
                    name_servers: vec!["ns1.google.com".into(), "ns2.google.com".into(), "ns3.google.com".into(), "ns4.google.com".into()],
                    status: vec!["clientDeleteProhibited".into(), "clientTransferProhibited".into(), "clientUpdateProhibited".into()],
                    raw_data: "Domain Name: GOOGLE.COM\nRegistry Domain ID: 2138514_DOMAIN_COM-VRSN\nRegistrar: MarkMonitor Inc.\n...".into(),
                },
                WhoisResult {
                    query: "github.com".into(),
                    timestamp: Local::now(),
                    registrar: Some("MarkMonitor Inc.".into()),
                    creation_date: Some("2007-10-09".into()),
                    expiration_date: Some("2026-10-09".into()),
                    name_servers: vec!["dns1.p08.nsone.net".into(), "dns2.p08.nsone.net".into()],
                    status: vec!["clientDeleteProhibited".into(), "clientTransferProhibited".into()],
                    raw_data: "Domain Name: GITHUB.COM\nRegistry Domain ID: 1264983250_DOMAIN_COM-VRSN\nRegistrar: MarkMonitor Inc.\n...".into(),
                },
                WhoisResult {
                    query: "8.8.8.8".into(),
                    timestamp: Local::now(),
                    registrar: None,
                    creation_date: None,
                    expiration_date: None,
                    name_servers: vec![],
                    status: vec![],
                    raw_data: "NetRange: 8.0.0.0 - 8.255.255.255\nCIDR: 8.0.0.0/8\nNetName: LVLT-ORG-8-8\nOrganization: Google LLC\n...".into(),
                },
            ],
            selected: 0,
            view_mode: ViewMode::List,
            scroll_offset: 0,
            input_buffer: String::new(),
            is_inputting: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }

        if self.is_inputting {
            match key.code {
                KeyCode::Esc => self.is_inputting = false,
                KeyCode::Enter => {
                    if !self.input_buffer.is_empty() {
                        // Would perform actual WHOIS lookup
                        self.history.insert(0, WhoisResult {
                            query: self.input_buffer.clone(),
                            timestamp: Local::now(),
                            registrar: Some("Example Registrar".into()),
                            creation_date: Some("2020-01-01".into()),
                            expiration_date: Some("2025-01-01".into()),
                            name_servers: vec!["ns1.example.com".into()],
                            status: vec!["ok".into()],
                            raw_data: format!("Domain Name: {}\n...", self.input_buffer.to_uppercase()),
                        });
                        self.input_buffer.clear();
                        self.selected = 0;
                    }
                    self.is_inputting = false;
                },
                KeyCode::Backspace => { self.input_buffer.pop(); },
                KeyCode::Char(c) => self.input_buffer.push(c),
                _ => {}
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.view_mode == ViewMode::Detail {
                    self.view_mode = ViewMode::List;
                } else {
                    return true;
                }
            },
            KeyCode::Char('j') | KeyCode::Down => {
                if self.view_mode == ViewMode::List {
                    if self.selected < self.history.len().saturating_sub(1) { self.selected += 1; }
                } else {
                    self.scroll_offset += 1;
                }
            },
            KeyCode::Char('k') | KeyCode::Up => {
                if self.view_mode == ViewMode::List {
                    self.selected = self.selected.saturating_sub(1);
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
            },
            KeyCode::Enter => {
                if self.view_mode == ViewMode::List && !self.history.is_empty() {
                    self.view_mode = ViewMode::Detail;
                    self.scroll_offset = 0;
                }
            },
            KeyCode::Char('n') | KeyCode::Char('/') => {
                self.is_inputting = true;
                self.input_buffer.clear();
            },
            KeyCode::Char('d') => {
                if !self.history.is_empty() {
                    self.history.remove(self.selected);
                    self.selected = self.selected.min(self.history.len().saturating_sub(1));
                }
            },
            KeyCode::Char('c') => { self.history.clear(); self.selected = 0; },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        if self.is_inputting {
            format!("Query: {}_ [Enter:search Esc:cancel]", self.input_buffer)
        } else {
            match self.view_mode {
                ViewMode::List => "j/k:nav enter:detail n:new-query d:delete c:clear q:quit".into(),
                ViewMode::Detail => "j/k:scroll esc:back q:quit".into(),
            }
        }
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
