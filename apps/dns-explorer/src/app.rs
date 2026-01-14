use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Input,
    Results,
}

pub struct App {
    pub domain: String,
    pub records: Vec<DnsRecord>,
    pub selected: usize,
    pub view: View,
    pub history: Vec<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            domain: String::new(),
            records: Vec::new(),
            selected: 0,
            view: View::Input,
            history: Vec::new(),
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::Input => self.handle_input_key(key),
            View::Results => self.handle_results_key(key),
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') if self.domain.is_empty() => return true,
            KeyCode::Esc => return true,
            KeyCode::Enter => {
                if !self.domain.is_empty() {
                    self.lookup();
                }
            }
            KeyCode::Backspace => {
                self.domain.pop();
            }
            KeyCode::Char(c) => {
                self.domain.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_results_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Input;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.records.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('y') => {
                if let Some(record) = self.records.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", record.value));
                }
            }
            KeyCode::Char('n') => {
                self.view = View::Input;
                self.domain.clear();
            }
            _ => {}
        }
        false
    }

    fn lookup(&mut self) {
        self.records = create_demo_records(&self.domain);
        self.history.push(self.domain.clone());
        self.selected = 0;
        self.view = View::Results;
        self.status_message = Some(format!("Found {} records", self.records.len()));
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Input => "Enter domain to lookup | Esc:quit".to_string(),
            View::Results => "j/k:navigate y:copy n:new-query Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_records(domain: &str) -> Vec<DnsRecord> {
    vec![
        DnsRecord {
            record_type: "A".to_string(),
            name: domain.to_string(),
            value: "93.184.216.34".to_string(),
            ttl: 3600,
        },
        DnsRecord {
            record_type: "AAAA".to_string(),
            name: domain.to_string(),
            value: "2606:2800:220:1:248:1893:25c8:1946".to_string(),
            ttl: 3600,
        },
        DnsRecord {
            record_type: "MX".to_string(),
            name: domain.to_string(),
            value: "10 mail.example.com".to_string(),
            ttl: 3600,
        },
        DnsRecord {
            record_type: "NS".to_string(),
            name: domain.to_string(),
            value: "ns1.example.com".to_string(),
            ttl: 86400,
        },
        DnsRecord {
            record_type: "TXT".to_string(),
            name: domain.to_string(),
            value: "v=spf1 include:_spf.example.com ~all".to_string(),
            ttl: 3600,
        },
    ]
}
