use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkAction {
    Include,
    Exclude,
    Split,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub file: String,
    pub start_line: usize,
    pub old_lines: usize,
    pub new_lines: usize,
    pub content: Vec<DiffLine>,
    pub action: HunkAction,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: LineType,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Context,
    Addition,
    Deletion,
}

pub struct App {
    pub hunks: Vec<DiffHunk>,
    pub selected_hunk: usize,
    pub selected_line: usize,
    pub show_preview: bool,
    pub patch_name: String,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            hunks: create_demo_hunks(),
            selected_hunk: 0,
            selected_line: 0,
            show_preview: false,
            patch_name: "changes.patch".to_string(),
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
                if self.selected_hunk < self.hunks.len().saturating_sub(1) {
                    self.selected_hunk += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_hunk = self.selected_hunk.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                if let Some(hunk) = self.hunks.get_mut(self.selected_hunk) {
                    hunk.action = match hunk.action {
                        HunkAction::Include => HunkAction::Exclude,
                        HunkAction::Exclude => HunkAction::Include,
                        HunkAction::Split => HunkAction::Include,
                    };
                }
            }
            KeyCode::Char('a') => {
                for hunk in &mut self.hunks {
                    hunk.action = HunkAction::Include;
                }
                self.status_message = Some("All hunks included".to_string());
            }
            KeyCode::Char('n') => {
                for hunk in &mut self.hunks {
                    hunk.action = HunkAction::Exclude;
                }
                self.status_message = Some("All hunks excluded".to_string());
            }
            KeyCode::Char('s') => {
                if let Some(hunk) = self.hunks.get_mut(self.selected_hunk) {
                    hunk.action = HunkAction::Split;
                    self.status_message = Some("Would split hunk...".to_string());
                }
            }
            KeyCode::Char('p') => {
                self.show_preview = !self.show_preview;
            }
            KeyCode::Char('g') => {
                let included = self.hunks.iter().filter(|h| h.action == HunkAction::Include).count();
                self.status_message = Some(format!("Generating patch with {} hunks...", included));
            }
            KeyCode::Char('w') => {
                self.status_message = Some(format!("Would save to: {}", self.patch_name));
            }
            _ => {}
        }
        false
    }

    pub fn included_count(&self) -> usize {
        self.hunks.iter().filter(|h| h.action == HunkAction::Include).count()
    }

    pub fn total_additions(&self) -> usize {
        self.hunks.iter()
            .filter(|h| h.action == HunkAction::Include)
            .flat_map(|h| &h.content)
            .filter(|l| l.line_type == LineType::Addition)
            .count()
    }

    pub fn total_deletions(&self) -> usize {
        self.hunks.iter()
            .filter(|h| h.action == HunkAction::Include)
            .flat_map(|h| &h.content)
            .filter(|l| l.line_type == LineType::Deletion)
            .count()
    }

    pub fn current_hunk(&self) -> Option<&DiffHunk> {
        self.hunks.get(self.selected_hunk)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav space:toggle a:all n:none s:split p:preview g:generate w:write q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_hunks() -> Vec<DiffHunk> {
    vec![
        DiffHunk {
            file: "src/main.rs".to_string(),
            start_line: 10,
            old_lines: 3,
            new_lines: 5,
            content: vec![
                DiffLine { line_type: LineType::Context, content: "fn main() {".to_string() },
                DiffLine { line_type: LineType::Deletion, content: "    println!(\"Hello\");".to_string() },
                DiffLine { line_type: LineType::Addition, content: "    println!(\"Hello, World!\");".to_string() },
                DiffLine { line_type: LineType::Addition, content: "    println!(\"Welcome!\");".to_string() },
                DiffLine { line_type: LineType::Context, content: "}".to_string() },
            ],
            action: HunkAction::Include,
        },
        DiffHunk {
            file: "src/lib.rs".to_string(),
            start_line: 25,
            old_lines: 4,
            new_lines: 6,
            content: vec![
                DiffLine { line_type: LineType::Context, content: "pub fn process() {".to_string() },
                DiffLine { line_type: LineType::Deletion, content: "    let x = 1;".to_string() },
                DiffLine { line_type: LineType::Deletion, content: "    let y = 2;".to_string() },
                DiffLine { line_type: LineType::Addition, content: "    let x = 10;".to_string() },
                DiffLine { line_type: LineType::Addition, content: "    let y = 20;".to_string() },
                DiffLine { line_type: LineType::Addition, content: "    let z = 30;".to_string() },
                DiffLine { line_type: LineType::Context, content: "}".to_string() },
            ],
            action: HunkAction::Include,
        },
        DiffHunk {
            file: "Cargo.toml".to_string(),
            start_line: 5,
            old_lines: 1,
            new_lines: 2,
            content: vec![
                DiffLine { line_type: LineType::Deletion, content: "version = \"0.1.0\"".to_string() },
                DiffLine { line_type: LineType::Addition, content: "version = \"0.2.0\"".to_string() },
                DiffLine { line_type: LineType::Addition, content: "edition = \"2021\"".to_string() },
            ],
            action: HunkAction::Exclude,
        },
    ]
}
