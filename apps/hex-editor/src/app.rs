use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::buffer::HexBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Hex,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Edit,
    Search(String),
    Goto(String),
}

pub struct App {
    pub buffer: HexBuffer,
    pub view: View,
    pub mode: Mode,
    pub cursor: usize,
    pub scroll: usize,
    pub bytes_per_row: usize,
    pub editing_nibble: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            buffer: HexBuffer::new(),
            view: View::Hex,
            mode: Mode::Normal,
            cursor: 0,
            scroll: 0,
            bytes_per_row: 16,
            editing_nibble: false,
            message: None,
            error: None,
        }
    }

    pub fn open(&mut self, path: PathBuf) -> Result<()> {
        self.buffer = HexBuffer::open(path)?;
        self.cursor = 0;
        self.scroll = 0;
        Ok(())
    }

    pub fn visible_rows(&self, height: usize) -> usize {
        height.saturating_sub(4)
    }

    pub fn ensure_visible(&mut self, height: usize) {
        let visible = self.visible_rows(height);
        let cursor_row = self.cursor / self.bytes_per_row;

        if cursor_row < self.scroll {
            self.scroll = cursor_row;
        } else if cursor_row >= self.scroll + visible {
            self.scroll = cursor_row - visible + 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, height: usize) -> bool {
        self.message = None;
        self.error = None;

        match &self.mode {
            Mode::Normal => self.handle_normal_key(key, height),
            Mode::Edit => self.handle_edit_key(key, height),
            Mode::Search(_) => self.handle_search_key(key),
            Mode::Goto(_) => self.handle_goto_key(key, height),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent, height: usize) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Esc => {
                if self.view == View::Help {
                    self.view = View::Hex;
                }
            }

            // Navigation
            KeyCode::Right | KeyCode::Char('l') => {
                if self.cursor < self.buffer.len().saturating_sub(1) {
                    self.cursor += 1;
                    self.ensure_visible(height);
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.ensure_visible(height);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + self.bytes_per_row < self.buffer.len() {
                    self.cursor += self.bytes_per_row;
                    self.ensure_visible(height);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor >= self.bytes_per_row {
                    self.cursor -= self.bytes_per_row;
                    self.ensure_visible(height);
                }
            }
            KeyCode::PageDown => {
                let jump = self.visible_rows(height) * self.bytes_per_row;
                self.cursor = (self.cursor + jump).min(self.buffer.len().saturating_sub(1));
                self.ensure_visible(height);
            }
            KeyCode::PageUp => {
                let jump = self.visible_rows(height) * self.bytes_per_row;
                self.cursor = self.cursor.saturating_sub(jump);
                self.ensure_visible(height);
            }
            KeyCode::Home => {
                self.cursor = 0;
                self.scroll = 0;
            }
            KeyCode::End => {
                self.cursor = self.buffer.len().saturating_sub(1);
                self.ensure_visible(height);
            }

            // Edit mode
            KeyCode::Char('i') | KeyCode::Enter => {
                if !self.buffer.is_empty() {
                    self.mode = Mode::Edit;
                    self.editing_nibble = false;
                }
            }

            // Search
            KeyCode::Char('/') => {
                self.mode = Mode::Search(String::new());
            }

            // Goto
            KeyCode::Char('g') => {
                self.mode = Mode::Goto(String::new());
            }

            // Save
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.buffer.save() {
                    Ok(_) => self.message = Some("Saved".to_string()),
                    Err(e) => self.error = Some(format!("Save failed: {}", e)),
                }
            }

            // Undo/Redo
            KeyCode::Char('u') => {
                if self.buffer.undo() {
                    self.message = Some("Undo".to_string());
                }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.buffer.redo() {
                    self.message = Some("Redo".to_string());
                }
            }

            // Find next
            KeyCode::Char('n') => {
                self.message = Some("Use / to search".to_string());
            }

            _ => {}
        }

        false
    }

    fn handle_edit_key(&mut self, key: KeyEvent, height: usize) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.editing_nibble = false;
            }
            KeyCode::Char(c) if c.is_ascii_hexdigit() => {
                let nibble = c.to_digit(16).unwrap() as u8;
                if let Some(current) = self.buffer.get(self.cursor) {
                    let new_value = if self.editing_nibble {
                        (current & 0xF0) | nibble
                    } else {
                        (nibble << 4) | (current & 0x0F)
                    };
                    self.buffer.set(self.cursor, new_value);

                    if self.editing_nibble {
                        // Move to next byte
                        if self.cursor < self.buffer.len() - 1 {
                            self.cursor += 1;
                            self.ensure_visible(height);
                        }
                        self.editing_nibble = false;
                    } else {
                        self.editing_nibble = true;
                    }
                }
            }
            KeyCode::Right | KeyCode::Tab => {
                if self.cursor < self.buffer.len() - 1 {
                    self.cursor += 1;
                    self.ensure_visible(height);
                }
                self.editing_nibble = false;
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.ensure_visible(height);
                }
                self.editing_nibble = false;
            }
            _ => {}
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        if let Mode::Search(ref mut query) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let q = query.clone();
                    if let Some(pos) = self.buffer.search_hex(&q, self.cursor + 1) {
                        self.cursor = pos;
                        self.message = Some(format!("Found at 0x{:08X}", pos));
                    } else if let Some(pos) = self.buffer.search_hex(&q, 0) {
                        self.cursor = pos;
                        self.message = Some(format!("Found at 0x{:08X} (wrapped)", pos));
                    } else {
                        self.error = Some("Pattern not found".to_string());
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    query.pop();
                }
                KeyCode::Char(c) if c.is_ascii_hexdigit() || c == ' ' => {
                    query.push(c);
                }
                _ => {}
            }
        }
        false
    }

    fn handle_goto_key(&mut self, key: KeyEvent, height: usize) -> bool {
        if let Mode::Goto(ref mut addr) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let a = addr.clone();
                    if let Ok(offset) = usize::from_str_radix(&a, 16) {
                        if offset < self.buffer.len() {
                            self.cursor = offset;
                            self.ensure_visible(height);
                            self.message = Some(format!("Jumped to 0x{:08X}", offset));
                        } else {
                            self.error = Some("Address out of range".to_string());
                        }
                    } else {
                        self.error = Some("Invalid hex address".to_string());
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    addr.pop();
                }
                KeyCode::Char(c) if c.is_ascii_hexdigit() => {
                    addr.push(c);
                }
                _ => {}
            }
        }
        false
    }
}
