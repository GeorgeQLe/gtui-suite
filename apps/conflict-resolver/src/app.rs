use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct ConflictHunk {
    pub ours: Vec<String>,
    pub theirs: Vec<String>,
    pub resolved: Option<Resolution>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Ours,
    Theirs,
    Both,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ConflictFile {
    pub path: String,
    pub hunks: Vec<ConflictHunk>,
    pub current_hunk: usize,
}

pub struct App {
    pub files: Vec<ConflictFile>,
    pub current_file: usize,
    pub view_mode: ViewMode,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    FileList,
    HunkView,
}

impl App {
    pub fn new() -> Self {
        Self {
            files: create_demo_conflicts(),
            current_file: 0,
            view_mode: ViewMode::FileList,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view_mode {
            ViewMode::FileList => self.handle_file_list_key(key),
            ViewMode::HunkView => self.handle_hunk_view_key(key),
        }
    }

    fn handle_file_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.current_file < self.files.len().saturating_sub(1) {
                    self.current_file += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.current_file = self.current_file.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !self.files.is_empty() {
                    self.view_mode = ViewMode::HunkView;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_hunk_view_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view_mode = ViewMode::FileList;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(file) = self.files.get_mut(self.current_file) {
                    if file.current_hunk < file.hunks.len().saturating_sub(1) {
                        file.current_hunk += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(file) = self.files.get_mut(self.current_file) {
                    file.current_hunk = file.current_hunk.saturating_sub(1);
                }
            }
            KeyCode::Char('o') | KeyCode::Char('1') => {
                self.resolve_current(Resolution::Ours);
            }
            KeyCode::Char('t') | KeyCode::Char('2') => {
                self.resolve_current(Resolution::Theirs);
            }
            KeyCode::Char('b') | KeyCode::Char('3') => {
                self.resolve_current(Resolution::Both);
            }
            KeyCode::Char('n') => {
                self.next_unresolved();
            }
            _ => {}
        }
        false
    }

    fn resolve_current(&mut self, resolution: Resolution) {
        if let Some(file) = self.files.get_mut(self.current_file) {
            if let Some(hunk) = file.hunks.get_mut(file.current_hunk) {
                hunk.resolved = Some(resolution);
                self.status_message = Some(format!("Resolved with {:?}", resolution));
            }
        }
    }

    fn next_unresolved(&mut self) {
        if let Some(file) = self.files.get_mut(self.current_file) {
            for i in (file.current_hunk + 1)..file.hunks.len() {
                if file.hunks[i].resolved.is_none() {
                    file.current_hunk = i;
                    return;
                }
            }
            // Wrap around
            for i in 0..file.current_hunk {
                if file.hunks[i].resolved.is_none() {
                    file.current_hunk = i;
                    return;
                }
            }
            self.status_message = Some("All hunks resolved!".to_string());
        }
    }

    pub fn current_file(&self) -> Option<&ConflictFile> {
        self.files.get(self.current_file)
    }

    pub fn resolved_count(&self) -> (usize, usize) {
        let total: usize = self.files.iter().map(|f| f.hunks.len()).sum();
        let resolved: usize = self.files.iter()
            .flat_map(|f| &f.hunks)
            .filter(|h| h.resolved.is_some())
            .count();
        (resolved, total)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        match self.view_mode {
            ViewMode::FileList => "j/k:navigate Enter:view-hunks q:quit".to_string(),
            ViewMode::HunkView => "o:ours t:theirs b:both n:next-unresolved Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_conflicts() -> Vec<ConflictFile> {
    vec![
        ConflictFile {
            path: "src/main.rs".to_string(),
            hunks: vec![
                ConflictHunk {
                    ours: vec![
                        "fn main() {".to_string(),
                        "    println!(\"Hello from main!\");".to_string(),
                        "}".to_string(),
                    ],
                    theirs: vec![
                        "fn main() {".to_string(),
                        "    println!(\"Hello, World!\");".to_string(),
                        "    run_app();".to_string(),
                        "}".to_string(),
                    ],
                    resolved: None,
                },
            ],
            current_hunk: 0,
        },
        ConflictFile {
            path: "src/lib.rs".to_string(),
            hunks: vec![
                ConflictHunk {
                    ours: vec![
                        "pub const VERSION: &str = \"1.0.0\";".to_string(),
                    ],
                    theirs: vec![
                        "pub const VERSION: &str = \"2.0.0\";".to_string(),
                    ],
                    resolved: None,
                },
                ConflictHunk {
                    ours: vec![
                        "pub fn add(a: i32, b: i32) -> i32 {".to_string(),
                        "    a + b".to_string(),
                        "}".to_string(),
                    ],
                    theirs: vec![
                        "pub fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {".to_string(),
                        "    a + b".to_string(),
                        "}".to_string(),
                    ],
                    resolved: None,
                },
            ],
            current_hunk: 0,
        },
        ConflictFile {
            path: "Cargo.toml".to_string(),
            hunks: vec![
                ConflictHunk {
                    ours: vec![
                        "[dependencies]".to_string(),
                        "serde = \"1.0\"".to_string(),
                    ],
                    theirs: vec![
                        "[dependencies]".to_string(),
                        "serde = { version = \"1.0\", features = [\"derive\"] }".to_string(),
                    ],
                    resolved: None,
                },
            ],
            current_hunk: 0,
        },
    ]
}
