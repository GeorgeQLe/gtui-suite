use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct HostEntry { pub ip: String, pub hostnames: Vec<String> }

pub struct App {
    pub hostname: String,
    pub hosts: Vec<HostEntry>,
    pub selected: usize,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            hostname: "myhost".into(),
            hosts: vec![
                HostEntry { ip: "127.0.0.1".into(), hostnames: vec!["localhost".into()] },
                HostEntry { ip: "::1".into(), hostnames: vec!["localhost".into(), "ip6-localhost".into()] },
                HostEntry { ip: "192.168.1.100".into(), hostnames: vec!["server".into(), "server.local".into()] },
            ],
            selected: 0, modified: false, status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.hosts.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('h') => self.status_message = Some("Would edit hostname...".into()),
            KeyCode::Char('e') => self.status_message = Some("Would edit entry...".into()),
            KeyCode::Char('a') => { self.status_message = Some("Would add entry...".into()); self.modified = true; },
            KeyCode::Char('d') => if !self.hosts.is_empty() { self.hosts.remove(self.selected); self.selected = self.selected.min(self.hosts.len().saturating_sub(1)); self.modified = true; },
            KeyCode::Char('s') => { self.modified = false; self.status_message = Some("Saved".into()); },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| format!("{}j/k:nav h:hostname e:edit a:add d:delete s:save q:quit", if self.modified { "[*] " } else { "" }))
    }
}
impl Default for App { fn default() -> Self { Self::new() } }
