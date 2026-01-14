use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct FstabEntry {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub options: String,
    pub dump: u8,
    pub pass: u8,
}

pub struct App {
    pub entries: Vec<FstabEntry>,
    pub selected: usize,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            entries: vec![
                FstabEntry { device: "/dev/sda1".into(), mount_point: "/boot".into(), fs_type: "ext4".into(), options: "defaults".into(), dump: 0, pass: 2 },
                FstabEntry { device: "/dev/sda2".into(), mount_point: "/".into(), fs_type: "ext4".into(), options: "defaults".into(), dump: 0, pass: 1 },
                FstabEntry { device: "/dev/sda3".into(), mount_point: "/home".into(), fs_type: "ext4".into(), options: "defaults".into(), dump: 0, pass: 2 },
                FstabEntry { device: "tmpfs".into(), mount_point: "/tmp".into(), fs_type: "tmpfs".into(), options: "defaults,size=2G".into(), dump: 0, pass: 0 },
            ],
            selected: 0, modified: false, status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.entries.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('e') => self.status_message = Some("Would edit entry...".into()),
            KeyCode::Char('a') => { self.status_message = Some("Would add entry...".into()); self.modified = true; },
            KeyCode::Char('d') => if !self.entries.is_empty() { self.entries.remove(self.selected); self.selected = self.selected.min(self.entries.len().saturating_sub(1)); self.modified = true; },
            KeyCode::Char('v') => self.status_message = Some("Validating fstab...".into()),
            KeyCode::Char('s') => { self.modified = false; self.status_message = Some("Saved /etc/fstab".into()); },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| format!("{}j/k:nav e:edit a:add d:delete v:validate s:save q:quit", if self.modified { "[*] " } else { "" }))
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
