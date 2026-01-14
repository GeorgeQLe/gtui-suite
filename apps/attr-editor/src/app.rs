use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct ExtendedAttr {
    pub namespace: String,
    pub name: String,
    pub value: String,
    pub size: usize,
}

impl ExtendedAttr {
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.namespace, self.name)
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub mode: String,
}

pub struct App {
    pub file: FileInfo,
    pub attrs: Vec<ExtendedAttr>,
    pub selected: usize,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            file: FileInfo {
                path: "/home/user/document.pdf".to_string(),
                size: 1048576,
                mode: "-rw-r--r--".to_string(),
            },
            attrs: create_demo_attrs(),
            selected: 0,
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
                if self.selected < self.attrs.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('a') => {
                self.attrs.push(ExtendedAttr {
                    namespace: "user".to_string(),
                    name: "new_attr".to_string(),
                    value: "value".to_string(),
                    size: 5,
                });
                self.modified = true;
                self.status_message = Some("Added new attribute".to_string());
            }
            KeyCode::Char('d') => {
                if !self.attrs.is_empty() {
                    let attr = self.attrs.remove(self.selected);
                    self.selected = self.selected.min(self.attrs.len().saturating_sub(1));
                    self.modified = true;
                    self.status_message = Some(format!("Deleted: {}", attr.full_name()));
                }
            }
            KeyCode::Char('e') => {
                if let Some(attr) = self.attrs.get(self.selected) {
                    self.status_message = Some(format!("Would edit: {}", attr.full_name()));
                }
            }
            KeyCode::Char('c') => {
                if let Some(attr) = self.attrs.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", attr.value));
                }
            }
            KeyCode::Char('s') => {
                self.modified = false;
                self.status_message = Some("Attributes saved".to_string());
            }
            KeyCode::Char('r') => {
                self.attrs = create_demo_attrs();
                self.modified = false;
                self.status_message = Some("Reloaded from file".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn total_size(&self) -> usize {
        self.attrs.iter().map(|a| a.size).sum()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        let mod_indicator = if self.modified { "[*] " } else { "" };
        format!("{}j/k:nav a:add d:delete e:edit c:copy s:save r:reload q:quit", mod_indicator)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_attrs() -> Vec<ExtendedAttr> {
    vec![
        ExtendedAttr {
            namespace: "user".to_string(),
            name: "mime_type".to_string(),
            value: "application/pdf".to_string(),
            size: 15,
        },
        ExtendedAttr {
            namespace: "user".to_string(),
            name: "author".to_string(),
            value: "John Doe".to_string(),
            size: 8,
        },
        ExtendedAttr {
            namespace: "user".to_string(),
            name: "created".to_string(),
            value: "2024-01-15T10:30:00Z".to_string(),
            size: 20,
        },
        ExtendedAttr {
            namespace: "user".to_string(),
            name: "tags".to_string(),
            value: "important,work,review".to_string(),
            size: 21,
        },
        ExtendedAttr {
            namespace: "security".to_string(),
            name: "selinux".to_string(),
            value: "unconfined_u:object_r:user_home_t:s0".to_string(),
            size: 37,
        },
        ExtendedAttr {
            namespace: "trusted".to_string(),
            name: "checksum".to_string(),
            value: "sha256:abc123def456...".to_string(),
            size: 71,
        },
    ]
}
