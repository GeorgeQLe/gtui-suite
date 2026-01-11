use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Board,
    CardDetail,
    Sources,
    Conflicts,
    Import,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    EditTitle,
    AddCard,
    SelectSource,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,

    // Board data
    pub board: Board,
    pub selected_column: usize,
    pub selected_card: usize,
    pub current_card: Option<Card>,

    // Sources
    pub selected_source: usize,

    // Conflicts
    pub conflicts: Vec<Conflict>,
    pub selected_conflict: usize,

    // Input
    pub input_buffer: String,
    pub search_query: String,

    // Status
    pub status_message: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            view: View::Board,
            input_mode: InputMode::Normal,
            board: Board::new("My Board"),
            selected_column: 0,
            selected_card: 0,
            current_card: None,
            selected_source: 0,
            conflicts: Vec::new(),
            selected_conflict: 0,
            input_buffer: String::new(),
            search_query: String::new(),
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        // Create demo board with columns
        self.board = Board::new("Project Board");
        self.board.add_column("Backlog");
        self.board.add_column("In Progress");
        self.board.add_column("Review");
        self.board.add_column("Done");

        // Add demo cards to Backlog
        {
            let mut card = Card::new("Implement user authentication");
            card.description = "Add login/logout functionality".to_string();
            card.labels = vec!["feature".to_string(), "backend".to_string()];
            card.priority = Priority::High;
            card.assignee = Some("alice".to_string());
            self.board.columns[0].cards.push(card);
        }
        {
            let mut card = Card::new("Design dashboard mockups");
            card.labels = vec!["design".to_string()];
            card.priority = Priority::Medium;
            self.board.columns[0].cards.push(card);
        }

        // Add cards to In Progress
        {
            let mut card = Card::new("API endpoint for users");
            card.labels = vec!["backend".to_string()];
            card.priority = Priority::High;
            card.assignee = Some("bob".to_string());
            card.link = Some(ExternalLink::new(
                uuid::Uuid::new_v4(),
                "123",
                "https://github.com/org/repo/issues/123",
            ));
            self.board.columns[1].cards.push(card);
        }

        // Add cards to Review
        {
            let mut card = Card::new("Fix login bug");
            card.labels = vec!["bug".to_string()];
            card.priority = Priority::Critical;
            card.checklist = vec![
                ChecklistItem::new("Reproduce issue"),
                ChecklistItem::new("Write fix"),
                ChecklistItem::new("Add tests"),
            ];
            card.checklist[0].done = true;
            card.checklist[1].done = true;
            self.board.columns[2].cards.push(card);
        }

        // Add cards to Done
        {
            let mut card = Card::new("Setup CI/CD pipeline");
            card.labels = vec!["devops".to_string()];
            card.priority = Priority::Medium;
            self.board.columns[3].cards.push(card);
        }

        // Add demo sources
        self.board.sources = vec![
            {
                let mut source = ExternalSource::new(SourceType::GitHub, "org/repo");
                source.last_sync = Some(Utc::now() - chrono::Duration::minutes(5));
                source.sync_status = SyncStatus::Synced;
                source
            },
            {
                let mut source = ExternalSource::new(SourceType::GitLab, "group/project");
                source.last_sync = Some(Utc::now() - chrono::Duration::hours(1));
                source.sync_status = SyncStatus::RemoteChanges;
                source
            },
        ];
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::EditTitle | InputMode::AddCard => self.handle_edit_key(key),
            InputMode::SelectSource => self.handle_source_key(key),
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') => {
                match self.view {
                    View::CardDetail => {
                        self.view = View::Board;
                        self.current_card = None;
                    }
                    View::Sources | View::Conflicts | View::Import => {
                        self.view = View::Board;
                    }
                    View::Board => return true,
                }
            }

            KeyCode::Char('h') | KeyCode::Left => {
                self.selected_column = self.selected_column.saturating_sub(1);
                self.selected_card = 0;
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.selected_column < self.board.columns.len().saturating_sub(1) {
                    self.selected_column += 1;
                    self.selected_card = 0;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(column) = self.board.columns.get(self.selected_column) {
                    if self.selected_card < column.cards.len().saturating_sub(1) {
                        self.selected_card += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_card = self.selected_card.saturating_sub(1);
            }

            KeyCode::Enter => {
                self.open_card_detail();
            }

            KeyCode::Char('n') | KeyCode::Char('a') => {
                self.input_mode = InputMode::AddCard;
                self.input_buffer.clear();
            }

            KeyCode::Char('d') => {
                self.delete_card();
            }

            KeyCode::Char('m') => {
                self.move_card_right();
            }
            KeyCode::Char('M') => {
                self.move_card_left();
            }

            KeyCode::Char('I') => {
                self.view = View::Import;
                self.input_mode = InputMode::SelectSource;
            }

            KeyCode::Char('X') => {
                self.view = View::Sources;
                self.selected_source = 0;
            }

            KeyCode::Char('S') => {
                self.sync_sources().await;
            }

            KeyCode::Char('C') => {
                self.view = View::Conflicts;
                self.selected_conflict = 0;
            }

            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }

            KeyCode::Char('L') => {
                self.link_card();
            }

            KeyCode::Char('U') => {
                self.unlink_card();
            }

            _ => {}
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
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

    fn handle_edit_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    if self.input_mode == InputMode::AddCard {
                        self.add_card(&self.input_buffer.clone());
                    }
                }
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_source_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.view = View::Board;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_source =
                    (self.selected_source + 1).min(self.board.sources.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_source = self.selected_source.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.do_import_from_source();
                self.input_mode = InputMode::Normal;
                self.view = View::Board;
            }
            _ => {}
        }
        false
    }

    fn open_card_detail(&mut self) {
        if let Some(column) = self.board.columns.get(self.selected_column) {
            if let Some(card) = column.cards.get(self.selected_card) {
                self.current_card = Some(card.clone());
                self.view = View::CardDetail;
            }
        }
    }

    fn add_card(&mut self, title: &str) {
        if let Some(column) = self.board.columns.get_mut(self.selected_column) {
            let card = Card::new(title);
            column.cards.push(card);
            self.status_message = Some(format!("Added card: {}", title));
        }
    }

    fn delete_card(&mut self) {
        if let Some(column) = self.board.columns.get_mut(self.selected_column) {
            if self.selected_card < column.cards.len() {
                let card = column.cards.remove(self.selected_card);
                self.status_message = Some(format!("Deleted: {}", card.title));
                if self.selected_card > 0 {
                    self.selected_card -= 1;
                }
            }
        }
    }

    fn move_card_right(&mut self) {
        if self.selected_column + 1 >= self.board.columns.len() {
            return;
        }

        if let Some(column) = self.board.columns.get_mut(self.selected_column) {
            if self.selected_card < column.cards.len() {
                let card = column.cards.remove(self.selected_card);
                let next_column = &mut self.board.columns[self.selected_column + 1];
                next_column.cards.push(card);
                self.selected_column += 1;
                self.selected_card = next_column.cards.len() - 1;
                self.status_message = Some("Moved card right".to_string());
            }
        }
    }

    fn move_card_left(&mut self) {
        if self.selected_column == 0 {
            return;
        }

        if let Some(column) = self.board.columns.get_mut(self.selected_column) {
            if self.selected_card < column.cards.len() {
                let card = column.cards.remove(self.selected_card);
                let prev_column = &mut self.board.columns[self.selected_column - 1];
                prev_column.cards.push(card);
                self.selected_column -= 1;
                self.selected_card = prev_column.cards.len() - 1;
                self.status_message = Some("Moved card left".to_string());
            }
        }
    }

    async fn sync_sources(&mut self) {
        for source in &mut self.board.sources {
            source.last_sync = Some(Utc::now());
            source.sync_status = SyncStatus::Synced;
        }
        self.status_message = Some("Synced all sources".to_string());
    }

    fn do_import_from_source(&mut self) {
        if let Some(source) = self.board.sources.get(self.selected_source) {
            self.status_message = Some(format!("Imported from {}", source.name));
        }
    }

    fn link_card(&mut self) {
        self.status_message = Some("Link card to external issue (demo)".to_string());
    }

    fn unlink_card(&mut self) {
        if let Some(column) = self.board.columns.get_mut(self.selected_column) {
            if let Some(card) = column.cards.get_mut(self.selected_card) {
                if card.link.is_some() {
                    card.link = None;
                    self.status_message = Some("Unlinked card".to_string());
                }
            }
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Board => format!(
                "{} cards | h/l:columns j/k:cards n:new m:move→ M:move← d:delete q:quit",
                self.board.card_count()
            ),
            View::CardDetail => "q:back".to_string(),
            View::Sources => format!(
                "{} sources | S:sync Enter:import q:back",
                self.board.sources.len()
            ),
            View::Conflicts => format!(
                "{} conflicts | Enter:resolve q:back",
                self.conflicts.len()
            ),
            View::Import => "j/k:select Enter:import q:cancel".to_string(),
        }
    }
}
