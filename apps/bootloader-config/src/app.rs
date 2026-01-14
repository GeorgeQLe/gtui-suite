use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct BootEntry {
    pub name: String,
    pub kernel: String,
    pub initrd: String,
    pub options: String,
    pub is_default: bool,
}

pub struct App {
    pub entries: Vec<BootEntry>,
    pub selected: usize,
    pub timeout: u32,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            entries: vec![
                BootEntry {
                    name: "Arch Linux".to_string(),
                    kernel: "/vmlinuz-linux".to_string(),
                    initrd: "/initramfs-linux.img".to_string(),
                    options: "root=/dev/sda2 rw quiet".to_string(),
                    is_default: true,
                },
                BootEntry {
                    name: "Arch Linux (fallback)".to_string(),
                    kernel: "/vmlinuz-linux".to_string(),
                    initrd: "/initramfs-linux-fallback.img".to_string(),
                    options: "root=/dev/sda2 rw".to_string(),
                    is_default: false,
                },
                BootEntry {
                    name: "Windows Boot Manager".to_string(),
                    kernel: "EFI/Microsoft/Boot/bootmgfw.efi".to_string(),
                    initrd: String::new(),
                    options: String::new(),
                    is_default: false,
                },
            ],
            selected: 0,
            timeout: 5,
            modified: false,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.entries.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('d') => {
                for e in &mut self.entries { e.is_default = false; }
                if let Some(e) = self.entries.get_mut(self.selected) {
                    e.is_default = true;
                    self.modified = true;
                    self.status_message = Some(format!("Set default: {}", e.name));
                }
            }
            KeyCode::Char('+') => {
                self.timeout = self.timeout.saturating_add(1).min(30);
                self.modified = true;
                self.status_message = Some(format!("Timeout: {}s", self.timeout));
            }
            KeyCode::Char('-') => {
                self.timeout = self.timeout.saturating_sub(1);
                self.modified = true;
                self.status_message = Some(format!("Timeout: {}s", self.timeout));
            }
            KeyCode::Char('s') => {
                self.modified = false;
                self.status_message = Some("Configuration saved".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(||
            format!("{}j/k:nav d:default +/-:timeout s:save q:quit", if self.modified { "[*] " } else { "" }))
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
