use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct BlameLine {
    pub line_number: usize,
    pub content: String,
    pub commit_hash: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub commit_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Blame,
    Commit,
}

pub struct App {
    pub file_path: String,
    pub lines: Vec<BlameLine>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub view: View,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            file_path: "src/main.rs".to_string(),
            lines: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            view: View::Blame,
            status_message: None,
        }
    }

    pub fn load_demo_blame(&mut self) {
        self.lines = create_demo_blame();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::Blame => self.handle_blame_key(key),
            View::Commit => self.handle_commit_key(key),
        }
    }

    fn handle_blame_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.lines.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.view = View::Commit;
            }
            KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::Char('G') => {
                self.selected = self.lines.len().saturating_sub(1);
            }
            KeyCode::Char('y') => {
                if let Some(line) = self.lines.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", line.commit_hash));
                }
            }
            _ => {}
        }
        false
    }

    fn handle_commit_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Blame;
            }
            KeyCode::Char('y') => {
                if let Some(line) = self.lines.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", line.commit_hash));
                }
            }
            _ => {}
        }
        false
    }

    pub fn selected_line(&self) -> Option<&BlameLine> {
        self.lines.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Blame => format!(
                "Line {}/{} | Enter:commit y:copy g/G:top/bottom",
                self.selected + 1,
                self.lines.len()
            ),
            View::Commit => "y:copy hash Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_blame() -> Vec<BlameLine> {
    let authors = ["Alice", "Bob", "Charlie", "Diana"];
    let hashes = ["abc1234", "def5678", "ghi9abc", "jkl0def"];
    let messages = [
        "Initial commit",
        "Add main function",
        "Fix bug in handler",
        "Refactor imports",
    ];

    let code_lines = [
        "use std::io;",
        "",
        "fn main() {",
        "    println!(\"Hello, world!\");",
        "    let x = 42;",
        "    process(x);",
        "}",
        "",
        "fn process(n: i32) {",
        "    for i in 0..n {",
        "        println!(\"{}\", i);",
        "    }",
        "}",
        "",
        "// TODO: Add error handling",
    ];

    code_lines
        .iter()
        .enumerate()
        .map(|(i, content)| {
            let idx = i % 4;
            BlameLine {
                line_number: i + 1,
                content: content.to_string(),
                commit_hash: hashes[idx].to_string(),
                author: authors[idx].to_string(),
                date: Utc::now(),
                commit_message: messages[idx].to_string(),
            }
        })
        .collect()
}
