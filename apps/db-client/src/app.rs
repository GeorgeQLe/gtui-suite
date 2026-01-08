use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use anyhow::Result;

use crate::config::{Config, ConnectionInfo};
use crate::database::{Database, TableInfo, ColumnInfo, QueryResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Tables,
    Data,
    Schema,
    Query,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Tables,
    Content,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    QueryInput(String),
    OpenFile(String),
}

pub struct App {
    pub config: Config,
    pub db: Option<Database>,
    pub view: View,
    pub mode: Mode,
    pub pane: Pane,

    // Tables
    pub tables: Vec<TableInfo>,
    pub selected_table: usize,

    // Schema
    pub schema: Vec<ColumnInfo>,

    // Data view
    pub data: Option<QueryResult>,
    pub data_scroll: usize,
    pub data_offset: usize,
    pub selected_row: usize,
    pub selected_col: usize,

    // Query
    pub query_history: Vec<String>,
    pub query_result: Option<QueryResult>,

    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            db: None,
            view: View::Tables,
            mode: Mode::Normal,
            pane: Pane::Tables,
            tables: Vec::new(),
            selected_table: 0,
            schema: Vec::new(),
            data: None,
            data_scroll: 0,
            data_offset: 0,
            selected_row: 0,
            selected_col: 0,
            query_history: Vec::new(),
            query_result: None,
            message: None,
            error: None,
        }
    }

    pub fn open_database(&mut self, path: &str) -> Result<()> {
        let db = Database::open(path)?;
        self.tables = db.list_tables()?;
        self.db = Some(db);
        self.selected_table = 0;
        self.view = View::Tables;

        // Add to recent
        self.config.add_recent(ConnectionInfo {
            name: path.split('/').last().unwrap_or(path).to_string(),
            db_type: "sqlite".to_string(),
            path: path.to_string(),
        });
        let _ = self.config.save();

        self.message = Some(format!("Opened: {}", path));
        Ok(())
    }

    pub fn current_table(&self) -> Option<&str> {
        self.tables.get(self.selected_table).map(|t| t.name.as_str())
    }

    pub fn load_table_data(&mut self) {
        if let (Some(db), Some(table)) = (&self.db, self.current_table()) {
            match db.get_table_data(table, self.config.display.max_rows, self.data_offset) {
                Ok(result) => {
                    self.data = Some(result);
                    self.selected_row = 0;
                    self.selected_col = 0;
                    self.data_scroll = 0;
                }
                Err(e) => {
                    self.error = Some(format!("Error loading data: {}", e));
                }
            }
        }
    }

    pub fn load_table_schema(&mut self) {
        if let (Some(db), Some(table)) = (&self.db, self.current_table()) {
            match db.get_table_schema(table) {
                Ok(schema) => {
                    self.schema = schema;
                }
                Err(e) => {
                    self.error = Some(format!("Error loading schema: {}", e));
                }
            }
        }
    }

    pub fn execute_query(&mut self, sql: &str) {
        if let Some(db) = &self.db {
            match db.query(sql) {
                Ok(result) => {
                    self.query_result = Some(result);
                    self.query_history.push(sql.to_string());
                    self.message = Some("Query executed successfully".to_string());
                }
                Err(e) => {
                    self.error = Some(format!("Query error: {}", e));
                }
            }
        } else {
            self.error = Some("No database connected".to_string());
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::QueryInput(_) => self.handle_query_input(key),
            Mode::OpenFile(_) => self.handle_open_file(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Esc => {
                if self.view == View::Help {
                    self.view = View::Tables;
                }
            }

            // Tab between panes
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::Tables => Pane::Content,
                    Pane::Content => Pane::Tables,
                };
            }

            // View switching
            KeyCode::Char('1') => {
                self.view = View::Tables;
            }
            KeyCode::Char('2') => {
                self.view = View::Data;
                self.load_table_data();
            }
            KeyCode::Char('3') => {
                self.view = View::Schema;
                self.load_table_schema();
            }
            KeyCode::Char('4') => {
                self.view = View::Query;
            }

            // Open file
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.mode = Mode::OpenFile(String::new());
            }

            // Query mode
            KeyCode::Char(':') => {
                self.mode = Mode::QueryInput(String::new());
            }

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                match self.pane {
                    Pane::Tables => {
                        if self.selected_table < self.tables.len().saturating_sub(1) {
                            self.selected_table += 1;
                        }
                    }
                    Pane::Content => {
                        if let Some(data) = &self.data {
                            if self.selected_row < data.rows.len().saturating_sub(1) {
                                self.selected_row += 1;
                            }
                        }
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.pane {
                    Pane::Tables => {
                        if self.selected_table > 0 {
                            self.selected_table -= 1;
                        }
                    }
                    Pane::Content => {
                        if self.selected_row > 0 {
                            self.selected_row -= 1;
                        }
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.pane == Pane::Content && self.selected_col > 0 {
                    self.selected_col -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let (Pane::Content, Some(data)) = (&self.pane, &self.data) {
                    if self.selected_col < data.columns.len().saturating_sub(1) {
                        self.selected_col += 1;
                    }
                }
            }

            // Enter to load table data
            KeyCode::Enter => {
                if self.pane == Pane::Tables && !self.tables.is_empty() {
                    self.view = View::Data;
                    self.load_table_data();
                    self.pane = Pane::Content;
                }
            }

            // Refresh
            KeyCode::Char('r') | KeyCode::F(5) => {
                if let Some(db) = &self.db {
                    match db.list_tables() {
                        Ok(tables) => {
                            self.tables = tables;
                            self.message = Some("Refreshed".to_string());
                        }
                        Err(e) => {
                            self.error = Some(format!("Error: {}", e));
                        }
                    }
                }
            }

            _ => {}
        }

        false
    }

    fn handle_query_input(&mut self, key: KeyEvent) -> bool {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        if let Mode::QueryInput(mut query) = mode {
            match key.code {
                KeyCode::Enter => {
                    if !query.is_empty() {
                        self.execute_query(&query);
                        self.view = View::Query;
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    query.pop();
                    self.mode = Mode::QueryInput(query);
                }
                KeyCode::Char(c) => {
                    query.push(c);
                    self.mode = Mode::QueryInput(query);
                }
                _ => self.mode = Mode::QueryInput(query),
            }
        }
        false
    }

    fn handle_open_file(&mut self, key: KeyEvent) -> bool {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        if let Mode::OpenFile(mut path) = mode {
            match key.code {
                KeyCode::Enter => {
                    if !path.is_empty() {
                        if let Err(e) = self.open_database(&path) {
                            self.error = Some(format!("Failed to open: {}", e));
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    path.pop();
                    self.mode = Mode::OpenFile(path);
                }
                KeyCode::Char(c) => {
                    path.push(c);
                    self.mode = Mode::OpenFile(path);
                }
                _ => self.mode = Mode::OpenFile(path),
            }
        }
        false
    }
}
