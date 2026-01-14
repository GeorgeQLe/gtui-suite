use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub row_count: usize,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected_rows: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Tables,
    Data,
    Schema,
    Query,
}

pub struct App {
    pub db_path: Option<String>,
    pub tables: Vec<Table>,
    pub selected_table: usize,
    pub view: View,
    pub query: String,
    pub query_result: Option<QueryResult>,
    pub data_rows: Vec<Vec<String>>,
    pub data_columns: Vec<String>,
    pub selected_row: usize,
    pub column_offset: usize,
    pub is_editing_query: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            db_path: None,
            tables: Vec::new(),
            selected_table: 0,
            view: View::Tables,
            query: String::new(),
            query_result: None,
            data_rows: Vec::new(),
            data_columns: Vec::new(),
            selected_row: 0,
            column_offset: 0,
            is_editing_query: false,
            status_message: None,
        }
    }

    pub fn load_demo_data(&mut self) {
        self.db_path = Some("demo.db".to_string());

        self.tables = vec![
            Table {
                name: "users".to_string(),
                row_count: 5,
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    Column {
                        name: "name".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "email".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "created_at".to_string(),
                        col_type: "DATETIME".to_string(),
                        nullable: true,
                        primary_key: false,
                    },
                ],
            },
            Table {
                name: "posts".to_string(),
                row_count: 12,
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    Column {
                        name: "user_id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "title".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "content".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: true,
                        primary_key: false,
                    },
                ],
            },
            Table {
                name: "comments".to_string(),
                row_count: 28,
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    Column {
                        name: "post_id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    Column {
                        name: "body".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                ],
            },
            Table {
                name: "settings".to_string(),
                row_count: 8,
                columns: vec![
                    Column {
                        name: "key".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    Column {
                        name: "value".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: true,
                        primary_key: false,
                    },
                ],
            },
        ];

        self.load_table_data(0);
    }

    fn load_table_data(&mut self, table_idx: usize) {
        if let Some(table) = self.tables.get(table_idx) {
            self.data_columns = table.columns.iter().map(|c| c.name.clone()).collect();

            // Generate demo data
            self.data_rows = match table.name.as_str() {
                "users" => vec![
                    vec!["1".into(), "Alice".into(), "alice@example.com".into(), "2024-01-01".into()],
                    vec!["2".into(), "Bob".into(), "bob@example.com".into(), "2024-01-02".into()],
                    vec!["3".into(), "Charlie".into(), "charlie@example.com".into(), "2024-01-03".into()],
                    vec!["4".into(), "Diana".into(), "diana@example.com".into(), "2024-01-04".into()],
                    vec!["5".into(), "Eve".into(), "eve@example.com".into(), "2024-01-05".into()],
                ],
                "posts" => vec![
                    vec!["1".into(), "1".into(), "Hello World".into(), "My first post".into()],
                    vec!["2".into(), "1".into(), "Rust is great".into(), "Learning Rust...".into()],
                    vec!["3".into(), "2".into(), "SQLite tips".into(), "Some useful tips".into()],
                ],
                "comments" => vec![
                    vec!["1".into(), "1".into(), "Great post!".into()],
                    vec!["2".into(), "1".into(), "Thanks for sharing".into()],
                    vec!["3".into(), "2".into(), "I agree".into()],
                ],
                "settings" => vec![
                    vec!["theme".into(), "dark".into()],
                    vec!["language".into(), "en".into()],
                    vec!["timezone".into(), "UTC".into()],
                ],
                _ => Vec::new(),
            };

            self.selected_row = 0;
            self.column_offset = 0;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        if self.is_editing_query {
            return self.handle_query_edit(key);
        }

        match self.view {
            View::Tables => self.handle_tables_key(key),
            View::Data => self.handle_data_key(key),
            View::Schema => self.handle_schema_key(key),
            View::Query => self.handle_query_key(key),
        }
    }

    fn handle_tables_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_table < self.tables.len().saturating_sub(1) {
                    self.selected_table += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_table = self.selected_table.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('d') => {
                self.load_table_data(self.selected_table);
                self.view = View::Data;
            }
            KeyCode::Char('s') => {
                self.view = View::Schema;
            }
            KeyCode::Char('/') => {
                self.view = View::Query;
                self.is_editing_query = true;
            }
            _ => {}
        }
        false
    }

    fn handle_data_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Tables;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_row < self.data_rows.len().saturating_sub(1) {
                    self.selected_row += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_row = self.selected_row.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.column_offset < self.data_columns.len().saturating_sub(1) {
                    self.column_offset += 1;
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.column_offset = self.column_offset.saturating_sub(1);
            }
            KeyCode::Char('s') => {
                self.view = View::Schema;
            }
            KeyCode::Char('/') => {
                self.view = View::Query;
                self.is_editing_query = true;
            }
            _ => {}
        }
        false
    }

    fn handle_schema_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Tables;
            }
            KeyCode::Char('d') => {
                self.view = View::Data;
            }
            _ => {}
        }
        false
    }

    fn handle_query_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::Tables;
            }
            KeyCode::Enter | KeyCode::Char('e') => {
                self.is_editing_query = true;
            }
            _ => {}
        }
        false
    }

    fn handle_query_edit(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.is_editing_query = false;
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.execute_query();
                self.is_editing_query = false;
            }
            KeyCode::Backspace => {
                self.query.pop();
            }
            KeyCode::Char(c) => {
                self.query.push(c);
            }
            KeyCode::Enter => {
                self.query.push('\n');
            }
            _ => {}
        }
        false
    }

    fn execute_query(&mut self) {
        // Demo query execution
        if self.query.to_lowercase().starts_with("select") {
            self.query_result = Some(QueryResult {
                columns: vec!["id".into(), "name".into()],
                rows: vec![
                    vec!["1".into(), "Result 1".into()],
                    vec!["2".into(), "Result 2".into()],
                ],
                affected_rows: None,
                error: None,
            });
        } else {
            self.query_result = Some(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                affected_rows: Some(1),
                error: None,
            });
        }
        self.status_message = Some("Query executed".to_string());
    }

    pub fn selected_table_info(&self) -> Option<&Table> {
        self.tables.get(self.selected_table)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        if self.is_editing_query {
            return "Ctrl+Enter:execute Esc:cancel".to_string();
        }

        match self.view {
            View::Tables => "Enter/d:data s:schema /:query q:quit".to_string(),
            View::Data => "j/k:rows h/l:columns s:schema /:query Esc:back".to_string(),
            View::Schema => "d:data Esc:back".to_string(),
            View::Query => "e:edit Enter:execute Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
