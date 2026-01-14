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
    Create,
    Edit,
    SelectLanguage,
    Tags,
    Stats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    EditTitle,
    EditCode,
    EditTags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Title,
    Description,
    Code,
    Tags,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,
    pub snippets: Vec<Snippet>,
    pub selected: usize,
    pub search_query: String,
    pub language_filter: Option<Language>,
    pub tag_filter: Option<String>,
    pub show_favorites_only: bool,

    // Edit state
    pub edit_field: EditField,
    pub edit_buffer: String,
    pub edit_snippet: Option<Snippet>,
    pub selected_language: usize,

    // Tags view
    pub all_tags: Vec<String>,
    pub selected_tag: usize,

    pub clipboard: Option<Clipboard>,
    pub status_message: Option<String>,
    pub matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::List,
            input_mode: InputMode::Normal,
            snippets: Vec::new(),
            selected: 0,
            search_query: String::new(),
            language_filter: None,
            tag_filter: None,
            show_favorites_only: false,
            edit_field: EditField::Title,
            edit_buffer: String::new(),
            edit_snippet: None,
            selected_language: 0,
            all_tags: Vec::new(),
            selected_tag: 0,
            clipboard: Clipboard::new().ok(),
            status_message: None,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub async fn refresh(&mut self) {
        // Load demo snippets
        self.snippets = create_demo_snippets();
        self.update_tags();
    }

    fn update_tags(&mut self) {
        use std::collections::HashSet;
        let tags: HashSet<String> = self
            .snippets
            .iter()
            .flat_map(|s| s.tags.iter().cloned())
            .collect();
        self.all_tags = tags.into_iter().collect();
        self.all_tags.sort();
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') if self.input_mode == InputMode::Normal && self.view == View::List => {
                return true
            }
            _ => {}
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Search => self.handle_search_key(key),
            InputMode::EditTitle | InputMode::EditCode | InputMode::EditTags => {
                self.handle_edit_key(key)
            }
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match self.view {
            View::List | View::Preview => self.handle_list_key(key),
            View::Create | View::Edit => self.handle_create_key(key),
            View::SelectLanguage => self.handle_language_key(key),
            View::Tags => self.handle_tags_key(key),
            View::Stats => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                    self.view = View::List;
                }
                false
            }
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.navigate_down(),
            KeyCode::Char('k') | KeyCode::Up => self.navigate_up(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.view = if self.view == View::Preview {
                    View::List
                } else {
                    View::Preview
                };
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('n') => {
                self.start_create();
            }
            KeyCode::Char('e') => {
                self.start_edit();
            }
            KeyCode::Char('y') => self.copy_to_clipboard(),
            KeyCode::Char('f') => self.toggle_favorite(),
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('l') => {
                self.view = View::SelectLanguage;
                self.selected_language = 0;
            }
            KeyCode::Char('t') => {
                self.view = View::Tags;
                self.selected_tag = 0;
            }
            KeyCode::Char('F') => {
                self.show_favorites_only = !self.show_favorites_only;
                self.selected = 0;
            }
            KeyCode::Char('s') => {
                self.view = View::Stats;
            }
            KeyCode::Esc => {
                self.language_filter = None;
                self.tag_filter = None;
                self.show_favorites_only = false;
                self.search_query.clear();
            }
            _ => {}
        }
        false
    }

    fn handle_create_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
                self.edit_snippet = None;
            }
            KeyCode::Tab => {
                self.edit_field = match self.edit_field {
                    EditField::Title => EditField::Description,
                    EditField::Description => EditField::Code,
                    EditField::Code => EditField::Tags,
                    EditField::Tags => EditField::Title,
                };
            }
            KeyCode::Enter => {
                self.input_mode = match self.edit_field {
                    EditField::Title => InputMode::EditTitle,
                    EditField::Description => InputMode::EditTitle,
                    EditField::Code => InputMode::EditCode,
                    EditField::Tags => InputMode::EditTags,
                };
                if let Some(ref snippet) = self.edit_snippet {
                    self.edit_buffer = match self.edit_field {
                        EditField::Title => snippet.title.clone(),
                        EditField::Description => snippet.description.clone(),
                        EditField::Code => snippet.code.clone(),
                        EditField::Tags => snippet.tags.join(", "),
                    };
                }
            }
            KeyCode::Char('l') => {
                self.view = View::SelectLanguage;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_snippet();
            }
            _ => {}
        }
        false
    }

    fn handle_language_key(&mut self, key: KeyEvent) -> bool {
        let languages = Language::all();
        match key.code {
            KeyCode::Esc => {
                self.view = if self.edit_snippet.is_some() {
                    View::Create
                } else {
                    View::List
                };
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_language < languages.len().saturating_sub(1) {
                    self.selected_language += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_language = self.selected_language.saturating_sub(1);
            }
            KeyCode::Enter => {
                let lang = languages[self.selected_language];
                if let Some(ref mut snippet) = self.edit_snippet {
                    snippet.language = lang;
                    self.view = View::Create;
                } else {
                    self.language_filter = Some(lang);
                    self.selected = 0;
                    self.view = View::List;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_tags_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_tag < self.all_tags.len().saturating_sub(1) {
                    self.selected_tag += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_tag = self.selected_tag.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(tag) = self.all_tags.get(self.selected_tag) {
                    self.tag_filter = Some(tag.clone());
                    self.selected = 0;
                }
                self.view = View::List;
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

    fn handle_edit_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                // Apply changes
                if let Some(ref mut snippet) = self.edit_snippet {
                    match self.edit_field {
                        EditField::Title => snippet.title = self.edit_buffer.clone(),
                        EditField::Description => snippet.description = self.edit_buffer.clone(),
                        EditField::Code => snippet.code = self.edit_buffer.clone(),
                        EditField::Tags => {
                            snippet.tags = self
                                .edit_buffer
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                    }
                }
                self.edit_buffer.clear();
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
            }
            KeyCode::Enter if self.input_mode != InputMode::EditCode => {
                self.input_mode = InputMode::Normal;
                if let Some(ref mut snippet) = self.edit_snippet {
                    match self.edit_field {
                        EditField::Title => snippet.title = self.edit_buffer.clone(),
                        EditField::Description => snippet.description = self.edit_buffer.clone(),
                        EditField::Tags => {
                            snippet.tags = self
                                .edit_buffer
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                        _ => {}
                    }
                }
                self.edit_buffer.clear();
            }
            KeyCode::Enter if self.input_mode == InputMode::EditCode => {
                self.edit_buffer.push('\n');
            }
            KeyCode::Char(c) => {
                self.edit_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn navigate_down(&mut self) {
        let filtered = self.filtered_snippets();
        if self.selected < filtered.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    fn navigate_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn start_create(&mut self) {
        self.edit_snippet = Some(Snippet::new("New Snippet", "", Language::Other));
        self.edit_field = EditField::Title;
        self.view = View::Create;
    }

    fn start_edit(&mut self) {
        if let Some(snippet) = self.selected_snippet() {
            self.edit_snippet = Some(snippet.clone());
            self.edit_field = EditField::Title;
            self.view = View::Edit;
        }
    }

    fn save_snippet(&mut self) {
        if let Some(snippet) = self.edit_snippet.take() {
            if self.view == View::Edit {
                // Update existing
                if let Some(pos) = self.snippets.iter().position(|s| s.id == snippet.id) {
                    self.snippets[pos] = snippet;
                }
            } else {
                // Add new
                self.snippets.insert(0, snippet);
            }
            self.update_tags();
            self.status_message = Some("Snippet saved!".to_string());
        }
        self.view = View::List;
    }

    fn copy_to_clipboard(&mut self) {
        // Extract data before mutable borrow
        let snippet_data = self.selected_snippet().map(|s| (s.id, s.code.clone()));

        if let Some((id, code)) = snippet_data {
            let success = if let Some(ref mut clipboard) = self.clipboard {
                clipboard.set_text(&code).is_ok()
            } else {
                false
            };

            if success {
                // Update use count
                if let Some(s) = self.snippets.iter_mut().find(|s| s.id == id) {
                    s.use_count += 1;
                }
                self.status_message = Some("Copied to clipboard!".to_string());
            }
        }
    }

    fn toggle_favorite(&mut self) {
        if let Some(snippet) = self.selected_snippet() {
            let id = snippet.id;
            if let Some(s) = self.snippets.iter_mut().find(|s| s.id == id) {
                s.favorite = !s.favorite;
                self.status_message = Some(if s.favorite {
                    "Added to favorites".to_string()
                } else {
                    "Removed from favorites".to_string()
                });
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(snippet) = self.selected_snippet() {
            let id = snippet.id;
            self.snippets.retain(|s| s.id != id);
            if self.selected >= self.filtered_snippets().len() {
                self.selected = self.filtered_snippets().len().saturating_sub(1);
            }
            self.update_tags();
            self.status_message = Some("Snippet deleted".to_string());
        }
    }

    pub fn filtered_snippets(&self) -> Vec<&Snippet> {
        self.snippets
            .iter()
            .filter(|s| {
                if self.show_favorites_only && !s.favorite {
                    return false;
                }
                if let Some(lang) = self.language_filter {
                    if s.language != lang {
                        return false;
                    }
                }
                if let Some(ref tag) = self.tag_filter {
                    if !s.tags.contains(tag) {
                        return false;
                    }
                }
                if !self.search_query.is_empty() {
                    let searchable = format!("{} {} {}", s.title, s.description, s.code);
                    if self.matcher.fuzzy_match(&searchable, &self.search_query).is_none() {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn selected_snippet(&self) -> Option<&Snippet> {
        self.filtered_snippets().get(self.selected).copied()
    }

    pub fn stats(&self) -> SnippetStats {
        SnippetStats::compute(&self.snippets)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List | View::Preview => {
                let filtered = self.filtered_snippets();
                format!(
                    "{} snippets | n:new e:edit y:copy f:fav l:lang t:tags /:search",
                    filtered.len()
                )
            }
            View::Create | View::Edit => {
                "Tab:next field Enter:edit Ctrl+s:save Esc:cancel".to_string()
            }
            View::SelectLanguage => "j/k:navigate Enter:select Esc:cancel".to_string(),
            View::Tags => "j/k:navigate Enter:filter Esc:cancel".to_string(),
            View::Stats => "Press Esc or q to go back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_snippets() -> Vec<Snippet> {
    vec![
        {
            let mut s = Snippet::new(
                "Rust Error Handling",
                r#"fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_file("config.toml")?;
    println!("{}", content);
    Ok(())
}"#,
                Language::Rust,
            );
            s.description = "Basic error handling pattern with Result".to_string();
            s.tags = vec!["error-handling".to_string(), "result".to_string()];
            s.favorite = true;
            s
        },
        {
            let mut s = Snippet::new(
                "Python List Comprehension",
                r#"# Basic list comprehension
squares = [x**2 for x in range(10)]

# With condition
evens = [x for x in range(20) if x % 2 == 0]

# Nested comprehension
matrix = [[i*j for j in range(5)] for i in range(5)]"#,
                Language::Python,
            );
            s.description = "Common list comprehension patterns".to_string();
            s.tags = vec!["python".to_string(), "lists".to_string()];
            s
        },
        {
            let mut s = Snippet::new(
                "Git Rebase Commands",
                r#"# Interactive rebase last 5 commits
git rebase -i HEAD~5

# Rebase onto main
git rebase main

# Continue after resolving conflicts
git rebase --continue

# Abort rebase
git rebase --abort"#,
                Language::Shell,
            );
            s.description = "Common git rebase commands".to_string();
            s.tags = vec!["git".to_string(), "rebase".to_string()];
            s
        },
        {
            let mut s = Snippet::new(
                "Docker Compose Template",
                r#"version: "3.8"
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=development
    volumes:
      - .:/app
    depends_on:
      - db

  db:
    image: postgres:15
    environment:
      - POSTGRES_PASSWORD=secret
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:"#,
                Language::Yaml,
            );
            s.description = "Basic docker-compose template with app and database".to_string();
            s.tags = vec!["docker".to_string(), "compose".to_string()];
            s.favorite = true;
            s
        },
    ]
}
