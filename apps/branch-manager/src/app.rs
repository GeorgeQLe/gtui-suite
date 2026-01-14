use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub last_commit: String,
    pub last_commit_date: DateTime<Utc>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Compare,
}

pub struct App {
    pub branches: Vec<Branch>,
    pub selected: usize,
    pub view: View,
    pub show_remote: bool,
    pub compare_branch: Option<usize>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            selected: 0,
            view: View::List,
            show_remote: false,
            compare_branch: None,
            status_message: None,
        }
    }

    pub fn load_branches(&mut self) {
        self.branches = create_demo_branches();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Compare => self.handle_compare_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let visible = self.visible_branches();
                if self.selected < visible.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.checkout_branch();
            }
            KeyCode::Char('c') => {
                self.compare_branch = Some(self.selected);
                self.view = View::Compare;
            }
            KeyCode::Char('r') => {
                self.show_remote = !self.show_remote;
                self.selected = 0;
                self.status_message = Some(format!(
                    "Remote branches: {}",
                    if self.show_remote { "shown" } else { "hidden" }
                ));
            }
            KeyCode::Char('d') => {
                self.delete_branch();
            }
            KeyCode::Char('m') => {
                self.merge_branch();
            }
            _ => {}
        }
        false
    }

    fn handle_compare_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::List;
                self.compare_branch = None;
            }
            _ => {}
        }
        false
    }

    fn checkout_branch(&mut self) {
        if let Some(branch) = self.visible_branches().get(self.selected) {
            self.status_message = Some(format!("Checked out: {}", branch.name));
        }
    }

    fn delete_branch(&mut self) {
        if let Some(branch) = self.visible_branches().get(self.selected) {
            if branch.is_current {
                self.status_message = Some("Cannot delete current branch".to_string());
            } else {
                self.status_message = Some(format!("Would delete: {}", branch.name));
            }
        }
    }

    fn merge_branch(&mut self) {
        if let Some(branch) = self.visible_branches().get(self.selected) {
            self.status_message = Some(format!("Would merge: {}", branch.name));
        }
    }

    pub fn visible_branches(&self) -> Vec<&Branch> {
        if self.show_remote {
            self.branches.iter().collect()
        } else {
            self.branches.iter().filter(|b| !b.is_remote).collect()
        }
    }

    pub fn current_branch(&self) -> Option<&Branch> {
        self.branches.iter().find(|b| b.is_current)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List => "Enter:checkout c:compare r:remote d:delete m:merge".to_string(),
            View::Compare => "Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_branches() -> Vec<Branch> {
    vec![
        Branch {
            name: "main".to_string(),
            is_current: true,
            is_remote: false,
            last_commit: "abc1234".to_string(),
            last_commit_date: Utc::now(),
            ahead: 0,
            behind: 0,
        },
        Branch {
            name: "develop".to_string(),
            is_current: false,
            is_remote: false,
            last_commit: "def5678".to_string(),
            last_commit_date: Utc::now(),
            ahead: 5,
            behind: 2,
        },
        Branch {
            name: "feature/new-ui".to_string(),
            is_current: false,
            is_remote: false,
            last_commit: "ghi9abc".to_string(),
            last_commit_date: Utc::now(),
            ahead: 12,
            behind: 0,
        },
        Branch {
            name: "feature/api-refactor".to_string(),
            is_current: false,
            is_remote: false,
            last_commit: "jkl0def".to_string(),
            last_commit_date: Utc::now(),
            ahead: 3,
            behind: 8,
        },
        Branch {
            name: "origin/main".to_string(),
            is_current: false,
            is_remote: true,
            last_commit: "abc1234".to_string(),
            last_commit_date: Utc::now(),
            ahead: 0,
            behind: 0,
        },
        Branch {
            name: "origin/develop".to_string(),
            is_current: false,
            is_remote: true,
            last_commit: "mno3456".to_string(),
            last_commit_date: Utc::now(),
            ahead: 0,
            behind: 0,
        },
    ]
}
