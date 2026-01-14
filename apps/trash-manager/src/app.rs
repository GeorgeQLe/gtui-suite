use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TrashItem {
    pub name: String,
    pub original_path: PathBuf,
    pub size: u64,
    pub deleted_at: DateTime<Utc>,
    pub is_dir: bool,
}

impl TrashItem {
    pub fn size_formatted(&self) -> String {
        format_size(self.size)
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub struct App {
    pub items: Vec<TrashItem>,
    pub selected: usize,
    pub total_size: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            total_size: 0,
            status_message: None,
        }
    }

    pub fn load_trash(&mut self) {
        self.items = create_demo_trash();
        self.total_size = self.items.iter().map(|i| i.size).sum();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.items.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('r') => {
                self.restore_selected();
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                self.delete_selected();
            }
            KeyCode::Char('e') => {
                self.empty_trash();
            }
            KeyCode::Char('R') => {
                self.restore_all();
            }
            _ => {}
        }
        false
    }

    fn restore_selected(&mut self) {
        if let Some(item) = self.items.get(self.selected) {
            self.status_message = Some(format!("Restored: {}", item.name));
        }
    }

    fn delete_selected(&mut self) {
        if let Some(item) = self.items.get(self.selected) {
            self.status_message = Some(format!("Permanently deleted: {}", item.name));
        }
    }

    fn empty_trash(&mut self) {
        self.status_message = Some(format!(
            "Would empty {} items ({} total)",
            self.items.len(),
            format_size(self.total_size)
        ));
    }

    fn restore_all(&mut self) {
        self.status_message = Some(format!("Would restore {} items", self.items.len()));
    }

    pub fn selected_item(&self) -> Option<&TrashItem> {
        self.items.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        format!(
            "{} items ({}) | r:restore d:delete e:empty R:restore-all",
            self.items.len(),
            format_size(self.total_size)
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_trash() -> Vec<TrashItem> {
    vec![
        TrashItem {
            name: "old_project".to_string(),
            original_path: PathBuf::from("/home/user/projects/old_project"),
            size: 150 * 1024 * 1024,
            deleted_at: Utc::now(),
            is_dir: true,
        },
        TrashItem {
            name: "backup.tar.gz".to_string(),
            original_path: PathBuf::from("/home/user/backup.tar.gz"),
            size: 500 * 1024 * 1024,
            deleted_at: Utc::now(),
            is_dir: false,
        },
        TrashItem {
            name: "notes.txt".to_string(),
            original_path: PathBuf::from("/home/user/Documents/notes.txt"),
            size: 2048,
            deleted_at: Utc::now(),
            is_dir: false,
        },
        TrashItem {
            name: "screenshots".to_string(),
            original_path: PathBuf::from("/home/user/Pictures/screenshots"),
            size: 25 * 1024 * 1024,
            deleted_at: Utc::now(),
            is_dir: true,
        },
        TrashItem {
            name: "temp_data.json".to_string(),
            original_path: PathBuf::from("/tmp/temp_data.json"),
            size: 1024 * 50,
            deleted_at: Utc::now(),
            is_dir: false,
        },
    ]
}
