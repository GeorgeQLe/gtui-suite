use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum VncQuality { Auto, Low, Medium, High }

#[derive(Debug, Clone)]
pub struct VncConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub password_saved: bool,
    pub quality: VncQuality,
    pub view_only: bool,
    pub last_connected: Option<String>,
}

pub struct App {
    pub connections: Vec<VncConnection>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            connections: vec![
                VncConnection { name: "Home Server".into(), host: "192.168.1.10".into(), port: 5900, password_saved: true, quality: VncQuality::Auto, view_only: false, last_connected: Some("2024-01-15".into()) },
                VncConnection { name: "Raspberry Pi".into(), host: "192.168.1.50".into(), port: 5901, password_saved: true, quality: VncQuality::Low, view_only: false, last_connected: Some("2024-01-14".into()) },
                VncConnection { name: "Office Desktop".into(), host: "office.vpn.local".into(), port: 5900, password_saved: false, quality: VncQuality::High, view_only: false, last_connected: None },
                VncConnection { name: "Test VM".into(), host: "192.168.100.5".into(), port: 5902, password_saved: true, quality: VncQuality::Medium, view_only: true, last_connected: Some("2024-01-10".into()) },
            ],
            selected: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.connections.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter | KeyCode::Char('c') => {
                if let Some(conn) = self.connections.get(self.selected) {
                    self.status_message = Some(format!("Would connect to {}:{}...", conn.host, conn.port));
                }
            },
            KeyCode::Char('a') => self.status_message = Some("Would add connection...".into()),
            KeyCode::Char('e') => self.status_message = Some("Would edit connection...".into()),
            KeyCode::Char('d') => {
                if !self.connections.is_empty() {
                    self.connections.remove(self.selected);
                    self.selected = self.selected.min(self.connections.len().saturating_sub(1));
                }
            },
            KeyCode::Char('v') => {
                if let Some(conn) = self.connections.get_mut(self.selected) {
                    conn.view_only = !conn.view_only;
                }
            },
            KeyCode::Char('y') => {
                if let Some(conn) = self.connections.get_mut(self.selected) {
                    conn.quality = match conn.quality {
                        VncQuality::Auto => VncQuality::Low,
                        VncQuality::Low => VncQuality::Medium,
                        VncQuality::Medium => VncQuality::High,
                        VncQuality::High => VncQuality::Auto,
                    };
                }
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav enter:connect a:add e:edit d:delete v:view-only y:quality q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
