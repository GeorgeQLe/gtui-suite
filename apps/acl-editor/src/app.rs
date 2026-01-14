use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AclType {
    User,
    Group,
    Mask,
    Other,
    Default,
}

impl AclType {
    pub fn name(&self) -> &'static str {
        match self {
            AclType::User => "user",
            AclType::Group => "group",
            AclType::Mask => "mask",
            AclType::Other => "other",
            AclType::Default => "default",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AclEntry {
    pub acl_type: AclType,
    pub qualifier: Option<String>,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub is_default: bool,
}

impl AclEntry {
    pub fn permission_string(&self) -> String {
        format!(
            "{}{}{}",
            if self.read { 'r' } else { '-' },
            if self.write { 'w' } else { '-' },
            if self.execute { 'x' } else { '-' }
        )
    }

    pub fn full_string(&self) -> String {
        let prefix = if self.is_default { "default:" } else { "" };
        let qual = self.qualifier.clone().unwrap_or_default();
        format!("{}{}:{}:{}", prefix, self.acl_type.name(), qual, self.permission_string())
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub owner: String,
    pub group: String,
    pub mode: String,
}

pub struct App {
    pub file: FileInfo,
    pub entries: Vec<AclEntry>,
    pub selected: usize,
    pub editing: bool,
    pub modified: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            file: FileInfo {
                path: "/home/user/document.txt".to_string(),
                owner: "user".to_string(),
                group: "users".to_string(),
                mode: "0644".to_string(),
            },
            entries: create_demo_acl(),
            selected: 0,
            editing: false,
            modified: false,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.modified {
                    self.status_message = Some("Unsaved changes! Press 'q' again to quit".to_string());
                    self.modified = false;
                    return false;
                }
                return true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.entries.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('r') => {
                if let Some(entry) = self.entries.get_mut(self.selected) {
                    entry.read = !entry.read;
                    self.modified = true;
                    self.status_message = Some(format!("Read: {}", entry.read));
                }
            }
            KeyCode::Char('w') => {
                if let Some(entry) = self.entries.get_mut(self.selected) {
                    entry.write = !entry.write;
                    self.modified = true;
                    self.status_message = Some(format!("Write: {}", entry.write));
                }
            }
            KeyCode::Char('x') => {
                if let Some(entry) = self.entries.get_mut(self.selected) {
                    entry.execute = !entry.execute;
                    self.modified = true;
                    self.status_message = Some(format!("Execute: {}", entry.execute));
                }
            }
            KeyCode::Char('a') => {
                self.entries.push(AclEntry {
                    acl_type: AclType::User,
                    qualifier: Some("newuser".to_string()),
                    read: true,
                    write: false,
                    execute: false,
                    is_default: false,
                });
                self.modified = true;
                self.status_message = Some("Added new ACL entry".to_string());
            }
            KeyCode::Char('d') => {
                if self.entries.len() > 1 {
                    let entry = self.entries.remove(self.selected);
                    self.selected = self.selected.min(self.entries.len().saturating_sub(1));
                    self.modified = true;
                    self.status_message = Some(format!("Deleted: {}", entry.full_string()));
                }
            }
            KeyCode::Char('s') => {
                self.modified = false;
                self.status_message = Some("ACL saved".to_string());
            }
            KeyCode::Char('R') => {
                self.entries = create_demo_acl();
                self.modified = false;
                self.status_message = Some("ACL reloaded".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn has_extended_acl(&self) -> bool {
        self.entries.iter().any(|e| e.qualifier.is_some())
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        let mod_indicator = if self.modified { "[*] " } else { "" };
        format!("{}j/k:nav r/w/x:toggle a:add d:delete s:save R:reload q:quit", mod_indicator)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_acl() -> Vec<AclEntry> {
    vec![
        AclEntry {
            acl_type: AclType::User,
            qualifier: None,
            read: true,
            write: true,
            execute: false,
            is_default: false,
        },
        AclEntry {
            acl_type: AclType::User,
            qualifier: Some("alice".to_string()),
            read: true,
            write: true,
            execute: false,
            is_default: false,
        },
        AclEntry {
            acl_type: AclType::User,
            qualifier: Some("bob".to_string()),
            read: true,
            write: false,
            execute: false,
            is_default: false,
        },
        AclEntry {
            acl_type: AclType::Group,
            qualifier: None,
            read: true,
            write: false,
            execute: false,
            is_default: false,
        },
        AclEntry {
            acl_type: AclType::Group,
            qualifier: Some("developers".to_string()),
            read: true,
            write: true,
            execute: false,
            is_default: false,
        },
        AclEntry {
            acl_type: AclType::Mask,
            qualifier: None,
            read: true,
            write: true,
            execute: false,
            is_default: false,
        },
        AclEntry {
            acl_type: AclType::Other,
            qualifier: None,
            read: true,
            write: false,
            execute: false,
            is_default: false,
        },
    ]
}
