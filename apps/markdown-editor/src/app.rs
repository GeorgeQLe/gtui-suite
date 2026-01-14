use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Edit,
    Preview,
    Split,
}

pub struct App {
    pub content: String,
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub mode: Mode,
    pub view: View,
    pub modified: bool,
    pub file_path: Option<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            mode: Mode::Normal,
            view: View::Split,
            modified: false,
            file_path: None,
            status_message: None,
        }
    }

    pub fn load_demo_content(&mut self) {
        self.content = DEMO_MARKDOWN.to_string();
        self.lines = self.content.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Insert => self.handle_insert_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                self.status_message = Some("-- INSERT --".to_string());
            }
            KeyCode::Char('a') => {
                self.mode = Mode::Insert;
                self.cursor_col = self.cursor_col.saturating_add(1).min(self.current_line_len());
                self.status_message = Some("-- INSERT --".to_string());
            }
            KeyCode::Char('o') => {
                self.mode = Mode::Insert;
                self.lines.insert(self.cursor_line + 1, String::new());
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.modified = true;
                self.status_message = Some("-- INSERT --".to_string());
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.cursor_line < self.lines.len().saturating_sub(1) {
                    self.cursor_line += 1;
                    self.cursor_col = self.cursor_col.min(self.current_line_len());
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.cursor_line = self.cursor_line.saturating_sub(1);
                self.cursor_col = self.cursor_col.min(self.current_line_len());
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.cursor_col = (self.cursor_col + 1).min(self.current_line_len());
            }
            KeyCode::Char('0') => {
                self.cursor_col = 0;
            }
            KeyCode::Char('$') => {
                self.cursor_col = self.current_line_len();
            }
            KeyCode::Char('g') => {
                self.cursor_line = 0;
                self.cursor_col = 0;
            }
            KeyCode::Char('G') => {
                self.cursor_line = self.lines.len().saturating_sub(1);
                self.cursor_col = 0;
            }
            KeyCode::Char('d') => {
                if self.lines.len() > 1 {
                    self.lines.remove(self.cursor_line);
                    if self.cursor_line >= self.lines.len() {
                        self.cursor_line = self.lines.len().saturating_sub(1);
                    }
                    self.modified = true;
                }
            }
            KeyCode::Tab => {
                self.view = match self.view {
                    View::Edit => View::Preview,
                    View::Preview => View::Split,
                    View::Split => View::Edit,
                };
                self.status_message = Some(format!("View: {:?}", self.view));
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save();
            }
            _ => {}
        }
        false
    }

    fn handle_insert_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.cursor_col = self.cursor_col.saturating_sub(1);
                self.status_message = None;
            }
            KeyCode::Enter => {
                let current_line = self.lines.get(self.cursor_line).cloned().unwrap_or_default();
                let (before, after) = current_line.split_at(self.cursor_col.min(current_line.len()));
                self.lines[self.cursor_line] = before.to_string();
                self.lines.insert(self.cursor_line + 1, after.to_string());
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.modified = true;
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    if let Some(line) = self.lines.get_mut(self.cursor_line) {
                        let col = self.cursor_col.min(line.len());
                        if col > 0 {
                            line.remove(col - 1);
                            self.cursor_col -= 1;
                            self.modified = true;
                        }
                    }
                } else if self.cursor_line > 0 {
                    let current_line = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines[self.cursor_line].len();
                    self.lines[self.cursor_line].push_str(&current_line);
                    self.modified = true;
                }
            }
            KeyCode::Char(c) => {
                if let Some(line) = self.lines.get_mut(self.cursor_line) {
                    let col = self.cursor_col.min(line.len());
                    line.insert(col, c);
                    self.cursor_col += 1;
                    self.modified = true;
                }
            }
            KeyCode::Left => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor_col = (self.cursor_col + 1).min(self.current_line_len());
            }
            KeyCode::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.cursor_col.min(self.current_line_len());
                }
            }
            KeyCode::Down => {
                if self.cursor_line < self.lines.len().saturating_sub(1) {
                    self.cursor_line += 1;
                    self.cursor_col = self.cursor_col.min(self.current_line_len());
                }
            }
            _ => {}
        }
        false
    }

    fn current_line_len(&self) -> usize {
        self.lines.get(self.cursor_line).map(|l| l.len()).unwrap_or(0)
    }

    fn save(&mut self) {
        self.modified = false;
        self.status_message = Some("Saved!".to_string());
    }

    pub fn content_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        let modified_indicator = if self.modified { "[+] " } else { "" };
        format!(
            "{}{}:{} | Tab:view i:insert Ctrl+S:save",
            modified_indicator,
            self.cursor_line + 1,
            self.cursor_col + 1
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

const DEMO_MARKDOWN: &str = r#"# Markdown Editor

Welcome to the **TUI Markdown Editor**!

## Features

- Real-time preview
- Vim-like keybindings
- Syntax highlighting

## Usage

1. Press `i` to enter insert mode
2. Press `Esc` to return to normal mode
3. Press `Tab` to cycle views

### Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

### Links

- [Rust](https://www.rust-lang.org/)
- [Ratatui](https://ratatui.rs/)

> This is a blockquote.
> It can span multiple lines.

---

*Italic text* and **bold text** are supported.
"#;
