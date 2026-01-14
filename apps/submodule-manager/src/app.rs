use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    UpToDate,
    Behind,
    Ahead,
    Modified,
    Uninitialized,
}

impl SyncStatus {
    pub fn name(&self) -> &'static str {
        match self {
            SyncStatus::UpToDate => "Up to date",
            SyncStatus::Behind => "Behind",
            SyncStatus::Ahead => "Ahead",
            SyncStatus::Modified => "Modified",
            SyncStatus::Uninitialized => "Not initialized",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Submodule {
    pub name: String,
    pub path: String,
    pub url: String,
    pub branch: String,
    pub commit: String,
    pub status: SyncStatus,
}

pub struct App {
    pub submodules: Vec<Submodule>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            submodules: create_demo_submodules(),
            selected: 0,
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
                if self.selected < self.submodules.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('i') => {
                if let Some(sm) = self.submodules.get_mut(self.selected) {
                    if sm.status == SyncStatus::Uninitialized {
                        sm.status = SyncStatus::UpToDate;
                        self.status_message = Some(format!("Initialized: {}", sm.name));
                    } else {
                        self.status_message = Some("Already initialized".to_string());
                    }
                }
            }
            KeyCode::Char('u') => {
                if let Some(sm) = self.submodules.get_mut(self.selected) {
                    sm.status = SyncStatus::UpToDate;
                    self.status_message = Some(format!("Updated: {}", sm.name));
                }
            }
            KeyCode::Char('s') => {
                for sm in &mut self.submodules {
                    sm.status = SyncStatus::UpToDate;
                }
                self.status_message = Some("All submodules synced".to_string());
            }
            KeyCode::Char('a') => {
                self.status_message = Some("Would add new submodule...".to_string());
            }
            KeyCode::Char('d') => {
                if !self.submodules.is_empty() {
                    let name = self.submodules[self.selected].name.clone();
                    self.submodules.remove(self.selected);
                    self.selected = self.selected.min(self.submodules.len().saturating_sub(1));
                    self.status_message = Some(format!("Removed: {}", name));
                }
            }
            KeyCode::Char('y') => {
                if let Some(sm) = self.submodules.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", sm.url));
                }
            }
            _ => {}
        }
        false
    }

    pub fn out_of_sync(&self) -> usize {
        self.submodules.iter().filter(|s| s.status != SyncStatus::UpToDate).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav i:init u:update s:sync-all a:add d:remove y:copy q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_submodules() -> Vec<Submodule> {
    vec![
        Submodule {
            name: "lib-core".to_string(),
            path: "libs/core".to_string(),
            url: "https://github.com/example/lib-core.git".to_string(),
            branch: "main".to_string(),
            commit: "a1b2c3d".to_string(),
            status: SyncStatus::UpToDate,
        },
        Submodule {
            name: "lib-utils".to_string(),
            path: "libs/utils".to_string(),
            url: "https://github.com/example/lib-utils.git".to_string(),
            branch: "main".to_string(),
            commit: "e4f5g6h".to_string(),
            status: SyncStatus::Behind,
        },
        Submodule {
            name: "vendor-theme".to_string(),
            path: "vendor/theme".to_string(),
            url: "https://github.com/example/theme.git".to_string(),
            branch: "v2".to_string(),
            commit: "i7j8k9l".to_string(),
            status: SyncStatus::Modified,
        },
        Submodule {
            name: "docs".to_string(),
            path: "docs/external".to_string(),
            url: "https://github.com/example/docs.git".to_string(),
            branch: "main".to_string(),
            commit: "".to_string(),
            status: SyncStatus::Uninitialized,
        },
    ]
}
