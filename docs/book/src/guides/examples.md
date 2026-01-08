# Practical Examples

This guide provides practical examples from the TUI Suite apps, showing common patterns you can use in your own applications.

## Table with Sorting and Filtering (csv-viewer)

The csv-viewer app demonstrates a complete data table implementation with sorting, filtering, and search.

### Data Structure

```rust
pub struct CsvData {
    pub path: Option<PathBuf>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub delimiter: u8,
}

impl CsvData {
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn col_count(&self) -> usize {
        self.headers.len()
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&str> {
        self.rows.get(row).and_then(|r| r.get(col).map(|s| s.as_str()))
    }
}
```

### Smart Sorting (Numeric-aware)

```rust
pub fn sort_by_column(&mut self, col: usize, ascending: bool) {
    self.rows.sort_by(|a, b| {
        let val_a = a.get(col).map(|s| s.as_str()).unwrap_or("");
        let val_b = b.get(col).map(|s| s.as_str()).unwrap_or("");

        // Try numeric comparison first
        let cmp = match (val_a.parse::<f64>(), val_b.parse::<f64>()) {
            (Ok(num_a), Ok(num_b)) => num_a.partial_cmp(&num_b).unwrap_or(Ordering::Equal),
            _ => val_a.cmp(val_b),
        };

        if ascending { cmp } else { cmp.reverse() }
    });
}
```

### Case-insensitive Filtering

```rust
pub fn filter(&self, column: usize, query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    self.rows.iter()
        .enumerate()
        .filter(|(_, row)| {
            row.get(column)
                .map(|cell| cell.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
        })
        .map(|(i, _)| i)
        .collect()
}
```

### Multi-column Search

```rust
pub fn search(&self, query: &str) -> Vec<(usize, usize)> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for (row_idx, row) in self.rows.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            if cell.to_lowercase().contains(&query_lower) {
                results.push((row_idx, col_idx));
            }
        }
    }

    results
}
```

## Undo/Redo System (hex-editor)

The hex-editor demonstrates a complete undo/redo implementation.

### Edit Operation Stack

```rust
#[derive(Clone)]
pub struct EditOperation {
    pub offset: usize,
    pub old_value: u8,
    pub new_value: u8,
}

pub struct HexBuffer {
    pub data: Vec<u8>,
    pub modified: bool,
    pub undo_stack: Vec<EditOperation>,
    pub redo_stack: Vec<EditOperation>,
}
```

### Recording Changes

```rust
pub fn set(&mut self, offset: usize, value: u8) {
    if offset < self.data.len() {
        let old_value = self.data[offset];
        if old_value != value {
            // Record the operation for undo
            self.undo_stack.push(EditOperation {
                offset,
                old_value,
                new_value: value,
            });
            // Clear redo stack when new edit is made
            self.redo_stack.clear();
            self.data[offset] = value;
            self.modified = true;
        }
    }
}
```

### Undo and Redo

```rust
pub fn undo(&mut self) -> bool {
    if let Some(op) = self.undo_stack.pop() {
        self.data[op.offset] = op.old_value;
        self.redo_stack.push(op);
        self.modified = true;
        true
    } else {
        false
    }
}

pub fn redo(&mut self) -> bool {
    if let Some(op) = self.redo_stack.pop() {
        self.data[op.offset] = op.new_value;
        self.undo_stack.push(op);
        self.modified = true;
        true
    } else {
        false
    }
}
```

## HTTP Client (api-tester)

The api-tester shows how to build an async HTTP client with authentication.

### Request Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            // ...
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Method::GET => Method::POST,
            Method::POST => Method::PUT,
            // Cycle through methods...
            Method::OPTIONS => Method::GET,
        }
    }
}
```

### Authentication Config

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    ApiKey,
}

#[derive(Debug, Clone, Default)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub api_key: Option<String>,
    pub api_key_name: Option<String>,
}
```

### Async Request Execution

```rust
pub async fn send_request(request: &SavedRequest) -> Result<Response> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let method = match request.method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        // ...
    };

    let mut req_builder = client.request(method, &request.url);

    // Add headers
    for header in &request.headers {
        if header.enabled {
            req_builder = req_builder.header(&header.key, &header.value);
        }
    }

    // Add authentication
    match request.auth.auth_type {
        AuthType::Basic => {
            if let (Some(user), Some(pass)) = (&request.auth.username, &request.auth.password) {
                req_builder = req_builder.basic_auth(user, Some(pass));
            }
        }
        AuthType::Bearer => {
            if let Some(token) = &request.auth.token {
                req_builder = req_builder.bearer_auth(token);
            }
        }
        // ...
    }

    let start = Instant::now();
    let response = req_builder.send().await?;
    let duration = start.elapsed();

    Ok(Response {
        status: response.status().as_u16(),
        duration_ms: duration.as_millis() as u64,
        // ...
    })
}
```

