use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub enum HttpMethod { Get, Post, Put, Delete, Patch, Head, Options }

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub status: u16,
    pub duration_ms: u64,
    pub request_size: usize,
    pub response_size: usize,
    pub timestamp: DateTime<Local>,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode { List, RequestDetail, ResponseDetail }

pub struct App {
    pub requests: Vec<HttpRequest>,
    pub selected: usize,
    pub view_mode: ViewMode,
    pub scroll_offset: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            requests: vec![
                HttpRequest {
                    method: HttpMethod::Get, url: "https://api.example.com/users".into(),
                    status: 200, duration_ms: 145, request_size: 256, response_size: 4096,
                    timestamp: Local::now(),
                    headers: vec![("Content-Type".into(), "application/json".into()), ("Authorization".into(), "Bearer xxx".into())],
                    body: Some(r#"{"users": [{"id": 1, "name": "John"}]}"#.into()),
                },
                HttpRequest {
                    method: HttpMethod::Post, url: "https://api.example.com/users".into(),
                    status: 201, duration_ms: 234, request_size: 512, response_size: 128,
                    timestamp: Local::now(),
                    headers: vec![("Content-Type".into(), "application/json".into())],
                    body: Some(r#"{"name": "Jane", "email": "jane@example.com"}"#.into()),
                },
                HttpRequest {
                    method: HttpMethod::Get, url: "https://api.example.com/products?page=1".into(),
                    status: 200, duration_ms: 89, request_size: 128, response_size: 8192,
                    timestamp: Local::now(),
                    headers: vec![("Accept".into(), "application/json".into())],
                    body: None,
                },
                HttpRequest {
                    method: HttpMethod::Delete, url: "https://api.example.com/users/5".into(),
                    status: 404, duration_ms: 45, request_size: 64, response_size: 64,
                    timestamp: Local::now(),
                    headers: vec![],
                    body: Some(r#"{"error": "User not found"}"#.into()),
                },
                HttpRequest {
                    method: HttpMethod::Put, url: "https://api.example.com/config".into(),
                    status: 500, duration_ms: 1234, request_size: 1024, response_size: 256,
                    timestamp: Local::now(),
                    headers: vec![("Content-Type".into(), "application/json".into())],
                    body: Some(r#"{"error": "Internal server error"}"#.into()),
                },
            ],
            selected: 0,
            view_mode: ViewMode::List,
            scroll_offset: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.view_mode != ViewMode::List {
                    self.view_mode = ViewMode::List;
                } else {
                    return true;
                }
            },
            KeyCode::Char('j') | KeyCode::Down => {
                if self.view_mode == ViewMode::List {
                    if self.selected < self.requests.len().saturating_sub(1) { self.selected += 1; }
                } else {
                    self.scroll_offset += 1;
                }
            },
            KeyCode::Char('k') | KeyCode::Up => {
                if self.view_mode == ViewMode::List {
                    self.selected = self.selected.saturating_sub(1);
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
            },
            KeyCode::Enter => {
                if self.view_mode == ViewMode::List {
                    self.view_mode = ViewMode::RequestDetail;
                    self.scroll_offset = 0;
                }
            },
            KeyCode::Tab => {
                self.view_mode = match self.view_mode {
                    ViewMode::List => ViewMode::RequestDetail,
                    ViewMode::RequestDetail => ViewMode::ResponseDetail,
                    ViewMode::ResponseDetail => ViewMode::List,
                };
                self.scroll_offset = 0;
            },
            KeyCode::Char('c') => { self.requests.clear(); self.selected = 0; },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        match self.view_mode {
            ViewMode::List => "j/k:nav enter:detail c:clear q:quit".into(),
            _ => "j/k:scroll tab:switch-view esc:back q:quit".into(),
        }
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
