#![allow(dead_code)]

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};
use uuid::Uuid;

use crate::board::{Board, Card, ChecklistItem, Column};
use crate::config::Config;
use crate::database::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    BoardList,
    Board,
    CardDetail,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AddBoard(String),
    AddColumn(String),
    AddCard(String),
    EditTitle(String),
    AddChecklist(String),
}

pub struct App {
    pub db: Database,
    pub config: Config,
    pub view: View,
    pub mode: Mode,

    // Board list state
    pub boards: Vec<Board>,
    pub board_index: usize,

    // Current board state
    pub current_board: Option<Board>,
    pub columns: Vec<Column>,
    pub cards: Vec<Vec<Card>>, // Cards per column
    pub column_index: usize,
    pub card_index: usize,

    // Card detail state
    pub detail_card: Option<Card>,
    pub checklist_index: usize,

    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new(db: Database, config: Config) -> Self {
        Self {
            db,
            config,
            view: View::BoardList,
            mode: Mode::Normal,
            boards: Vec::new(),
            board_index: 0,
            current_board: None,
            columns: Vec::new(),
            cards: Vec::new(),
            column_index: 0,
            card_index: 0,
            detail_card: None,
            checklist_index: 0,
            message: None,
            error: None,
        }
    }

    pub fn load_boards(&mut self) -> Result<()> {
        self.boards = self.db.list_boards()?;
        if self.boards.is_empty() {
            // Create a default board
            let default_board = Board::new(self.config.board.default.clone());
            self.db.save_board(&default_board)?;

            // Create default columns
            let columns = ["To Do", "In Progress", "Done"];
            for (i, name) in columns.iter().enumerate() {
                let mut col = Column::new(default_board.id, name.to_string());
                col.position = i as i32;
                self.db.save_column(&col)?;
            }

            self.boards = self.db.list_boards()?;
        }
        self.board_index = 0;
        Ok(())
    }

    pub fn load_board(&mut self, board_id: Uuid) -> Result<()> {
        self.current_board = self.boards.iter().find(|b| b.id == board_id).cloned();
        self.columns = self.db.list_columns(board_id)?;
        self.cards = Vec::new();
        for column in &self.columns {
            let column_cards = self.db.list_cards(column.id)?;
            self.cards.push(column_cards);
        }
        self.column_index = 0;
        self.card_index = 0;
        Ok(())
    }

