use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Stash {
    pub index: usize,
    pub message: String,
    pub branch: String,
    pub created_at: DateTime<Utc>,
    pub files: Vec<StashFile>,
}

#[derive(Debug, Clone)]
pub struct StashFile {
    pub path: String,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

impl FileStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            FileStatus::Modified => "M",
            FileStatus::Added => "A",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Details,
    Diff,
}

pub struct App {
    pub stashes: Vec<Stash>,
    pub selected: usize,
    pub selected_file: usize,
    pub view: View,
    pub diff_content: String,
    pub status_message: Option<String>,
    pub scroll_offset: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            stashes: Vec::new(),
            selected: 0,
            selected_file: 0,
            view: View::List,
            diff_content: String::new(),
            status_message: None,
            scroll_offset: 0,
        }
    }

    pub async fn refresh(&mut self) {
        self.stashes = create_demo_stashes();
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Details => self.handle_details_key(key),
            View::Diff => self.handle_diff_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.stashes.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !self.stashes.is_empty() {
                    self.view = View::Details;
                    self.selected_file = 0;
                }
            }
            KeyCode::Char('a') => {
                self.apply_stash();
            }
            KeyCode::Char('p') => {
                self.pop_stash();
            }
            KeyCode::Char('d') => {
                self.drop_stash();
            }
            KeyCode::Char('b') => {
                self.branch_from_stash();
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Refreshing...".to_string());
            }
            _ => {}
        }
        false
    }

    fn handle_details_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::List;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(stash) = self.stashes.get(self.selected) {
                    if self.selected_file < stash.files.len().saturating_sub(1) {
                        self.selected_file += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_file = self.selected_file.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.show_diff();
            }
            KeyCode::Char('a') => {
                self.apply_stash();
            }
            _ => {}
        }
        false
    }

    fn handle_diff_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Details;
                self.scroll_offset = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_offset += 1;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('d') => {
                self.scroll_offset += 10;
            }
            KeyCode::Char('u') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            _ => {}
        }
        false
    }

    fn apply_stash(&mut self) {
        if let Some(stash) = self.stashes.get(self.selected) {
            self.status_message = Some(format!("Applied stash@{{{}}}", stash.index));
        }
    }

    fn pop_stash(&mut self) {
        if self.selected < self.stashes.len() {
            let stash = self.stashes.remove(self.selected);
            self.status_message = Some(format!("Popped stash@{{{}}}", stash.index));
            if self.selected >= self.stashes.len() {
                self.selected = self.stashes.len().saturating_sub(1);
            }
        }
    }

    fn drop_stash(&mut self) {
        if self.selected < self.stashes.len() {
            let stash = self.stashes.remove(self.selected);
            self.status_message = Some(format!("Dropped stash@{{{}}}", stash.index));
            if self.selected >= self.stashes.len() {
                self.selected = self.stashes.len().saturating_sub(1);
            }
        }
    }

    fn branch_from_stash(&mut self) {
        if let Some(stash) = self.stashes.get(self.selected) {
            self.status_message = Some(format!(
                "Created branch from stash@{{{}}}",
                stash.index
            ));
        }
    }

    fn show_diff(&mut self) {
        if let Some(stash) = self.stashes.get(self.selected) {
            if let Some(file) = stash.files.get(self.selected_file) {
                self.diff_content = generate_demo_diff(&file.path);
                self.view = View::Diff;
                self.scroll_offset = 0;
            }
        }
    }

    pub fn selected_stash(&self) -> Option<&Stash> {
        self.stashes.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List => format!(
                "{} stashes | a:apply p:pop d:drop b:branch Enter:details",
                self.stashes.len()
            ),
            View::Details => "j/k:navigate Enter:diff a:apply Esc:back".to_string(),
            View::Diff => "j/k:scroll d/u:page Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_stashes() -> Vec<Stash> {
    vec![
        Stash {
            index: 0,
            message: "WIP on main: feat: add user auth".to_string(),
            branch: "main".to_string(),
            created_at: Utc::now() - chrono::Duration::hours(2),
            files: vec![
                StashFile {
                    path: "src/auth/login.rs".to_string(),
                    status: FileStatus::Modified,
                    additions: 45,
                    deletions: 12,
                },
                StashFile {
                    path: "src/auth/mod.rs".to_string(),
                    status: FileStatus::Modified,
                    additions: 5,
                    deletions: 0,
                },
                StashFile {
                    path: "src/models/user.rs".to_string(),
                    status: FileStatus::Added,
                    additions: 120,
                    deletions: 0,
                },
            ],
        },
        Stash {
            index: 1,
            message: "WIP on feature/api: refactor endpoints".to_string(),
            branch: "feature/api".to_string(),
            created_at: Utc::now() - chrono::Duration::days(1),
            files: vec![
                StashFile {
                    path: "src/api/routes.rs".to_string(),
                    status: FileStatus::Modified,
                    additions: 89,
                    deletions: 34,
                },
                StashFile {
                    path: "src/api/handlers.rs".to_string(),
                    status: FileStatus::Modified,
                    additions: 156,
                    deletions: 78,
                },
            ],
        },
        Stash {
            index: 2,
            message: "On main: fix: resolve merge conflict".to_string(),
            branch: "main".to_string(),
            created_at: Utc::now() - chrono::Duration::days(3),
            files: vec![
                StashFile {
                    path: "Cargo.toml".to_string(),
                    status: FileStatus::Modified,
                    additions: 2,
                    deletions: 2,
                },
            ],
        },
        Stash {
            index: 3,
            message: "WIP on develop: experimental feature".to_string(),
            branch: "develop".to_string(),
            created_at: Utc::now() - chrono::Duration::days(7),
            files: vec![
                StashFile {
                    path: "src/experimental/feature.rs".to_string(),
                    status: FileStatus::Added,
                    additions: 234,
                    deletions: 0,
                },
                StashFile {
                    path: "src/lib.rs".to_string(),
                    status: FileStatus::Modified,
                    additions: 1,
                    deletions: 0,
                },
                StashFile {
                    path: "tests/feature_test.rs".to_string(),
                    status: FileStatus::Added,
                    additions: 67,
                    deletions: 0,
                },
                StashFile {
                    path: "README.md".to_string(),
                    status: FileStatus::Modified,
                    additions: 15,
                    deletions: 3,
                },
            ],
        },
    ]
}

fn generate_demo_diff(path: &str) -> String {
    format!(
        r#"diff --git a/{path} b/{path}
index 1234567..abcdefg 100644
--- a/{path}
+++ b/{path}
@@ -1,10 +1,15 @@
+// New import added
+use std::collections::HashMap;
+
 fn main() {{
-    println!("Hello, world!");
+    println!("Hello, updated world!");
+
+    let mut map = HashMap::new();
+    map.insert("key", "value");
 }}

-fn old_function() {{
-    // This was removed
-}}
+fn new_function() {{
+    // This was added
+    todo!()
+}}"#,
        path = path
    )
}
