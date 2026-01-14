use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: String,
    pub head: String,
    pub is_main: bool,
    pub is_locked: bool,
    pub is_prunable: bool,
}

pub struct App {
    pub worktrees: Vec<Worktree>,
    pub selected: usize,
    pub show_details: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            worktrees: create_demo_worktrees(),
            selected: 0,
            show_details: false,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                } else {
                    return true;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.worktrees.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.show_details = !self.show_details;
            }
            KeyCode::Char('a') => {
                self.status_message = Some("Would add new worktree...".to_string());
            }
            KeyCode::Char('d') => {
                if let Some(wt) = self.worktrees.get(self.selected) {
                    if !wt.is_main {
                        self.status_message = Some(format!("Would remove worktree: {}", wt.path.display()));
                    } else {
                        self.status_message = Some("Cannot remove main worktree".to_string());
                    }
                }
            }
            KeyCode::Char('l') => {
                if let Some(wt) = self.worktrees.get_mut(self.selected) {
                    wt.is_locked = !wt.is_locked;
                    self.status_message = Some(format!(
                        "Worktree {}: {}",
                        if wt.is_locked { "locked" } else { "unlocked" },
                        wt.path.display()
                    ));
                }
            }
            KeyCode::Char('p') => {
                let prunable: Vec<_> = self.worktrees.iter()
                    .filter(|w| w.is_prunable)
                    .collect();
                self.status_message = Some(format!("Would prune {} worktrees", prunable.len()));
            }
            KeyCode::Char('y') => {
                if let Some(wt) = self.worktrees.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", wt.path.display()));
                }
            }
            _ => {}
        }
        false
    }

    pub fn selected_worktree(&self) -> Option<&Worktree> {
        self.worktrees.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        if self.show_details {
            "Esc:back l:lock/unlock d:remove".to_string()
        } else {
            "j/k:nav Enter:details a:add d:remove l:lock p:prune q:quit".to_string()
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_worktrees() -> Vec<Worktree> {
    vec![
        Worktree {
            path: PathBuf::from("/home/user/project"),
            branch: "main".to_string(),
            head: "a1b2c3d".to_string(),
            is_main: true,
            is_locked: false,
            is_prunable: false,
        },
        Worktree {
            path: PathBuf::from("/home/user/project-feature"),
            branch: "feature/new-ui".to_string(),
            head: "e4f5g6h".to_string(),
            is_main: false,
            is_locked: false,
            is_prunable: false,
        },
        Worktree {
            path: PathBuf::from("/home/user/project-bugfix"),
            branch: "bugfix/auth-issue".to_string(),
            head: "i7j8k9l".to_string(),
            is_main: false,
            is_locked: true,
            is_prunable: false,
        },
        Worktree {
            path: PathBuf::from("/home/user/project-old"),
            branch: "release/1.0".to_string(),
            head: "m0n1o2p".to_string(),
            is_main: false,
            is_locked: false,
            is_prunable: true,
        },
    ]
}
