use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json::Value;

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Tree,
    Raw,
    Stats,
    Query,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Query,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,
    pub root: Option<JsonNode>,
    pub flat_nodes: Vec<FlatNode>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub jq_query: String,
    pub query_result: Option<String>,
    pub file_path: Option<String>,
    pub clipboard: Option<Clipboard>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Tree,
            input_mode: InputMode::Normal,
            root: None,
            flat_nodes: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            search_query: String::new(),
            jq_query: String::new(),
            query_result: None,
            file_path: None,
            clipboard: Clipboard::new().ok(),
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        // Load demo JSON
        self.load_demo_json();
    }

    fn load_demo_json(&mut self) {
        let demo = serde_json::json!({
            "name": "TUI Suite",
            "version": "0.1.0",
            "description": "A collection of TUI applications",
            "authors": ["developer"],
            "repository": {
                "type": "git",
                "url": "https://github.com/example/tui-suite"
            },
            "apps": [
                {
                    "name": "json-explorer",
                    "tier": 4,
                    "features": ["tree-view", "search", "jq-queries"]
                },
                {
                    "name": "clipboard-manager",
                    "tier": 3,
                    "features": ["history", "categories", "search"]
                },
                {
                    "name": "snippet-manager",
                    "tier": 3,
                    "features": ["syntax-highlighting", "tags", "export"]
                }
            ],
            "config": {
                "theme": "catppuccin",
                "keybinds": "vim",
                "features": {
                    "auto_save": true,
                    "syntax_highlighting": true,
                    "line_numbers": true
                }
            },
            "stats": {
                "total_apps": 49,
                "implemented": 45,
                "lines_of_code": 50000
            }
        });

        self.load_json(demo);
    }

    pub fn load_json(&mut self, value: Value) {
        let root = JsonNode::from_value(value);
        self.flat_nodes.clear();
        flatten_tree(&root, &mut self.flat_nodes);
        self.root = Some(root);
        self.selected = 0;
    }

    pub fn load_from_string(&mut self, json_str: &str) -> Result<(), String> {
        match serde_json::from_str::<Value>(json_str) {
            Ok(value) => {
                self.load_json(value);
                Ok(())
            }
            Err(e) => Err(format!("Invalid JSON: {}", e)),
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') if self.input_mode == InputMode::Normal => return true,
            _ => {}
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Search => self.handle_search_key(key),
            InputMode::Query => self.handle_query_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.navigate_down(),
            KeyCode::Char('k') | KeyCode::Up => self.navigate_up(),
            KeyCode::Char('h') | KeyCode::Left => self.collapse_node(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => self.expand_node(),
            KeyCode::Char(' ') => self.toggle_node(),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => {
                self.selected = self.flat_nodes.len().saturating_sub(1);
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char(':') => {
                self.input_mode = InputMode::Query;
                self.view = View::Query;
                self.jq_query.clear();
            }
            KeyCode::Char('y') => self.copy_value(),
            KeyCode::Char('Y') => self.copy_path(),
            KeyCode::Char('e') => self.expand_all(),
            KeyCode::Char('c') => self.collapse_all(),
            KeyCode::Char('r') => self.view = View::Raw,
            KeyCode::Char('t') => self.view = View::Tree,
            KeyCode::Char('s') => self.view = View::Stats,
            KeyCode::Esc => {
                self.view = View::Tree;
                self.query_result = None;
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.search_next();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_query_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.view = View::Tree;
                self.jq_query.clear();
            }
            KeyCode::Enter => {
                self.execute_query();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.jq_query.pop();
            }
            KeyCode::Char(c) => {
                self.jq_query.push(c);
            }
            _ => {}
        }
        false
    }

    fn navigate_down(&mut self) {
        if self.selected < self.flat_nodes.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn navigate_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn expand_node(&mut self) {
        if let Some(flat_node) = self.flat_nodes.get_mut(self.selected) {
            if flat_node.node.is_expandable() && !flat_node.node.expanded {
                flat_node.node.expanded = true;
                self.rebuild_tree();
            }
        }
    }

    fn collapse_node(&mut self) {
        if let Some(flat_node) = self.flat_nodes.get_mut(self.selected) {
            if flat_node.node.expanded {
                flat_node.node.expanded = false;
                self.rebuild_tree();
            }
        }
    }

    fn toggle_node(&mut self) {
        if let Some(flat_node) = self.flat_nodes.get_mut(self.selected) {
            if flat_node.node.is_expandable() {
                flat_node.node.expanded = !flat_node.node.expanded;
                self.rebuild_tree();
            }
        }
    }

    fn expand_all(&mut self) {
        self.set_all_expanded(true);
        self.status_message = Some("Expanded all nodes".to_string());
    }

    fn collapse_all(&mut self) {
        self.set_all_expanded(false);
        self.status_message = Some("Collapsed all nodes".to_string());
    }

    fn set_all_expanded(&mut self, expanded: bool) {
        for flat_node in &mut self.flat_nodes {
            if flat_node.node.is_expandable() {
                flat_node.node.expanded = expanded;
            }
        }
        self.rebuild_tree();
    }

    fn rebuild_tree(&mut self) {
        // Store expanded state
        let expanded_paths: std::collections::HashSet<String> = self
            .flat_nodes
            .iter()
            .filter(|n| n.node.expanded)
            .map(|n| n.node.path.clone())
            .collect();

        self.flat_nodes.clear();

        // Take root out, rebuild, put back
        if let Some(mut root) = self.root.take() {
            Self::rebuild_node_recursive(&mut root, &expanded_paths, &mut self.flat_nodes);
            self.root = Some(root);
        }
    }

    fn rebuild_node_recursive(
        node: &mut JsonNode,
        expanded_paths: &std::collections::HashSet<String>,
        flat_nodes: &mut Vec<FlatNode>,
    ) {
        node.expanded = expanded_paths.contains(&node.path);

        flat_nodes.push(FlatNode {
            node: node.clone(),
            visible: true,
        });

        if node.expanded {
            for mut child in node.children() {
                Self::rebuild_node_recursive(&mut child, expanded_paths, flat_nodes);
            }
        }
    }

    fn search_next(&mut self) {
        if self.search_query.is_empty() {
            return;
        }

        let query = self.search_query.to_lowercase();
        let start = (self.selected + 1) % self.flat_nodes.len();

        for i in 0..self.flat_nodes.len() {
            let idx = (start + i) % self.flat_nodes.len();
            let node = &self.flat_nodes[idx].node;

            let searchable = format!(
                "{} {}",
                node.display_key(),
                node.display_value()
            ).to_lowercase();

            if searchable.contains(&query) {
                self.selected = idx;
                self.status_message = Some(format!("Found at {}", node.path));
                return;
            }
        }

        self.status_message = Some("No match found".to_string());
    }

    fn execute_query(&mut self) {
        if self.jq_query.is_empty() {
            return;
        }

        let Some(ref root) = self.root else {
            return;
        };

        // Simple path query implementation
        let result = self.query_path(&root.value, &self.jq_query);

        self.query_result = Some(match result {
            Some(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| "Error".to_string()),
            None => "No result".to_string(),
        });
    }

    fn query_path(&self, value: &Value, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path
            .trim_start_matches('.')
            .split('.')
            .filter(|s| !s.is_empty())
            .collect();

        let mut current = value;

        for part in parts {
            // Handle array index
            if let Some(idx_str) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    current = current.get(idx)?;
                    continue;
                }
            }

            // Handle array index in key
            if part.contains('[') {
                let (key, rest) = part.split_once('[')?;
                let idx_str = rest.strip_suffix(']')?;
                let idx: usize = idx_str.parse().ok()?;

                current = current.get(key)?.get(idx)?;
                continue;
            }

            current = current.get(part)?;
        }

        Some(current.clone())
    }

    fn copy_value(&mut self) {
        if let Some(flat_node) = self.flat_nodes.get(self.selected) {
            let value_str = serde_json::to_string_pretty(&flat_node.node.value)
                .unwrap_or_else(|_| flat_node.node.display_value());

            if let Some(ref mut clipboard) = self.clipboard {
                if clipboard.set_text(&value_str).is_ok() {
                    self.status_message = Some("Value copied!".to_string());
                }
            }
        }
    }

    fn copy_path(&mut self) {
        if let Some(flat_node) = self.flat_nodes.get(self.selected) {
            if let Some(ref mut clipboard) = self.clipboard {
                if clipboard.set_text(&flat_node.node.path).is_ok() {
                    self.status_message = Some(format!("Path copied: {}", flat_node.node.path));
                }
            }
        }
    }

    pub fn selected_node(&self) -> Option<&FlatNode> {
        self.flat_nodes.get(self.selected)
    }

    pub fn stats(&self) -> Option<JsonStats> {
        self.root.as_ref().map(|r| JsonStats::compute(&r.value))
    }

    pub fn raw_json(&self) -> String {
        self.root
            .as_ref()
            .map(|r| serde_json::to_string_pretty(&r.value).unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Tree => {
                if let Some(node) = self.selected_node() {
                    format!(
                        "{} | y:copy Y:path e:expand c:collapse /:search",
                        node.node.path
                    )
                } else {
                    "No JSON loaded".to_string()
                }
            }
            View::Raw => "Raw JSON view | t:tree s:stats".to_string(),
            View::Stats => "Statistics | t:tree r:raw".to_string(),
            View::Query => format!("Query: {} | Enter:execute Esc:cancel", self.jq_query),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
