use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum TunnelType { Local, Remote, Dynamic }

#[derive(Debug, Clone, PartialEq)]
pub enum TunnelStatus { Active, Inactive, Connecting, Error }

#[derive(Debug, Clone)]
pub struct SshTunnel {
    pub name: String,
    pub tunnel_type: TunnelType,
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub status: TunnelStatus,
    pub auto_reconnect: bool,
}

pub struct App {
    pub tunnels: Vec<SshTunnel>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            tunnels: vec![
                SshTunnel { name: "DB Local Forward".into(), tunnel_type: TunnelType::Local, ssh_host: "bastion.example.com".into(), ssh_port: 22, ssh_user: "admin".into(), local_port: 5432, remote_host: "db.internal".into(), remote_port: 5432, status: TunnelStatus::Active, auto_reconnect: true },
                SshTunnel { name: "Redis Forward".into(), tunnel_type: TunnelType::Local, ssh_host: "bastion.example.com".into(), ssh_port: 22, ssh_user: "admin".into(), local_port: 6379, remote_host: "redis.internal".into(), remote_port: 6379, status: TunnelStatus::Active, auto_reconnect: true },
                SshTunnel { name: "Dev SOCKS Proxy".into(), tunnel_type: TunnelType::Dynamic, ssh_host: "jump.dev.example.com".into(), ssh_port: 22, ssh_user: "developer".into(), local_port: 1080, remote_host: "".into(), remote_port: 0, status: TunnelStatus::Inactive, auto_reconnect: false },
                SshTunnel { name: "Expose Local API".into(), tunnel_type: TunnelType::Remote, ssh_host: "public.example.com".into(), ssh_port: 22, ssh_user: "api".into(), local_port: 3000, remote_host: "0.0.0.0".into(), remote_port: 8080, status: TunnelStatus::Inactive, auto_reconnect: true },
                SshTunnel { name: "Internal Web".into(), tunnel_type: TunnelType::Local, ssh_host: "vpn.company.com".into(), ssh_port: 22, ssh_user: "user".into(), local_port: 8080, remote_host: "intranet.internal".into(), remote_port: 80, status: TunnelStatus::Error, auto_reconnect: true },
            ],
            selected: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.tunnels.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(tunnel) = self.tunnels.get_mut(self.selected) {
                    tunnel.status = match tunnel.status {
                        TunnelStatus::Active => TunnelStatus::Inactive,
                        TunnelStatus::Inactive | TunnelStatus::Error => TunnelStatus::Connecting,
                        TunnelStatus::Connecting => TunnelStatus::Active,
                    };
                    self.status_message = Some(format!("Tunnel '{}': {:?}", tunnel.name, tunnel.status));
                }
            },
            KeyCode::Char('a') => self.status_message = Some("Would add tunnel...".into()),
            KeyCode::Char('e') => self.status_message = Some("Would edit tunnel...".into()),
            KeyCode::Char('d') => {
                if !self.tunnels.is_empty() {
                    self.tunnels.remove(self.selected);
                    self.selected = self.selected.min(self.tunnels.len().saturating_sub(1));
                }
            },
            KeyCode::Char('r') => {
                if let Some(tunnel) = self.tunnels.get_mut(self.selected) {
                    tunnel.auto_reconnect = !tunnel.auto_reconnect;
                }
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav space:toggle a:add e:edit d:delete r:auto-reconnect q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
