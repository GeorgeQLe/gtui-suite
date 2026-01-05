//! Application state and logic.

use crate::config::Config;
use crate::formats::{parse_json, parse_toml, validate_format, ConfigFormat, ConfigNode, ConfigValue};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

pub struct App {
    pub config: Config,
    pub file_path: Option<PathBuf>,
    pub format: ConfigFormat,
    pub content: Vec<String>,
    pub tree: Option<ConfigNode>,
    pub tree_items: Vec<TreeItem>,
    pub selected_tree_index: usize,
    pub cursor: (usize, usize), // (line, col)
    pub modified: bool,
    pub mode: Mode,
    pub pane: Pane,
    pub validation_error: Option<String>,
    pub message: Option<String>,
    pub show_help: bool,
    pub show_quit_confirm: bool,
    pub input_buffer: String,
    pub input_mode: InputMode,
}

#[derive(Debug, Clone)]
pub struct TreeItem {
    pub key: String,
    pub value_display: String,
    pub depth: usize,
    pub expanded: bool,
    pub is_container: bool,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Tree,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    OpenFile,
    SaveAs,
}

impl App {
    pub fn new(file_path: Option<String>) -> anyhow::Result<Self> {
        let config = Config::load();

        let (content, format, path) = if let Some(ref path_str) = file_path {
            let path = PathBuf::from(path_str);
            let content = std::fs::read_to_string(&path)?;
            let format = ConfigFormat::from_path(&path);
            (content.lines().map(|s| s.to_string()).collect(), format, Some(path))
        } else {
            (vec![String::new()], ConfigFormat::Unknown, None)
        };

        let mut app = Self {
            config,
            file_path: path,
            format,
            content,
            tree: None,
            tree_items: Vec::new(),
            selected_tree_index: 0,
            cursor: (0, 0),
            modified: false,
            mode: Mode::Normal,
            pane: Pane::Tree,
            validation_error: None,
            message: None,
            show_help: false,
            show_quit_confirm: false,
            input_buffer: String::new(),
            input_mode: InputMode::None,
        };

        app.parse_content();
        Ok(app)
    }

    pub fn can_quit(&self) -> bool {
        self.mode == Mode::Normal && self.input_mode == InputMode::None
    }

    fn parse_content(&mut self) {
        let content_str = self.content.join("\n");

        // Validate
        if self.config.editor.auto_validate {
            self.validation_error = validate_format(&content_str, self.format).err();
        }

        // Parse tree
        self.tree = match self.format {
            ConfigFormat::Toml => parse_toml(&content_str).ok(),
            ConfigFormat::Json => parse_json(&content_str).ok(),
            ConfigFormat::Unknown => None,
        };

        self.rebuild_tree_items();
    }

    fn rebuild_tree_items(&mut self) {
        self.tree_items.clear();
        if let Some(tree) = self.tree.clone() {
            self.add_tree_items(&tree);
        }
    }

