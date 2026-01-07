use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use similar::ChangeTag;
use std::fs;
use std::path::PathBuf;

use crate::diff::{DiffLine, FileDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Unified,
    SideBySide,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
}

pub struct App {
    pub view: View,
    pub mode: Mode,
    pub left_path: Option<PathBuf>,
    pub right_path: Option<PathBuf>,
    pub left_content: String,
    pub right_content: String,
    pub diff: Option<FileDiff>,
    pub left_lines: Vec<DiffLine>,
    pub right_lines: Vec<DiffLine>,
    pub scroll: usize,
    pub hunk_index: usize,
    pub ignore_whitespace: bool,
    pub context_lines: usize,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::SideBySide,
            mode: Mode::Normal,
            left_path: None,
            right_path: None,
            left_content: String::new(),
            right_content: String::new(),
            diff: None,
            left_lines: Vec::new(),
            right_lines: Vec::new(),
            scroll: 0,
            hunk_index: 0,
            ignore_whitespace: false,
            context_lines: 3,
            message: None,
            error: None,
        }
    }

    pub fn load_files(&mut self, left: PathBuf, right: PathBuf) -> Result<()> {
        self.left_content = fs::read_to_string(&left)?;
        self.right_content = fs::read_to_string(&right)?;

        let left_str = left.display().to_string();
        let right_str = right.display().to_string();

        self.diff = Some(FileDiff::compute(
            &left_str,
            &self.left_content,
            &right_str,
            &self.right_content,
        ));

        self.left_path = Some(left);
        self.right_path = Some(right);

        self.build_side_by_side();
        Ok(())
    }

    fn build_side_by_side(&mut self) {
        self.left_lines.clear();
        self.right_lines.clear();

        let left_lines: Vec<&str> = self.left_content.lines().collect();
        let right_lines: Vec<&str> = self.right_content.lines().collect();

        if let Some(ref diff) = self.diff {
            let mut left_idx = 0usize;
            let mut right_idx = 0usize;

            for hunk in &diff.hunks {
                // Add context before hunk
                while left_idx < hunk.old_start.saturating_sub(1) && right_idx < hunk.new_start.saturating_sub(1) {
                    let left_content = left_lines.get(left_idx).unwrap_or(&"").to_string();
                    let right_content = right_lines.get(right_idx).unwrap_or(&"").to_string();

                    self.left_lines.push(DiffLine {
                        tag: ChangeTag::Equal,
                        old_line: Some(left_idx + 1),
                        new_line: None,
                        content: left_content,
                    });
                    self.right_lines.push(DiffLine {
                        tag: ChangeTag::Equal,
                        old_line: None,
                        new_line: Some(right_idx + 1),
                        content: right_content,
                    });

                    left_idx += 1;
                    right_idx += 1;
                }

                // Add hunk lines
                for line in &hunk.lines {
                    match line.tag {
                        ChangeTag::Delete => {
                            self.left_lines.push(line.clone());
                            self.right_lines.push(DiffLine {
                                tag: ChangeTag::Equal,
                                old_line: None,
                                new_line: None,
                                content: String::new(),
                            });
                            left_idx += 1;
                        }
                        ChangeTag::Insert => {
                            self.left_lines.push(DiffLine {
                                tag: ChangeTag::Equal,
                                old_line: None,
                                new_line: None,
                                content: String::new(),
                            });
                            self.right_lines.push(line.clone());
                            right_idx += 1;
                        }
                        ChangeTag::Equal => {
                            self.left_lines.push(line.clone());
                            self.right_lines.push(line.clone());
                            left_idx += 1;
                            right_idx += 1;
                        }
                    }
                }
            }

            // Add remaining lines
            while left_idx < left_lines.len() || right_idx < right_lines.len() {
                let left_content = left_lines.get(left_idx).map(|s| s.to_string()).unwrap_or_default();
                let right_content = right_lines.get(right_idx).map(|s| s.to_string()).unwrap_or_default();

                self.left_lines.push(DiffLine {
                    tag: ChangeTag::Equal,
                    old_line: if left_idx < left_lines.len() { Some(left_idx + 1) } else { None },
                    new_line: None,
                    content: left_content,
                });
                self.right_lines.push(DiffLine {
                    tag: ChangeTag::Equal,
                    old_line: None,
                    new_line: if right_idx < right_lines.len() { Some(right_idx + 1) } else { None },
                    content: right_content,
                });

                if left_idx < left_lines.len() { left_idx += 1; }
                if right_idx < right_lines.len() { right_idx += 1; }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Esc => {
                if self.view == View::Help {
                    self.view = View::SideBySide;
                }
            }

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.left_lines.len().max(self.right_lines.len());
                if self.scroll < max.saturating_sub(1) {
                    self.scroll += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(20);
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(20);
            }
            KeyCode::Home | KeyCode::Char('g') => self.scroll = 0,
            KeyCode::End | KeyCode::Char('G') => {
                let max = self.left_lines.len().max(self.right_lines.len());
                self.scroll = max.saturating_sub(1);
            }

            // Hunk navigation
            KeyCode::Char('n') => self.next_hunk(),
            KeyCode::Char('p') => self.prev_hunk(),

            // View toggle
            KeyCode::Tab => {
                self.view = match self.view {
                    View::SideBySide => View::Unified,
                    View::Unified => View::SideBySide,
                    View::Help => View::SideBySide,
                };
            }

            // Toggle whitespace
            KeyCode::Char('w') => {
                self.ignore_whitespace = !self.ignore_whitespace;
                self.message = Some(format!("Ignore whitespace: {}", self.ignore_whitespace));
            }

            _ => {}
        }

        false
    }

    fn next_hunk(&mut self) {
        if let Some(ref diff) = self.diff {
            if self.hunk_index < diff.hunks.len().saturating_sub(1) {
                self.hunk_index += 1;
                if let Some(hunk) = diff.hunks.get(self.hunk_index) {
                    self.scroll = hunk.old_start.saturating_sub(self.context_lines);
                }
            }
        }
    }

    fn prev_hunk(&mut self) {
        if let Some(ref diff) = self.diff {
            if self.hunk_index > 0 {
                self.hunk_index -= 1;
                if let Some(hunk) = diff.hunks.get(self.hunk_index) {
                    self.scroll = hunk.old_start.saturating_sub(self.context_lines);
                }
            }
        }
    }

    pub fn stats(&self) -> (usize, usize) {
        self.diff.as_ref()
            .map(|d| (d.total_additions(), d.total_deletions()))
            .unwrap_or((0, 0))
    }
}
