use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct RdpConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub domain: Option<String>,
    pub resolution: String,
    pub fullscreen: bool,
    pub last_connected: Option<String>,
}

pub struct App {
    pub connections: Vec<RdpConnection>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            connections: vec![
                RdpConnection { name: "Work Desktop".into(), host: "192.168.1.50".into(), port: 3389, username: "admin".into(), domain: Some("CORP".into()), resolution: "1920x1080".into(), fullscreen: true, last_connected: Some("2024-01-15".into()) },
                RdpConnection { name: "Dev Server".into(), host: "dev.internal.company.com".into(), port: 3389, username: "developer".into(), domain: None, resolution: "1600x900".into(), fullscreen: false, last_connected: Some("2024-01-14".into()) },
                RdpConnection { name: "Home PC".into(), host: "192.168.0.100".into(), port: 3389, username: "user".into(), domain: None, resolution: "2560x1440".into(), fullscreen: true, last_connected: None },
                RdpConnection { name: "Test Environment".into(), host: "test-vm.local".into(), port: 3389, username: "tester".into(), domain: Some("TEST".into()), resolution: "1280x720".into(), fullscreen: false, last_connected: Some("2024-01-10".into()) },
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
                    self.status_message = Some(format!("Would connect to {}...", conn.host));
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
            KeyCode::Char('f') => {
                if let Some(conn) = self.connections.get_mut(self.selected) {
                    conn.fullscreen = !conn.fullscreen;
                }
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav enter:connect a:add e:edit d:delete f:fullscreen q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
