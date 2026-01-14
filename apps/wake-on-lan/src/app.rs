use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceStatus { Online, Offline, Unknown, Waking }

#[derive(Debug, Clone)]
pub struct WolDevice {
    pub name: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub broadcast_ip: String,
    pub port: u16,
    pub status: DeviceStatus,
    pub last_wake: Option<String>,
}

pub struct App {
    pub devices: Vec<WolDevice>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            devices: vec![
                WolDevice { name: "Server".into(), mac_address: "AA:BB:CC:DD:EE:01".into(), ip_address: Some("192.168.1.10".into()), broadcast_ip: "192.168.1.255".into(), port: 9, status: DeviceStatus::Offline, last_wake: Some("2024-01-15 08:00".into()) },
                WolDevice { name: "NAS".into(), mac_address: "AA:BB:CC:DD:EE:02".into(), ip_address: Some("192.168.1.20".into()), broadcast_ip: "192.168.1.255".into(), port: 9, status: DeviceStatus::Online, last_wake: None },
                WolDevice { name: "Gaming PC".into(), mac_address: "AA:BB:CC:DD:EE:03".into(), ip_address: Some("192.168.1.100".into()), broadcast_ip: "192.168.1.255".into(), port: 9, status: DeviceStatus::Offline, last_wake: Some("2024-01-14 20:30".into()) },
                WolDevice { name: "Media Center".into(), mac_address: "AA:BB:CC:DD:EE:04".into(), ip_address: None, broadcast_ip: "192.168.1.255".into(), port: 7, status: DeviceStatus::Unknown, last_wake: None },
                WolDevice { name: "Office Workstation".into(), mac_address: "AA:BB:CC:DD:EE:05".into(), ip_address: Some("10.0.0.50".into()), broadcast_ip: "10.0.0.255".into(), port: 9, status: DeviceStatus::Offline, last_wake: Some("2024-01-13 09:00".into()) },
            ],
            selected: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.devices.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter | KeyCode::Char('w') => {
                if let Some(device) = self.devices.get_mut(self.selected) {
                    device.status = DeviceStatus::Waking;
                    self.status_message = Some(format!("Sending magic packet to {}...", device.mac_address));
                }
            },
            KeyCode::Char('W') => {
                // Wake all offline devices
                for device in &mut self.devices {
                    if device.status == DeviceStatus::Offline {
                        device.status = DeviceStatus::Waking;
                    }
                }
                self.status_message = Some("Waking all offline devices...".into());
            },
            KeyCode::Char('p') => {
                if let Some(device) = self.devices.get_mut(self.selected) {
                    // Would ping device
                    self.status_message = Some(format!("Pinging {}...", device.ip_address.as_deref().unwrap_or("unknown")));
                }
            },
            KeyCode::Char('a') => self.status_message = Some("Would add device...".into()),
            KeyCode::Char('e') => self.status_message = Some("Would edit device...".into()),
            KeyCode::Char('d') => {
                if !self.devices.is_empty() {
                    self.devices.remove(self.selected);
                    self.selected = self.selected.min(self.devices.len().saturating_sub(1));
                }
            },
            KeyCode::Char('r') => {
                for device in &mut self.devices {
                    device.status = DeviceStatus::Unknown;
                }
                self.status_message = Some("Refreshing device status...".into());
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav w:wake W:wake-all p:ping a:add e:edit d:delete r:refresh q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