    fn add_tree_items(&mut self, node: &ConfigNode) {
        if node.key != "root" {
            self.tree_items.push(TreeItem {
                key: node.key.clone(),
                value_display: node.value.display_value(),
                depth: node.depth,
                expanded: node.expanded,
                is_container: node.value.is_container(),
                path: node.path.clone(),
            });
        }

        if node.expanded || node.key == "root" {
            match &node.value {
                ConfigValue::Table(children) | ConfigValue::Array(children) => {
                    for child in children {
                        self.add_tree_items(child);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.message = None;

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.input_mode != InputMode::None {
            self.handle_input_key(key);
            return;
        }

        if self.mode == Mode::Editing {
            self.handle_editor_key(key);
            return;
        }

        match key.code {
            // Pane switching
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::Tree => Pane::Editor,
                    Pane::Editor => Pane::Tree,
                };
            }

            // Tree navigation
            KeyCode::Char('j') | KeyCode::Down => {
                if self.pane == Pane::Tree {
                    self.move_tree_selection(1);
                } else {
                    self.move_cursor(1, 0);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.pane == Pane::Tree {
                    self.move_tree_selection(-1);
                } else {
                    self.move_cursor(-1, 0);
                }
            }
            KeyCode::Enter => {
                if self.pane == Pane::Tree {
                    self.toggle_tree_item();
                }
            }

            // Actions
            KeyCode::Char('e') => {
                self.mode = Mode::Editing;
                self.pane = Pane::Editor;
            }
            KeyCode::Char('o') => {
                self.input_mode = InputMode::OpenFile;
                self.input_buffer.clear();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_file();
            }
            KeyCode::Char('S') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_mode = InputMode::SaveAs;
                self.input_buffer = self.file_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
            }
            KeyCode::Char('r') => {
                self.parse_content();
                self.message = Some("Refreshed".to_string());
            }

            // Help
            KeyCode::Char('?') => self.show_help = true,

            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::None;
                self.input_buffer.clear();
            }
            KeyCode::Enter => self.finish_input(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.pane = Pane::Tree;
                self.parse_content();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_file();
            }
            KeyCode::Up => self.move_cursor(-1, 0),
            KeyCode::Down => self.move_cursor(1, 0),
            KeyCode::Left => self.move_cursor(0, -1),
            KeyCode::Right => self.move_cursor(0, 1),
            KeyCode::Enter => {
                if let Some(line) = self.content.get_mut(self.cursor.0) {
                    let rest = line.split_off(self.cursor.1);
                    self.content.insert(self.cursor.0 + 1, rest);
                    self.cursor.0 += 1;
                    self.cursor.1 = 0;
                    self.modified = true;
                }
            }
            KeyCode::Backspace => {
                if self.cursor.1 > 0 {
                    if let Some(line) = self.content.get_mut(self.cursor.0) {
                        line.remove(self.cursor.1 - 1);
                        self.cursor.1 -= 1;
                        self.modified = true;
                    }
                } else if self.cursor.0 > 0 {
                    let current = self.content.remove(self.cursor.0);
                    self.cursor.0 -= 1;
                    if let Some(prev) = self.content.get_mut(self.cursor.0) {
                        self.cursor.1 = prev.len();
                        prev.push_str(&current);
                    }
                    self.modified = true;
                }
            }
            KeyCode::Char(c) => {
                if self.content.is_empty() {
                    self.content.push(String::new());
                }
                if let Some(line) = self.content.get_mut(self.cursor.0) {
                    line.insert(self.cursor.1, c);
                    self.cursor.1 += 1;
                    self.modified = true;
                }
            }
            _ => {}
        }
    }

    fn finish_input(&mut self) {
        match self.input_mode {
            InputMode::OpenFile => {
                let path = PathBuf::from(&self.input_buffer);
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.content = content.lines().map(|s| s.to_string()).collect();
                        self.format = ConfigFormat::from_path(&path);
                        self.file_path = Some(path.clone());
                        self.modified = false;
                        self.cursor = (0, 0);
                        self.parse_content();
                        self.config.add_recent(path);
                        let _ = self.config.save();
                        self.message = Some("File opened".to_string());
                    }
                } else {
                    self.message = Some("File not found".to_string());
                }
            }
            InputMode::SaveAs => {
                let path = PathBuf::from(&self.input_buffer);
                self.file_path = Some(path.clone());
                self.format = ConfigFormat::from_path(&path);
                self.save_file();
            }
            InputMode::None => {}
        }

        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }

    fn move_tree_selection(&mut self, delta: i32) {
        let len = self.tree_items.len();
        if len == 0 { return; }
        let new_idx = self.selected_tree_index as i32 + delta;
        self.selected_tree_index = new_idx.clamp(0, len as i32 - 1) as usize;
    }

    fn toggle_tree_item(&mut self) {
        if let Some(item) = self.tree_items.get(self.selected_tree_index) {
            if item.is_container {
                let path = item.path.clone();
                if let Some(ref mut tree) = self.tree {
                    Self::toggle_node(tree, &path);
                }
                self.rebuild_tree_items();
            }
        }
    }

    fn toggle_node(node: &mut ConfigNode, path: &[String]) {
        if node.path == path {
            node.expanded = !node.expanded;
            return;
        }

        match &mut node.value {
            ConfigValue::Table(children) | ConfigValue::Array(children) => {
                for child in children {
                    Self::toggle_node(child, path);
                }
            }
            _ => {}
        }
    }

    fn move_cursor(&mut self, line_delta: i32, col_delta: i32) {
        let new_line = (self.cursor.0 as i32 + line_delta).clamp(0, self.content.len() as i32 - 1) as usize;
        self.cursor.0 = new_line;

        if let Some(line) = self.content.get(self.cursor.0) {
            let new_col = (self.cursor.1 as i32 + col_delta).clamp(0, line.len() as i32) as usize;
            self.cursor.1 = new_col;
        }
    }

    fn save_file(&mut self) {
        if let Some(ref path) = self.file_path {
            let content = self.content.join("\n");
            if std::fs::write(path, &content).is_ok() {
                self.modified = false;
                self.parse_content();
                self.message = Some("Saved".to_string());
            } else {
                self.message = Some("Failed to save".to_string());
            }
        } else {
            self.input_mode = InputMode::SaveAs;
            self.input_buffer.clear();
        }
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "[New File]".to_string())
    }
}