    pub fn current_column_cards(&self) -> &[Card] {
        self.cards.get(self.column_index).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn current_card(&self) -> Option<&Card> {
        self.cards.get(self.column_index)?.get(self.card_index)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match &self.mode {
            Mode::Normal => match self.view {
                View::BoardList => self.handle_board_list_key(key),
                View::Board => self.handle_board_key(key),
                View::CardDetail => self.handle_card_detail_key(key),
                View::Help => self.handle_help_key(key),
            },
            Mode::AddBoard(_) | Mode::AddColumn(_) | Mode::AddCard(_) | Mode::EditTitle(_) | Mode::AddChecklist(_) => {
                self.handle_input_key(key)
            }
        }
    }

    fn handle_board_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.board_index < self.boards.len().saturating_sub(1) {
                    self.board_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.board_index > 0 {
                    self.board_index -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(board) = self.boards.get(self.board_index) {
                    let board_id = board.id;
                    if let Err(e) = self.load_board(board_id) {
                        self.error = Some(format!("Failed to load board: {}", e));
                    } else {
                        self.view = View::Board;
                    }
                }
            }
            KeyCode::Char('a') => {
                self.mode = Mode::AddBoard(String::new());
            }
            KeyCode::Char('d') => {
                if let Some(board) = self.boards.get(self.board_index) {
                    let board_id = board.id;
                    if let Err(e) = self.db.delete_board(board_id) {
                        self.error = Some(format!("Failed to delete: {}", e));
                    } else {
                        let _ = self.load_boards();
                        self.message = Some("Board deleted".to_string());
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn handle_board_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::BoardList;
                self.current_board = None;
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Right | KeyCode::Char('l') => {
                if self.column_index < self.columns.len().saturating_sub(1) {
                    self.column_index += 1;
                    let max_cards = self.current_column_cards().len().saturating_sub(1);
                    self.card_index = self.card_index.min(max_cards);
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.column_index > 0 {
                    self.column_index -= 1;
                    let max_cards = self.current_column_cards().len().saturating_sub(1);
                    self.card_index = self.card_index.min(max_cards);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.current_column_cards().len().saturating_sub(1);
                if self.card_index < max {
                    self.card_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.card_index > 0 {
                    self.card_index -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(card) = self.current_card().cloned() {
                    self.detail_card = Some(card);
                    self.checklist_index = 0;
                    self.view = View::CardDetail;
                }
            }
            KeyCode::Char('a') => {
                self.mode = Mode::AddCard(String::new());
            }
            KeyCode::Char('A') => {
                self.mode = Mode::AddColumn(String::new());
            }
            KeyCode::Char('d') => {
                if let Some(card) = self.current_card() {
                    let card_id = card.id;
                    if let Err(e) = self.db.delete_card(card_id) {
                        self.error = Some(format!("Failed to delete: {}", e));
                    } else if let Some(board) = &self.current_board {
                        let _ = self.load_board(board.id);
                        self.message = Some("Card deleted".to_string());
                    }
                }
            }
            KeyCode::Char('p') => {
                // Toggle priority
                if let Some(cards) = self.cards.get_mut(self.column_index) {
                    if let Some(card) = cards.get_mut(self.card_index) {
                        card.priority = card.priority.next();
                        card.updated_at = Utc::now();
                        let _ = self.db.save_card(card);
                    }
                }
            }
            KeyCode::Char('H') => self.move_card_left(),
            KeyCode::Char('L') => self.move_card_right(),
            KeyCode::Char('K') => self.move_card_up(),
            KeyCode::Char('J') => self.move_card_down(),
            _ => {}
        }
        false
    }

    fn handle_card_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Save changes and return to board
                if let Some(card) = &self.detail_card {
                    let _ = self.db.save_card(card);
                }
                if let Some(board) = &self.current_board {
                    let _ = self.load_board(board.id);
                }
                self.detail_card = None;
                self.view = View::Board;
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(card) = &self.detail_card {
                    if self.checklist_index < card.checklist.len().saturating_sub(1) {
                        self.checklist_index += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.checklist_index > 0 {
                    self.checklist_index -= 1;
                }
            }
            KeyCode::Char(' ') => {
                // Toggle checklist item
                if let Some(card) = &mut self.detail_card {
                    if let Some(item) = card.checklist.get_mut(self.checklist_index) {
                        item.completed = !item.completed;
                        card.updated_at = Utc::now();
                    }
                }
            }
            KeyCode::Char('e') => {
                if let Some(card) = &self.detail_card {
                    self.mode = Mode::EditTitle(card.title.clone());
                }
            }
            KeyCode::Char('c') => {
                self.mode = Mode::AddChecklist(String::new());
            }
            KeyCode::Char('p') => {
                if let Some(card) = &mut self.detail_card {
                    card.priority = card.priority.next();
                    card.updated_at = Utc::now();
                }
            }
            _ => {}
        }
        false
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                self.view = if self.current_board.is_some() {
                    View::Board
                } else {
                    View::BoardList
                };
            }
            _ => {}
        }
        false
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        match mode {
            Mode::AddBoard(mut text) => match key.code {
                KeyCode::Enter => {
                    if !text.is_empty() {
                        let board = Board::new(text);
                        if let Err(e) = self.db.save_board(&board) {
                            self.error = Some(format!("Failed to save: {}", e));
                        } else {
                            let _ = self.load_boards();
                            self.message = Some("Board created".to_string());
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::AddBoard(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::AddBoard(text);
                }
                _ => self.mode = Mode::AddBoard(text),
            },
            Mode::AddColumn(mut text) => match key.code {
                KeyCode::Enter => {
                    if !text.is_empty() {
                        if let Some(board) = &self.current_board {
                            let mut column = Column::new(board.id, text);
                            column.position = self.columns.len() as i32;
                            if let Err(e) = self.db.save_column(&column) {
                                self.error = Some(format!("Failed to save: {}", e));
                            } else {
                                let _ = self.load_board(board.id);
                                self.message = Some("Column created".to_string());
                            }
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::AddColumn(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::AddColumn(text);
                }
                _ => self.mode = Mode::AddColumn(text),
            },
            Mode::AddCard(mut text) => match key.code {
                KeyCode::Enter => {
                    if !text.is_empty() {
                        if let Some(column) = self.columns.get(self.column_index) {
                            let mut card = Card::new(column.id, text);
                            card.position = self.current_column_cards().len() as i32;
                            if let Err(e) = self.db.save_card(&card) {
                                self.error = Some(format!("Failed to save: {}", e));
                            } else if let Some(board) = &self.current_board {
                                let _ = self.load_board(board.id);
                                self.message = Some("Card created".to_string());
                            }
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::AddCard(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::AddCard(text);
                }
                _ => self.mode = Mode::AddCard(text),
            },
            Mode::EditTitle(mut text) => match key.code {
                KeyCode::Enter => {
                    if !text.is_empty() {
                        if let Some(card) = &mut self.detail_card {
                            card.title = text;
                            card.updated_at = Utc::now();
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::EditTitle(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::EditTitle(text);
                }
                _ => self.mode = Mode::EditTitle(text),
            },
            Mode::AddChecklist(mut text) => match key.code {
                KeyCode::Enter => {
                    if !text.is_empty() {
                        if let Some(card) = &mut self.detail_card {
                            card.checklist.push(ChecklistItem {
                                id: Uuid::new_v4(),
                                text,
                                completed: false,
                            });
                            card.updated_at = Utc::now();
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::AddChecklist(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::AddChecklist(text);
                }
                _ => self.mode = Mode::AddChecklist(text),
            },
            Mode::Normal => {}
        }
        false
    }

    fn move_card_left(&mut self) {
        if self.column_index == 0 {
            return;
        }
        if let Some(cards) = self.cards.get_mut(self.column_index) {
            if let Some(mut card) = cards.get(self.card_index).cloned() {
                // Remove from current column
                cards.remove(self.card_index);

                // Move to previous column
                let new_col_idx = self.column_index - 1;
                if let Some(new_column) = self.columns.get(new_col_idx) {
                    card.column_id = new_column.id;
                    card.position = self.cards.get(new_col_idx).map(|c| c.len()).unwrap_or(0) as i32;
                    card.updated_at = Utc::now();
                    let _ = self.db.save_card(&card);

                    if let Some(new_cards) = self.cards.get_mut(new_col_idx) {
                        new_cards.push(card);
                        self.column_index = new_col_idx;
                        self.card_index = new_cards.len().saturating_sub(1);
                    }
                }
            }
        }
    }

    fn move_card_right(&mut self) {
        if self.column_index >= self.columns.len().saturating_sub(1) {
            return;
        }
        if let Some(cards) = self.cards.get_mut(self.column_index) {
            if let Some(mut card) = cards.get(self.card_index).cloned() {
                cards.remove(self.card_index);

                let new_col_idx = self.column_index + 1;
                if let Some(new_column) = self.columns.get(new_col_idx) {
                    card.column_id = new_column.id;
                    card.position = self.cards.get(new_col_idx).map(|c| c.len()).unwrap_or(0) as i32;
                    card.updated_at = Utc::now();
                    let _ = self.db.save_card(&card);

                    if let Some(new_cards) = self.cards.get_mut(new_col_idx) {
                        new_cards.push(card);
                        self.column_index = new_col_idx;
                        self.card_index = new_cards.len().saturating_sub(1);
                    }
                }
            }
        }
    }

    fn move_card_up(&mut self) {
        if self.card_index == 0 {
            return;
        }
        if let Some(cards) = self.cards.get_mut(self.column_index) {
            if self.card_index < cards.len() {
                cards.swap(self.card_index, self.card_index - 1);
                // Update positions
                if let Some(card) = cards.get_mut(self.card_index) {
                    card.position = self.card_index as i32;
                    card.updated_at = Utc::now();
                    let _ = self.db.save_card(card);
                }
                if let Some(card) = cards.get_mut(self.card_index - 1) {
                    card.position = (self.card_index - 1) as i32;
                    card.updated_at = Utc::now();
                    let _ = self.db.save_card(card);
                }
                self.card_index -= 1;
            }
        }
    }

    fn move_card_down(&mut self) {
        if let Some(cards) = self.cards.get_mut(self.column_index) {
            if self.card_index < cards.len().saturating_sub(1) {
                cards.swap(self.card_index, self.card_index + 1);
                // Update positions
                if let Some(card) = cards.get_mut(self.card_index) {
                    card.position = self.card_index as i32;
                    card.updated_at = Utc::now();
                    let _ = self.db.save_card(card);
                }
                if let Some(card) = cards.get_mut(self.card_index + 1) {
                    card.position = (self.card_index + 1) as i32;
                    card.updated_at = Utc::now();
                    let _ = self.db.save_card(card);
                }
                self.card_index += 1;
            }
        }
    }
}
