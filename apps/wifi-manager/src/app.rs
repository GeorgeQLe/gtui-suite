use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Network {
    pub ssid: String,
    pub signal_strength: i32,
    pub security: Security,
    pub connected: bool,
    pub saved: bool,
    pub frequency: String,
    pub bssid: String,
}

impl Network {
    pub fn signal_bars(&self) -> u8 {
        if self.signal_strength >= -50 {
            4
        } else if self.signal_strength >= -60 {
            3
        } else if self.signal_strength >= -70 {
            2
        } else {
            1
        }
    }

    pub fn signal_icon(&self) -> &'static str {
        match self.signal_bars() {
            4 => "████",
            3 => "███░",
            2 => "██░░",
            _ => "█░░░",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
}

impl Security {
    pub fn as_str(&self) -> &'static str {
        match self {
            Security::Open => "Open",
            Security::WEP => "WEP",
            Security::WPA => "WPA",
            Security::WPA2 => "WPA2",
            Security::WPA3 => "WPA3",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Security::Open => "🔓",
            _ => "🔒",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Networks,
    Details,
    Connect,
}

pub struct App {
    pub networks: Vec<Network>,
    pub selected: usize,
    pub view: View,
    pub password_input: String,
    pub show_password: bool,
    pub is_scanning: bool,
    pub wifi_enabled: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            networks: Vec::new(),
            selected: 0,
            view: View::Networks,
            password_input: String::new(),
            show_password: false,
            is_scanning: false,
            wifi_enabled: true,
            status_message: None,
        }
    }

    pub async fn scan(&mut self) {
        self.is_scanning = true;
        self.networks = create_demo_networks();
        self.networks.sort_by(|a, b| {
            b.connected.cmp(&a.connected)
                .then(b.signal_strength.cmp(&a.signal_strength))
        });
        self.is_scanning = false;
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::Networks => self.handle_networks_key(key).await,
            View::Details => self.handle_details_key(key),
            View::Connect => self.handle_connect_key(key).await,
        }
    }

    async fn handle_networks_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.networks.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(network) = self.networks.get(self.selected) {
                    if network.connected {
                        self.view = View::Details;
                    } else if network.security == Security::Open || network.saved {
                        self.connect_to_network().await;
                    } else {
                        self.view = View::Connect;
                        self.password_input.clear();
                    }
                }
            }
            KeyCode::Char('i') => {
                self.view = View::Details;
            }
            KeyCode::Char('d') => {
                self.disconnect().await;
            }
            KeyCode::Char('f') => {
                self.forget_network();
            }
            KeyCode::Char('r') => {
                self.scan().await;
            }
            KeyCode::Char('t') => {
                self.wifi_enabled = !self.wifi_enabled;
                self.status_message = Some(format!(
                    "WiFi {}",
                    if self.wifi_enabled { "enabled" } else { "disabled" }
                ));
            }
            _ => {}
        }
        false
    }

    fn handle_details_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Networks;
            }
            KeyCode::Char('d') => {
                self.view = View::Networks;
            }
            _ => {}
        }
        false
    }

    async fn handle_connect_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::Networks;
                self.password_input.clear();
            }
            KeyCode::Enter => {
                self.connect_to_network().await;
            }
            KeyCode::Backspace => {
                self.password_input.pop();
            }
            KeyCode::Char(c) => {
                self.password_input.push(c);
            }
            KeyCode::Tab => {
                self.show_password = !self.show_password;
            }
            _ => {}
        }
        false
    }

    async fn connect_to_network(&mut self) {
        let ssid = self.networks.get(self.selected).map(|n| n.ssid.clone());

        // Disconnect all networks first
        for n in &mut self.networks {
            n.connected = false;
        }

        // Connect the selected one
        if let Some(network) = self.networks.get_mut(self.selected) {
            network.connected = true;
            network.saved = true;
        }

        if let Some(ssid) = ssid {
            self.status_message = Some(format!("Connected to {}", ssid));
        }
        self.view = View::Networks;
        self.password_input.clear();
    }

    async fn disconnect(&mut self) {
        if let Some(network) = self.networks.get_mut(self.selected) {
            if network.connected {
                network.connected = false;
                self.status_message = Some(format!("Disconnected from {}", network.ssid));
            }
        }
    }

    fn forget_network(&mut self) {
        if let Some(network) = self.networks.get_mut(self.selected) {
            if network.saved {
                network.saved = false;
                network.connected = false;
                self.status_message = Some(format!("Forgot network {}", network.ssid));
            }
        }
    }

    pub fn selected_network(&self) -> Option<&Network> {
        self.networks.get(self.selected)
    }

    pub fn connected_network(&self) -> Option<&Network> {
        self.networks.iter().find(|n| n.connected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        if self.is_scanning {
            return "Scanning...".to_string();
        }

        match self.view {
            View::Networks => {
                "Enter:connect i:info d:disconnect f:forget r:scan t:toggle wifi".to_string()
            }
            View::Details => "Esc:back d:disconnect".to_string(),
            View::Connect => "Enter:connect Tab:show/hide password Esc:cancel".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_networks() -> Vec<Network> {
    vec![
        Network {
            ssid: "HomeNetwork".to_string(),
            signal_strength: -45,
            security: Security::WPA2,
            connected: true,
            saved: true,
            frequency: "5 GHz".to_string(),
            bssid: "AA:BB:CC:DD:EE:FF".to_string(),
        },
        Network {
            ssid: "Office-5G".to_string(),
            signal_strength: -55,
            security: Security::WPA3,
            connected: false,
            saved: true,
            frequency: "5 GHz".to_string(),
            bssid: "11:22:33:44:55:66".to_string(),
        },
        Network {
            ssid: "CoffeeShop_Free".to_string(),
            signal_strength: -65,
            security: Security::Open,
            connected: false,
            saved: false,
            frequency: "2.4 GHz".to_string(),
            bssid: "77:88:99:AA:BB:CC".to_string(),
        },
        Network {
            ssid: "Neighbor's WiFi".to_string(),
            signal_strength: -72,
            security: Security::WPA2,
            connected: false,
            saved: false,
            frequency: "2.4 GHz".to_string(),
            bssid: "DD:EE:FF:00:11:22".to_string(),
        },
        Network {
            ssid: "Guest".to_string(),
            signal_strength: -58,
            security: Security::WPA,
            connected: false,
            saved: true,
            frequency: "2.4 GHz".to_string(),
            bssid: "33:44:55:66:77:88".to_string(),
        },
        Network {
            ssid: "IoT-Network".to_string(),
            signal_strength: -80,
            security: Security::WPA2,
            connected: false,
            saved: false,
            frequency: "2.4 GHz".to_string(),
            bssid: "99:AA:BB:CC:DD:EE".to_string(),
        },
    ]
}
