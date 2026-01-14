use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub version: String,
    pub path: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SharedDep {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub used_by: Vec<String>,
}

pub struct App {
    pub members: Vec<WorkspaceMember>,
    pub shared_deps: Vec<SharedDep>,
    pub selected: usize,
    pub view_deps: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            members: create_demo_members(),
            shared_deps: create_demo_shared_deps(),
            selected: 0,
            view_deps: false,
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
                let max = if self.view_deps {
                    self.shared_deps.len()
                } else {
                    self.members.len()
                };
                if self.selected < max.saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Tab => {
                self.view_deps = !self.view_deps;
                self.selected = 0;
                self.status_message = Some(format!(
                    "View: {}",
                    if self.view_deps { "Shared Dependencies" } else { "Workspace Members" }
                ));
            }
            KeyCode::Char('u') => {
                if self.view_deps {
                    if let Some(dep) = self.shared_deps.get_mut(self.selected) {
                        self.status_message = Some(format!("Would update {} to latest", dep.name));
                    }
                }
            }
            KeyCode::Char('a') => {
                if self.view_deps {
                    self.status_message = Some("Would add new shared dependency...".to_string());
                } else {
                    self.status_message = Some("Would add new workspace member...".to_string());
                }
            }
            KeyCode::Char('y') => {
                if self.view_deps {
                    if let Some(dep) = self.shared_deps.get(self.selected) {
                        self.status_message = Some(format!("Copied: {} = \"{}\"", dep.name, dep.version));
                    }
                } else if let Some(member) = self.members.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", member.name));
                }
            }
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "Tab:switch-view j/k:nav u:update a:add y:copy q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_members() -> Vec<WorkspaceMember> {
    vec![
        WorkspaceMember {
            name: "my-app".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/my-app".to_string(),
            dependencies: vec!["my-lib".to_string(), "my-utils".to_string()],
        },
        WorkspaceMember {
            name: "my-lib".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/my-lib".to_string(),
            dependencies: vec!["my-utils".to_string()],
        },
        WorkspaceMember {
            name: "my-utils".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/my-utils".to_string(),
            dependencies: vec![],
        },
        WorkspaceMember {
            name: "my-cli".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/my-cli".to_string(),
            dependencies: vec!["my-app".to_string()],
        },
    ]
}

fn create_demo_shared_deps() -> Vec<SharedDep> {
    vec![
        SharedDep {
            name: "serde".to_string(),
            version: "1.0".to_string(),
            features: vec!["derive".to_string()],
            used_by: vec!["my-app".to_string(), "my-lib".to_string()],
        },
        SharedDep {
            name: "tokio".to_string(),
            version: "1".to_string(),
            features: vec!["full".to_string()],
            used_by: vec!["my-app".to_string(), "my-cli".to_string()],
        },
        SharedDep {
            name: "anyhow".to_string(),
            version: "1".to_string(),
            features: vec![],
            used_by: vec!["my-app".to_string(), "my-lib".to_string(), "my-cli".to_string()],
        },
        SharedDep {
            name: "thiserror".to_string(),
            version: "2".to_string(),
            features: vec![],
            used_by: vec!["my-lib".to_string()],
        },
    ]
}
