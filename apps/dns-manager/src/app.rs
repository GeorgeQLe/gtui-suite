use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordType { A, AAAA, CNAME, MX, TXT, NS, PTR, SRV }

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: RecordType,
    pub value: String,
    pub ttl: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct DnsZone {
    pub name: String,
    pub records: Vec<DnsRecord>,
}

pub struct App {
    pub zones: Vec<DnsZone>,
    pub selected_zone: usize,
    pub selected_record: usize,
    pub service_status: String,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            zones: vec![
                DnsZone {
                    name: "local.lan".into(),
                    records: vec![
                        DnsRecord { name: "@".into(), record_type: RecordType::A, value: "192.168.1.1".into(), ttl: 3600, enabled: true },
                        DnsRecord { name: "router".into(), record_type: RecordType::A, value: "192.168.1.1".into(), ttl: 3600, enabled: true },
                        DnsRecord { name: "nas".into(), record_type: RecordType::A, value: "192.168.1.10".into(), ttl: 3600, enabled: true },
                        DnsRecord { name: "printer".into(), record_type: RecordType::A, value: "192.168.1.20".into(), ttl: 3600, enabled: true },
                        DnsRecord { name: "*.apps".into(), record_type: RecordType::CNAME, value: "nas.local.lan".into(), ttl: 3600, enabled: true },
                    ],
                },
                DnsZone {
                    name: "168.192.in-addr.arpa".into(),
                    records: vec![
                        DnsRecord { name: "1.1".into(), record_type: RecordType::PTR, value: "router.local.lan".into(), ttl: 3600, enabled: true },
                        DnsRecord { name: "10.1".into(), record_type: RecordType::PTR, value: "nas.local.lan".into(), ttl: 3600, enabled: true },
                    ],
                },
            ],
            selected_zone: 0,
            selected_record: 0,
            service_status: "running".into(),
            modified: false,
            status_message: None,
        }
    }

    pub fn current_zone(&self) -> Option<&DnsZone> {
        self.zones.get(self.selected_zone)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(zone) = self.current_zone() {
                    if self.selected_record < zone.records.len().saturating_sub(1) {
                        self.selected_record += 1;
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected_record = self.selected_record.saturating_sub(1),
            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
                self.selected_zone = (self.selected_zone + 1) % self.zones.len().max(1);
                self.selected_record = 0;
            },
            KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
                self.selected_zone = self.selected_zone.checked_sub(1).unwrap_or(self.zones.len().saturating_sub(1));
                self.selected_record = 0;
            },
            KeyCode::Char(' ') => {
                if let Some(zone) = self.zones.get_mut(self.selected_zone) {
                    if let Some(record) = zone.records.get_mut(self.selected_record) {
                        record.enabled = !record.enabled;
                        self.modified = true;
                    }
                }
            },
            KeyCode::Char('a') => self.status_message = Some("Would add record...".into()),
            KeyCode::Char('e') => self.status_message = Some("Would edit record...".into()),
            KeyCode::Char('d') => {
                if let Some(zone) = self.zones.get_mut(self.selected_zone) {
                    if !zone.records.is_empty() {
                        zone.records.remove(self.selected_record);
                        self.selected_record = self.selected_record.min(zone.records.len().saturating_sub(1));
                        self.modified = true;
                    }
                }
            },
            KeyCode::Char('z') => self.status_message = Some("Would add zone...".into()),
            KeyCode::Char('r') => self.status_message = Some("Reloading DNS service...".into()),
            KeyCode::Char('s') => {
                self.modified = false;
                self.status_message = Some("Configuration saved".into());
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| {
            format!("{}h/l:zone j/k:record space:toggle a:add e:edit d:delete z:zone r:reload s:save q:quit",
                if self.modified { "[*] " } else { "" })
        })
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
