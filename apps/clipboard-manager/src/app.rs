use arboard::Clipboard;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Preview,
    Search,
    Categories,
    Stats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Category,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,
    pub entries: Vec<ClipboardEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub category_filter: Option<String>,
    pub show_pinned_only: bool,
    pub show_favorites_only: bool,
    pub categories: Vec<Category>,
    pub selected_category: usize,
    pub clipboard: Option<Clipboard>,
    pub last_clipboard_content: String,
    pub status_message: Option<String>,
    pub matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Self {
        let clipboard = Clipboard::new().ok();

        Self {
            view: View::List,
            input_mode: InputMode::Normal,
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            search_query: String::new(),
            category_filter: None,
            show_pinned_only: false,
            show_favorites_only: false,
            categories: vec![
                Category::new("Work", CategoryColor::Blue),
                Category::new("Personal", CategoryColor::Green),
                Category::new("Code", CategoryColor::Yellow),
                Category::new("URLs", CategoryColor::Cyan),
            ],
            selected_category: 0,
            clipboard,
            last_clipboard_content: String::new(),
            status_message: None,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub async fn refresh(&mut self) {
        // Check for new clipboard content
        self.check_clipboard();
    }

    pub fn check_clipboard(&mut self) {
        if let Some(ref mut clipboard) = self.clipboard {
            if let Ok(content) = clipboard.get_text() {
                if !content.is_empty() && content != self.last_clipboard_content {
                    self.last_clipboard_content = content.clone();

                    // Don't add duplicates
                    if !self.entries.iter().any(|e| e.content == content) {
                        let entry = ClipboardEntry::new(content);
                        self.entries.insert(0, entry);

                        // Limit history size
                        if self.entries.len() > 1000 {
                            // Remove oldest non-pinned entries
                            while self.entries.len() > 1000 {
                                if let Some(pos) = self.entries.iter().rposition(|e| !e.pinned) {
                                    self.entries.remove(pos);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
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
            InputMode::Category => self.handle_category_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.navigate_down(),
            KeyCode::Char('k') | KeyCode::Up => self.navigate_up(),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => {
                let filtered = self.filtered_entries();
                self.selected = filtered.len().saturating_sub(1);
            }
            KeyCode::Enter => self.paste_selected(),
            KeyCode::Char('y') => self.copy_selected(),
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('p') => self.toggle_pin(),
            KeyCode::Char('f') => self.toggle_favorite(),
            KeyCode::Char('c') => {
                self.view = View::Categories;
                self.input_mode = InputMode::Category;
            }
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('D') => self.clear_history(),
            KeyCode::Char('P') => {
                self.show_pinned_only = !self.show_pinned_only;
                self.selected = 0;
            }
            KeyCode::Char('F') => {
                self.show_favorites_only = !self.show_favorites_only;
                self.selected = 0;
            }
            KeyCode::Char('s') => self.view = View::Stats,
            KeyCode::Char('v') | KeyCode::Char(' ') => {
                self.view = if self.view == View::Preview {
                    View::List
                } else {
                    View::Preview
                };
            }
            KeyCode::Esc => {
                self.view = View::List;
                self.category_filter = None;
                self.show_pinned_only = false;
                self.show_favorites_only = false;
            }
            KeyCode::Tab => self.cycle_content_type_filter(),
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
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.selected = 0;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.selected = 0;
            }
            _ => {}
        }
        false
    }

    fn handle_category_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.view = View::List;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_category < self.categories.len().saturating_sub(1) {
                    self.selected_category += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_category = self.selected_category.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(cat) = self.categories.get(self.selected_category) {
                    self.assign_category(&cat.name.clone());
                }
                self.input_mode = InputMode::Normal;
                self.view = View::List;
            }
            _ => {}
        }
        false
    }

    fn navigate_down(&mut self) {
        let filtered = self.filtered_entries();
        if self.selected < filtered.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn navigate_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn paste_selected(&mut self) {
        // Extract data needed before mutable borrow
        let entry_data = {
            let filtered = self.filtered_entries();
            filtered.get(self.selected).map(|e| (e.id, e.content.clone()))
        };

        if let Some((id, content)) = entry_data {
            let success = if let Some(ref mut clipboard) = self.clipboard {
                clipboard.set_text(&content).is_ok()
            } else {
                false
            };

            if success {
                // Update use count
                if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
                    e.use_count += 1;
                    e.last_used = Utc::now();
                }
                self.last_clipboard_content = content;
                self.status_message = Some("Copied to clipboard!".to_string());
            }
        }
    }

    fn copy_selected(&mut self) {
        self.paste_selected();
    }

    fn toggle_pin(&mut self) {
        let filtered = self.filtered_entries();
        if let Some(entry) = filtered.get(self.selected) {
            let id = entry.id;
            if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
                e.pinned = !e.pinned;
                self.status_message = Some(if e.pinned {
                    "Entry pinned".to_string()
                } else {
                    "Entry unpinned".to_string()
                });
            }
        }
    }

    fn toggle_favorite(&mut self) {
        let filtered = self.filtered_entries();
        if let Some(entry) = filtered.get(self.selected) {
            let id = entry.id;
            if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
                e.favorite = !e.favorite;
                self.status_message = Some(if e.favorite {
                    "Added to favorites".to_string()
                } else {
                    "Removed from favorites".to_string()
                });
            }
        }
    }

    fn assign_category(&mut self, category: &str) {
        let filtered = self.filtered_entries();
        if let Some(entry) = filtered.get(self.selected) {
            let id = entry.id;
            if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
                e.category = Some(category.to_string());
                self.status_message = Some(format!("Assigned to '{}'", category));
            }
        }
    }

    fn delete_selected(&mut self) {
        let filtered = self.filtered_entries();
        if let Some(entry) = filtered.get(self.selected) {
            let id = entry.id;
            self.entries.retain(|e| e.id != id);
            if self.selected >= self.filtered_entries().len() {
                self.selected = self.filtered_entries().len().saturating_sub(1);
            }
            self.status_message = Some("Entry deleted".to_string());
        }
    }

    fn clear_history(&mut self) {
        // Keep pinned entries
        self.entries.retain(|e| e.pinned);
        self.selected = 0;
        self.status_message = Some("History cleared (pinned items kept)".to_string());
    }

    fn cycle_content_type_filter(&mut self) {
        // Toggle through content type filters
        self.status_message = Some("Content type filter cycling".to_string());
    }

    pub fn filtered_entries(&self) -> Vec<&ClipboardEntry> {
        self.entries
            .iter()
            .filter(|e| {
                // Pinned filter
                if self.show_pinned_only && !e.pinned {
                    return false;
                }

                // Favorites filter
                if self.show_favorites_only && !e.favorite {
                    return false;
                }

                // Category filter
                if let Some(ref cat) = self.category_filter {
                    if e.category.as_ref() != Some(cat) {
                        return false;
                    }
                }

                // Search filter
                if !self.search_query.is_empty() {
                    if self.matcher.fuzzy_match(&e.content, &self.search_query).is_none() {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    pub fn selected_entry(&self) -> Option<&ClipboardEntry> {
        self.filtered_entries().get(self.selected).copied()
    }

    pub fn stats(&self) -> ClipboardStats {
        ClipboardStats::compute(&self.entries)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        let filtered = self.filtered_entries();
        let filters = [
            self.show_pinned_only.then_some("📌"),
            self.show_favorites_only.then_some("⭐"),
            (!self.search_query.is_empty()).then_some("🔍"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");

        format!(
            "{} entries{} | y:copy p:pin f:fav d:del /:search c:category",
            filtered.len(),
            if filters.is_empty() {
                String::new()
            } else {
                format!(" [{}]", filters)
            }
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
