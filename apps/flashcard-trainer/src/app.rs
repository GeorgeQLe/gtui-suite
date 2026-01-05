//! Application state and logic.

use crate::algorithm::{get_algorithm, SrsAlgorithm};
use crate::config::Config;
use crate::db::{Database, DbResult};
use crate::models::{Card, CardSchedule, Deck, DeckId, DeckStats, Response, Review, Session};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;

pub struct App {
    pub db: Database,
    pub config: Config,
    pub view: View,
    pub decks: Vec<Deck>,
    pub selected_deck: usize,
    pub deck_stats: HashMap<DeckId, DeckStats>,
    pub session: Option<Session>,
    pub current_card: Option<Card>,
    pub current_schedule: Option<CardSchedule>,
    pub algorithm: Box<dyn SrsAlgorithm>,
    pub editing: bool,
    pub input_buffer: String,
    pub input_field: InputField,
    pub message: Option<String>,
    pub show_help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    DeckList,
    Study,
    Stats,
    CardBrowser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    None,
    DeckName,
    CardFront,
    CardBack,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "flashcards.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;

        let algorithm = get_algorithm(&config.algorithm.default_algorithm);

        let mut app = Self {
            db,
            config,
            view: View::DeckList,
            decks: Vec::new(),
            selected_deck: 0,
            deck_stats: HashMap::new(),
            session: None,
            current_card: None,
            current_schedule: None,
            algorithm,
            editing: false,
            input_buffer: String::new(),
            input_field: InputField::None,
            message: None,
            show_help: false,
        };

        app.refresh_decks()?;
        Ok(app)
    }

    pub fn refresh_decks(&mut self) -> DbResult<()> {
        self.decks = self.db.list_decks()?;
        self.deck_stats.clear();
        for deck in &self.decks {
            if let Ok(stats) = self.db.get_deck_stats(deck.id) {
                self.deck_stats.insert(deck.id, stats);
            }
        }
        if self.selected_deck >= self.decks.len() && !self.decks.is_empty() {
            self.selected_deck = self.decks.len() - 1;
        }
        Ok(())
    }

    pub fn can_quit(&self) -> bool {
        !self.editing && self.session.is_none()
    }

    pub fn selected_deck(&self) -> Option<&Deck> {
        self.decks.get(self.selected_deck)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.message = None;

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.editing {
            self.handle_edit_key(key);
            return;
        }

        match self.view {
            View::DeckList => self.handle_deck_list_key(key),
            View::Study => self.handle_study_key(key),
            View::Stats => self.handle_stats_key(key),
            View::CardBrowser => self.handle_browser_key(key),
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.editing = false;
                self.input_buffer.clear();
                self.input_field = InputField::None;
            }
            KeyCode::Enter => self.finish_editing(),
            KeyCode::Backspace => { self.input_buffer.pop(); }
            KeyCode::Char(c) => self.input_buffer.push(c),
            _ => {}
        }
    }

    fn handle_deck_list_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.decks.is_empty() {
                    self.selected_deck = (self.selected_deck + 1).min(self.decks.len() - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_deck = self.selected_deck.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.start_study(),
            KeyCode::Char('a') => {
                self.editing = true;
                self.input_field = InputField::DeckName;
                self.input_buffer.clear();
            }
            KeyCode::Char('s') => self.view = View::Stats,
            KeyCode::Char('b') => self.view = View::CardBrowser,
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }

    fn handle_study_key(&mut self, key: KeyEvent) {
        let Some(session) = &mut self.session else {
            self.view = View::DeckList;
            return;
        };

        if session.is_complete() {
            self.end_session();
            return;
        }

        if !session.flipped {
            match key.code {
                KeyCode::Char(' ') | KeyCode::Enter => session.flipped = true,
                KeyCode::Char('q') | KeyCode::Esc => self.end_session(),
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('1') => self.answer(Response::Again),
                KeyCode::Char('2') => self.answer(Response::Hard),
                KeyCode::Char('3') => self.answer(Response::Good),
                KeyCode::Char('4') => self.answer(Response::Easy),
                KeyCode::Char('q') | KeyCode::Esc => self.end_session(),
                _ => {}
            }
        }
    }

    fn handle_stats_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.view = View::DeckList,
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }

    fn handle_browser_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.view = View::DeckList,
            KeyCode::Char('a') => {
                self.editing = true;
                self.input_field = InputField::CardFront;
                self.input_buffer.clear();
            }
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }

    fn finish_editing(&mut self) {
        match self.input_field {
            InputField::DeckName => {
                if !self.input_buffer.is_empty() {
                    let deck = Deck::new(&self.input_buffer);
                    if self.db.insert_deck(&deck).is_ok() {
                        self.message = Some("Deck created".to_string());
                        let _ = self.refresh_decks();
                    }
                }
            }
            InputField::CardFront => {
                // Store front, prompt for back
                self.input_field = InputField::CardBack;
                return; // Don't clear editing state
            }
            InputField::CardBack => {
                if let Some(deck) = self.selected_deck() {
                    // Create card with stored front
                    let front = "TODO: Store front temporarily".to_string();
                    let card = Card::new_basic(deck.id, front, &self.input_buffer);
                    if self.db.insert_card(&card).is_ok() {
                        self.message = Some("Card created".to_string());
                    }
                }
            }
            InputField::None => {}
        }
        self.editing = false;
        self.input_buffer.clear();
        self.input_field = InputField::None;
    }

    fn start_study(&mut self) {
        let Some(deck) = self.selected_deck().cloned() else {
            return;
        };

        let session_config = self.config.to_session_config();

        let cards_due = self.db.get_due_cards(deck.id).unwrap_or_default();
        let cards_new = self.db.get_new_cards(deck.id, session_config.new_cards_per_day).unwrap_or_default();

        if cards_due.is_empty() && cards_new.is_empty() {
            self.message = Some("No cards to study!".to_string());
            return;
        }

        let session = Session::new(deck.id, cards_due, cards_new);
        self.session = Some(session);
        self.view = View::Study;
        self.load_current_card();
    }

    fn load_current_card(&mut self) {
        let Some(session) = &self.session else { return };
        let Some(card_id) = session.current_card() else { return };

        self.current_card = self.db.get_card(card_id).ok().flatten();
        self.current_schedule = self.db.get_schedule(card_id).ok().flatten();
    }

    fn answer(&mut self, response: Response) {
        let Some(schedule) = &self.current_schedule else { return };

        // Calculate new schedule
        let new_schedule = self.algorithm.calculate_next_review(schedule, response);

        // Update database
        let mut updated = schedule.clone();
        updated.due = new_schedule.next_review;
        updated.interval = new_schedule.interval.num_days();
        updated.ease_factor = new_schedule.ease_factor;
        updated.state = new_schedule.state;
        updated.review_count += 1;
        if matches!(response, Response::Again) {
            updated.lapses += 1;
        }
        let _ = self.db.upsert_schedule(&updated);

        // Record review
        if let Some(session) = &self.session {
            let review = Review::new(
                schedule.card_id,
                response,
                session.card_time().num_milliseconds(),
            );
            let _ = self.db.insert_review(&review);
        }

        // Update session
        if let Some(session) = &mut self.session {
            session.record_response(response);
            session.next_card();
        }

        self.load_current_card();
    }

    fn end_session(&mut self) {
        self.session = None;
        self.current_card = None;
        self.current_schedule = None;
        self.view = View::DeckList;
        let _ = self.refresh_decks();
    }
}