## Modal Dialog Pattern

Common pattern for input dialogs used across apps.

### Mode Enum

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search(String),
    Filter(String),
    EditUrl(String),
    Confirm(ConfirmAction),
}
```

### Rendering Modal

```rust
fn render_input_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(50, 20, frame.area());

    // Clear background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show input with cursor
    let input = Paragraph::new(format!("{}█", value));
    frame.render_widget(input, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
```

## Status Bar Pattern

Common status bar implementation.

```rust
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let position = format!("Row {}/{} Col {}/{}",
        app.cursor_row + 1, app.total_rows(),
        app.cursor_col + 1, app.total_cols());

    let mode_str = match &app.mode {
        Mode::Sort => " [SORT]",
        Mode::Search(_) => " [SEARCH]",
        _ => "",
    };

    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | / Search | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(" {}{} | {} ", position, mode_str, message))
        .style(style);
    frame.render_widget(status, area);
}
```

## SQLite Database Pattern

Common pattern for SQLite storage.

```rust
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_items_name ON items(name);"
        )?;
        Ok(())
    }

    pub fn save_item(&self, item: &Item) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO items (id, name, data, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                item.id.to_string(),
                item.name,
                serde_json::to_string(&item.data)?,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
}
```

## Hex String Parsing

Utility for parsing hex input.

```rust
pub fn parse_hex_string(s: &str) -> Option<Vec<u8>> {
    // Remove spaces and validate
    let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() % 2 != 0 {
        return None;
    }

    let mut result = Vec::new();
    for i in (0..clean.len()).step_by(2) {
        let byte = u8::from_str_radix(&clean[i..i+2], 16).ok()?;
        result.push(byte);
    }
    Some(result)
}

pub fn format_hex(byte: u8) -> String {
    format!("{:02X}", byte)
}

pub fn format_ascii(byte: u8) -> char {
    if byte.is_ascii_graphic() || byte == b' ' {
        byte as char
    } else {
        '.'
    }
}
```

## cURL Command Generation

Generate cURL commands from requests.

```rust
pub fn generate_curl(request: &SavedRequest) -> String {
    let mut parts = vec![format!("curl -X {}", request.method.as_str())];

    for header in &request.headers {
        if header.enabled {
            parts.push(format!("-H '{}: {}'", header.key, header.value));
        }
    }

    match request.auth.auth_type {
        AuthType::Basic => {
            if let (Some(user), Some(pass)) = (&request.auth.username, &request.auth.password) {
                parts.push(format!("-u '{}:{}'", user, pass));
            }
        }
        AuthType::Bearer => {
            if let Some(token) = &request.auth.token {
                parts.push(format!("-H 'Authorization: Bearer {}'", token));
            }
        }
        // ...
    }

    if let Some(body) = &request.body {
        parts.push(format!("-d '{}'", body.replace('\'', "'\\''")));
    }

    parts.push(format!("'{}'", request.url));
    parts.join(" \\\n  ")
}
```

## Keyboard Handling Pattern

Standard keyboard handling with mode-aware input.

```rust
pub fn handle_key(&mut self, key: KeyEvent) -> bool {
    self.message = None;
    self.error = None;

    match &self.mode {
        Mode::Normal => self.handle_normal_key(key),
        Mode::Search(query) => self.handle_search_key(key, query.clone()),
        Mode::Filter(query) => self.handle_filter_key(key, query.clone()),
        Mode::Confirm(action) => self.handle_confirm_key(key, action.clone()),
    }
}

fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('?') => self.view = View::Help,
        KeyCode::Char('/') => self.mode = Mode::Search(String::new()),
        KeyCode::Esc => {
            if self.view != View::Main {
                self.view = View::Main;
            }
        }
        // Navigation
        KeyCode::Down | KeyCode::Char('j') => self.move_down(),
        KeyCode::Up | KeyCode::Char('k') => self.move_up(),
        KeyCode::PageDown => self.page_down(),
        KeyCode::PageUp => self.page_up(),
        KeyCode::Home => self.go_to_start(),
        KeyCode::End => self.go_to_end(),
        _ => {}
    }
    false
}

fn handle_search_key(&mut self, key: KeyEvent, mut query: String) -> bool {
    match key.code {
        KeyCode::Enter => {
            self.execute_search(&query);
            self.mode = Mode::Normal;
        }
        KeyCode::Esc => self.mode = Mode::Normal,
        KeyCode::Backspace => {
            query.pop();
            self.mode = Mode::Search(query);
        }
        KeyCode::Char(c) => {
            query.push(c);
            self.mode = Mode::Search(query);
        }
        _ => self.mode = Mode::Search(query),
    }
    false
}
```
