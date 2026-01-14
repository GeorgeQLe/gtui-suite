use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub children: Vec<DirEntry>,
    pub expanded: bool,
    pub depth: usize,
}

impl DirEntry {
    pub fn size_formatted(&self) -> String {
        format_size(self.size)
    }

    pub fn percentage(&self, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (self.size as f64 / total as f64) * 100.0
        }
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Size,
    Name,
    Items,
}

pub struct App {
    pub root: Option<DirEntry>,
    pub flat_list: Vec<FlatEntry>,
    pub selected: usize,
    pub current_path: PathBuf,
    pub total_size: u64,
    pub sort_by: SortBy,
    pub show_hidden: bool,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FlatEntry {
    pub entry: DirEntry,
    pub depth: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            root: None,
            flat_list: Vec::new(),
            selected: 0,
            current_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            total_size: 0,
            sort_by: SortBy::Size,
            show_hidden: false,
            status_message: None,
        }
    }

    pub async fn scan_current_dir(&mut self) {
        self.status_message = Some("Scanning...".to_string());
        let path = self.current_path.clone();
        self.root = Some(self.scan_directory(&path, 0));
        if let Some(ref root) = self.root {
            self.total_size = root.size;
        }
        self.rebuild_flat_list();
        self.status_message = None;
    }

    fn scan_directory(&self, path: &PathBuf, depth: usize) -> DirEntry {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let mut entry = DirEntry {
            path: path.clone(),
            name,
            size: 0,
            is_dir: path.is_dir(),
            children: Vec::new(),
            expanded: depth == 0,
            depth,
        };

        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for e in entries.flatten() {
                    let child_path = e.path();
                    let child_name = child_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if !self.show_hidden && child_name.starts_with('.') {
                        continue;
                    }

                    if child_path.is_dir() && depth < 10 {
                        let child = self.scan_directory(&child_path, depth + 1);
                        entry.size += child.size;
                        entry.children.push(child);
                    } else if child_path.is_file() {
                        if let Ok(meta) = child_path.metadata() {
                            let size = meta.len();
                            entry.size += size;
                            entry.children.push(DirEntry {
                                path: child_path,
                                name: child_name,
                                size,
                                is_dir: false,
                                children: Vec::new(),
                                expanded: false,
                                depth: depth + 1,
                            });
                        }
                    }
                }
            }

            // Sort children
            match self.sort_by {
                SortBy::Size => entry.children.sort_by(|a, b| b.size.cmp(&a.size)),
                SortBy::Name => entry.children.sort_by(|a, b| a.name.cmp(&b.name)),
                SortBy::Items => entry
                    .children
                    .sort_by(|a, b| b.children.len().cmp(&a.children.len())),
            }
        } else if let Ok(meta) = path.metadata() {
            entry.size = meta.len();
        }

        entry
    }

    fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        if let Some(root) = self.root.take() {
            self.add_to_flat_list(&root, 0);
            self.root = Some(root);
        }
    }

    fn add_to_flat_list(&mut self, entry: &DirEntry, depth: usize) {
        self.flat_list.push(FlatEntry {
            entry: entry.clone(),
            depth,
        });

        if entry.expanded {
            for child in &entry.children {
                self.add_to_flat_list(child, depth + 1);
            }
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.flat_list.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                self.toggle_expand();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.collapse_or_parent();
            }
            KeyCode::Char('s') => {
                self.cycle_sort();
            }
            KeyCode::Char('.') => {
                self.show_hidden = !self.show_hidden;
                self.scan_current_dir().await;
            }
            KeyCode::Char('r') => {
                self.scan_current_dir().await;
            }
            KeyCode::Char('u') => {
                self.go_up();
            }
            KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::Char('G') => {
                self.selected = self.flat_list.len().saturating_sub(1);
            }
            _ => {}
        }
        false
    }

    fn toggle_expand(&mut self) {
        if let Some(flat_entry) = self.flat_list.get(self.selected) {
            if flat_entry.entry.is_dir {
                let path = flat_entry.entry.path.clone();
                self.toggle_expanded_at_path(&path);
                self.rebuild_flat_list();
            }
        }
    }

    fn toggle_expanded_at_path(&mut self, path: &PathBuf) {
        if let Some(ref mut root) = self.root {
            Self::toggle_in_tree(root, path);
        }
    }

    fn toggle_in_tree(entry: &mut DirEntry, path: &PathBuf) {
        if &entry.path == path {
            entry.expanded = !entry.expanded;
            return;
        }
        for child in &mut entry.children {
            Self::toggle_in_tree(child, path);
        }
    }

    fn collapse_or_parent(&mut self) {
        if let Some(flat_entry) = self.flat_list.get(self.selected) {
            if flat_entry.entry.expanded {
                let path = flat_entry.entry.path.clone();
                self.toggle_expanded_at_path(&path);
                self.rebuild_flat_list();
            } else if flat_entry.depth > 0 {
                // Find parent
                for (i, fe) in self.flat_list.iter().enumerate() {
                    if fe.depth == flat_entry.depth - 1 && i < self.selected {
                        self.selected = i;
                        break;
                    }
                }
            }
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Size => SortBy::Name,
            SortBy::Name => SortBy::Items,
            SortBy::Items => SortBy::Size,
        };
        self.status_message = Some(format!("Sorted by {:?}", self.sort_by));
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.selected = 0;
            // Trigger rescan on next tick
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        format!(
            "Total: {} | Sort: {:?} | Hidden: {} | j/k:nav Enter:expand s:sort .:hidden r:refresh",
            format_size(self.total_size),
            self.sort_by,
            if self.show_hidden { "on" } else { "off" }
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
