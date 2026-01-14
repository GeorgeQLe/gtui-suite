use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum RuleAction { Allow, Deny, Reject, Drop }

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol { Tcp, Udp, Icmp, Any }

#[derive(Debug, Clone, PartialEq)]
pub enum Direction { In, Out, Both }

#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub id: u32,
    pub action: RuleAction,
    pub direction: Direction,
    pub protocol: Protocol,
    pub source: String,
    pub destination: String,
    pub port: Option<String>,
    pub enabled: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Chain { Input, Output, Forward }

pub struct App {
    pub rules: Vec<FirewallRule>,
    pub selected: usize,
    pub chain: Chain,
    pub firewall_enabled: bool,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            rules: vec![
                FirewallRule { id: 1, action: RuleAction::Allow, direction: Direction::In, protocol: Protocol::Tcp, source: "any".into(), destination: "any".into(), port: Some("22".into()), enabled: true, comment: Some("SSH".into()) },
                FirewallRule { id: 2, action: RuleAction::Allow, direction: Direction::In, protocol: Protocol::Tcp, source: "any".into(), destination: "any".into(), port: Some("80,443".into()), enabled: true, comment: Some("HTTP/HTTPS".into()) },
                FirewallRule { id: 3, action: RuleAction::Allow, direction: Direction::In, protocol: Protocol::Tcp, source: "192.168.1.0/24".into(), destination: "any".into(), port: Some("5432".into()), enabled: true, comment: Some("PostgreSQL (LAN only)".into()) },
                FirewallRule { id: 4, action: RuleAction::Allow, direction: Direction::In, protocol: Protocol::Udp, source: "any".into(), destination: "any".into(), port: Some("53".into()), enabled: true, comment: Some("DNS".into()) },
                FirewallRule { id: 5, action: RuleAction::Allow, direction: Direction::In, protocol: Protocol::Icmp, source: "any".into(), destination: "any".into(), port: None, enabled: true, comment: Some("Ping".into()) },
                FirewallRule { id: 6, action: RuleAction::Deny, direction: Direction::In, protocol: Protocol::Tcp, source: "10.0.0.0/8".into(), destination: "any".into(), port: Some("3306".into()), enabled: true, comment: Some("Block MySQL from internal".into()) },
                FirewallRule { id: 7, action: RuleAction::Allow, direction: Direction::Out, protocol: Protocol::Any, source: "any".into(), destination: "any".into(), port: None, enabled: true, comment: Some("Allow all outbound".into()) },
            ],
            selected: 0,
            chain: Chain::Input,
            firewall_enabled: true,
            modified: false,
            status_message: None,
        }
    }

    pub fn filtered_rules(&self) -> Vec<&FirewallRule> {
        self.rules.iter()
            .filter(|r| match self.chain {
                Chain::Input => matches!(r.direction, Direction::In | Direction::Both),
                Chain::Output => matches!(r.direction, Direction::Out | Direction::Both),
                Chain::Forward => false, // No forward rules in demo
            })
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_rules();
                if self.selected < filtered.len().saturating_sub(1) { self.selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Tab => {
                self.chain = match self.chain {
                    Chain::Input => Chain::Output,
                    Chain::Output => Chain::Forward,
                    Chain::Forward => Chain::Input,
                };
                self.selected = 0;
            },
            KeyCode::Char(' ') => {
                let filtered: Vec<u32> = self.filtered_rules().iter().map(|r| r.id).collect();
                if let Some(&id) = filtered.get(self.selected) {
                    if let Some(rule) = self.rules.iter_mut().find(|r| r.id == id) {
                        rule.enabled = !rule.enabled;
                        self.modified = true;
                    }
                }
            },
            KeyCode::Char('f') => {
                self.firewall_enabled = !self.firewall_enabled;
                self.status_message = Some(format!("Firewall {}", if self.firewall_enabled { "enabled" } else { "disabled" }));
            },
            KeyCode::Char('a') => self.status_message = Some("Would add rule...".into()),
            KeyCode::Char('e') => self.status_message = Some("Would edit rule...".into()),
            KeyCode::Char('d') => self.status_message = Some("Would delete rule...".into()),
            KeyCode::Char('r') => self.status_message = Some("Rules reloaded".into()),
            KeyCode::Char('s') => {
                self.modified = false;
                self.status_message = Some("Rules saved".into());
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| {
            format!("{}j/k:nav tab:chain space:toggle f:firewall({}) a:add e:edit d:del s:save q:quit",
                if self.modified { "[*] " } else { "" },
                if self.firewall_enabled { "on" } else { "off" })
        })
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
