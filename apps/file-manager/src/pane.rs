use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::entry::{EntryType, FileEntry, SortMethod};

/// A single file browser pane
pub struct Pane {
    /// Current directory path
    pub path: PathBuf,
    /// All entries in current directory
    pub entries: Vec<FileEntry>,
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset for display
    pub scroll_offset: usize,
    /// Selected files (for bulk operations)
    pub selection: HashSet<PathBuf>,
    /// Show hidden files
    pub show_hidden: bool,
    /// Current sort method
    pub sort_method: SortMethod,
    /// Sort ascending
    pub sort_ascending: bool,
    /// Directories first
    pub dirs_first: bool,
}

impl Pane {
    pub fn new(path: Option<&str>) -> Self {
        let path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
            });

        let mut pane = Self {
            path: path.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            selection: HashSet::new(),
            show_hidden: false,
            sort_method: SortMethod::Name,
            sort_ascending: true,
            dirs_first: true,
        };

        pane.refresh();
        pane
    }

    /// Refresh directory contents
    pub fn refresh(&mut self) {
        self.entries.clear();

        // Add parent directory entry
        if self.path.parent().is_some() {
            if let Some(parent) = FileEntry::parent(&self.path) {
                self.entries.push(parent);
            }
        }

        // Read directory entries
        if let Ok(read_dir) = fs::read_dir(&self.path) {
            for entry in read_dir.flatten() {
                if let Some(file_entry) = FileEntry::from_path(&entry.path()) {
                    // Filter hidden files
                    if !self.show_hidden && file_entry.is_hidden {
                        continue;
                    }
                    self.entries.push(file_entry);
                }
            }
        }

        // Sort entries
        self.sort_entries();

        // Ensure selection is valid
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }

    fn sort_entries(&mut self) {
        // Separate parent entry
        let has_parent = self.entries.first().map(|e| e.name == "..").unwrap_or(false);

        let start = if has_parent { 1 } else { 0 };
        let entries_to_sort = &mut self.entries[start..];

        entries_to_sort.sort_by(|a, b| {
            // Directories first
            if self.dirs_first {
                match (a.entry_type, b.entry_type) {
                    (EntryType::Directory, EntryType::File) |
                    (EntryType::Directory, EntryType::Symlink) => return std::cmp::Ordering::Less,
                    (EntryType::File, EntryType::Directory) |
                    (EntryType::Symlink, EntryType::Directory) => return std::cmp::Ordering::Greater,
                    _ => {}
                }
            }

            let cmp = match self.sort_method {
                SortMethod::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortMethod::Size => a.size.cmp(&b.size),
                SortMethod::Modified => a.modified.cmp(&b.modified),
                SortMethod::Type => {
                    let ext_a = a.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let ext_b = b.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    ext_a.cmp(ext_b)
                }
            };

            if self.sort_ascending { cmp } else { cmp.reverse() }
        });
    }

    /// Navigate to a directory
    pub fn navigate(&mut self, path: &Path) {
        if path.is_dir() {
            self.path = path.to_path_buf();
            self.selected = 0;
            self.scroll_offset = 0;
            self.selection.clear();
            self.refresh();
        }
    }

    /// Navigate to parent directory
    pub fn go_parent(&mut self) {
        if let Some(parent) = self.path.parent() {
            let old_path = self.path.clone();
            self.path = parent.to_path_buf();
            self.selected = 0;
            self.scroll_offset = 0;
            self.selection.clear();
            self.refresh();

            // Try to select the directory we came from
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.path == old_path {
                    self.selected = i;
                    break;
                }
            }
        }
    }

    /// Enter selected directory or open file
    pub fn enter(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.current_entry() {
            match entry.entry_type {
                EntryType::Directory => {
                    let path = entry.path.clone();
                    self.navigate(&path);
                    None
                }
                EntryType::File | EntryType::Symlink => {
                    Some(entry.path.clone())
                }
            }
        } else {
            None
        }
    }

    /// Get currently selected entry
    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected < self.entries.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move to top
    pub fn move_to_top(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Move to bottom
    pub fn move_to_bottom(&mut self) {
        self.selected = self.entries.len().saturating_sub(1);
    }

    /// Toggle selection on current item
    pub fn toggle_selection(&mut self) {
        if let Some(entry) = self.current_entry() {
            if entry.name == ".." {
                return;
            }

            let path = entry.path.clone();
            if self.selection.contains(&path) {
                self.selection.remove(&path);
            } else {
                self.selection.insert(path);
            }
        }
        self.move_down();
    }

    /// Invert selection
    pub fn invert_selection(&mut self) {
        let all_paths: HashSet<_> = self.entries.iter()
            .filter(|e| e.name != "..")
            .map(|e| e.path.clone())
            .collect();

        let new_selection: HashSet<_> = all_paths
            .difference(&self.selection)
            .cloned()
            .collect();

        self.selection = new_selection;
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Toggle hidden files
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.refresh();
    }

    /// Cycle sort method
    pub fn cycle_sort(&mut self) {
        self.sort_method = self.sort_method.cycle();
        self.sort_entries();
    }

    /// Toggle sort direction
    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.sort_entries();
    }

    /// Get selected files for operations
    pub fn get_selected_files(&self) -> Vec<PathBuf> {
        if self.selection.is_empty() {
            // If nothing selected, use current entry
            self.current_entry()
                .filter(|e| e.name != "..")
                .map(|e| vec![e.path.clone()])
                .unwrap_or_default()
        } else {
            self.selection.iter().cloned().collect()
        }
    }

    /// Calculate total size of selected files
    pub fn selected_size(&self) -> u64 {
        if self.selection.is_empty() {
            self.current_entry()
                .map(|e| e.size)
                .unwrap_or(0)
        } else {
            self.selection.iter()
                .filter_map(|p| self.entries.iter().find(|e| &e.path == p))
                .map(|e| e.size)
                .sum()
        }
    }
}
