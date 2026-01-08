#![allow(dead_code)]

use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};
use uuid::Uuid;

use crate::config::Config;
use crate::database::Database;
use crate::http_client;
use crate::request::{Collection, Header, HistoryEntry, Response, SavedRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Request,
    Response,
    Collections,
    History,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestSection {
    Method,
    Url,
    Headers,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    EditUrl(String),
    EditHeader(usize, String), // header index, editing key or value
    EditBody(String),
    AddHeader(String, String), // key, value
    SaveRequest(String),
    NewCollection(String),
}

pub struct App {
    pub db: Database,
    pub config: Config,
    pub view: View,
    pub mode: Mode,

    // Current request
    pub current_request: SavedRequest,
    pub section: RequestSection,
    pub header_index: usize,

    // Response
    pub response: Option<Response>,
    pub response_scroll: usize,

    // Collections
    pub collections: Vec<Collection>,
    pub requests: Vec<SavedRequest>,
    pub collection_index: usize,
    pub request_index: usize,

    // History
    pub history: Vec<HistoryEntry>,
    pub history_index: usize,

    // State
    pub is_loading: bool,
    pub message: Option<String>,
    pub error: Option<String>,

    // Async runtime handle
    pub rt: tokio::runtime::Runtime,
}

impl App {
    pub fn new(db: Database, config: Config) -> Self {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        Self {
            db,
            config,
            view: View::Request,
            mode: Mode::Normal,
            current_request: SavedRequest::new("New Request".to_string()),
            section: RequestSection::Url,
            header_index: 0,
            response: None,
            response_scroll: 0,
            collections: Vec::new(),
            requests: Vec::new(),
            collection_index: 0,
            request_index: 0,
            history: Vec::new(),
            history_index: 0,
            is_loading: false,
            message: None,
            error: None,
            rt,
        }
    }

    pub fn load_data(&mut self) {
        if let Ok(collections) = self.db.list_collections() {
            self.collections = collections;
        }
        if let Ok(requests) = self.db.list_requests() {
            self.requests = requests;
        }
        if let Ok(history) = self.db.list_history(50) {
            self.history = history;
        }
    }

    pub fn send_request(&mut self) {
        self.is_loading = true;
        self.error = None;
        self.message = None;

        let request = self.current_request.clone();

        match self.rt.block_on(http_client::send_request(&request)) {
            Ok(response) => {
                // Add to history
                let history_entry = HistoryEntry {
                    id: Uuid::new_v4(),
                    request_id: None,
                    method: request.method,
                    url: request.url.clone(),
                    status: response.status,
                    duration_ms: response.duration_ms,
                    timestamp: Utc::now(),
                };
                let _ = self.db.add_history(&history_entry);
                self.history.insert(0, history_entry);

                self.message = Some(format!(
                    "{} {} - {}ms",
                    response.status, response.status_text, response.duration_ms
                ));
                self.response = Some(response);
                self.response_scroll = 0;
                self.view = View::Response;
            }
            Err(e) => {
                self.error = Some(format!("Request failed: {}", e));
            }
        }

        self.is_loading = false;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match &self.mode {
            Mode::Normal => match self.view {
                View::Request => self.handle_request_key(key),
                View::Response => self.handle_response_key(key),
                View::Collections => self.handle_collections_key(key),
                View::History => self.handle_history_key(key),
                View::Help => self.handle_help_key(key),
            },
            _ => self.handle_input_key(key),
        }
    }

    fn handle_request_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Tab => {
                self.section = match self.section {
                    RequestSection::Method => RequestSection::Url,
                    RequestSection::Url => RequestSection::Headers,
                    RequestSection::Headers => RequestSection::Body,
                    RequestSection::Body => RequestSection::Method,
                };
            }
            KeyCode::Enter => match self.section {
                RequestSection::Method => {
                    self.current_request.method = self.current_request.method.next();
                }
                RequestSection::Url => {
                    self.mode = Mode::EditUrl(self.current_request.url.clone());
                }
                RequestSection::Headers => {
                    if self.current_request.headers.is_empty() {
                        self.mode = Mode::AddHeader(String::new(), String::new());
                    } else {
                        self.send_request();
                    }
                }
                RequestSection::Body => {
                    self.mode = Mode::EditBody(self.current_request.body.clone().unwrap_or_default());
                }
            },
            KeyCode::Char('m') => {
                self.current_request.method = self.current_request.method.next();
            }
            KeyCode::Char('u') => {
                self.section = RequestSection::Url;
                self.mode = Mode::EditUrl(self.current_request.url.clone());
            }
            KeyCode::Char('h') => {
                self.section = RequestSection::Headers;
                self.mode = Mode::AddHeader(String::new(), String::new());
            }
            KeyCode::Char('b') => {
                self.section = RequestSection::Body;
                self.mode = Mode::EditBody(self.current_request.body.clone().unwrap_or_default());
            }
            KeyCode::Char('s') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.mode = Mode::SaveRequest(self.current_request.name.clone());
            }
            KeyCode::F(5) | KeyCode::Char('r') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                if !self.current_request.url.is_empty() {
                    self.send_request();
                }
            }
            KeyCode::Char('c') => {
                self.load_data();
                self.view = View::Collections;
            }
            KeyCode::Char('H') => {
                self.load_data();
                self.view = View::History;
            }
            KeyCode::Char('v') => {
                if self.response.is_some() {
                    self.view = View::Response;
                }
            }
            KeyCode::Char('n') => {
                self.current_request = SavedRequest::new("New Request".to_string());
                self.response = None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.section == RequestSection::Headers && !self.current_request.headers.is_empty() {
                    self.header_index = (self.header_index + 1) % self.current_request.headers.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.section == RequestSection::Headers && !self.current_request.headers.is_empty() {
                    self.header_index = self.header_index.checked_sub(1).unwrap_or(self.current_request.headers.len() - 1);
                }
            }
            KeyCode::Char('d') => {
                if self.section == RequestSection::Headers && !self.current_request.headers.is_empty() {
                    self.current_request.headers.remove(self.header_index);
                    if self.header_index > 0 && self.header_index >= self.current_request.headers.len() {
                        self.header_index -= 1;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn handle_response_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Request;
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                self.response_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.response_scroll = self.response_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.response_scroll += 20;
            }
            KeyCode::PageUp => {
                self.response_scroll = self.response_scroll.saturating_sub(20);
            }
            KeyCode::Char('y') => {
                let curl = http_client::generate_curl(&self.current_request);
                self.message = Some(format!("Curl: {}", &curl[..curl.len().min(50)]));
            }
            _ => {}
        }
        false
    }

    fn handle_collections_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Request;
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.request_index < self.requests.len().saturating_sub(1) {
                    self.request_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.request_index > 0 {
                    self.request_index -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(request) = self.requests.get(self.request_index) {
                    self.current_request = request.clone();
                    self.response = None;
                    self.view = View::Request;
                }
            }
            KeyCode::Char('n') => {
                self.mode = Mode::NewCollection(String::new());
            }
            KeyCode::Char('d') => {
                if let Some(request) = self.requests.get(self.request_index) {
                    let id = request.id;
                    if let Err(e) = self.db.delete_request(id) {
                        self.error = Some(format!("Delete failed: {}", e));
                    } else {
                        self.requests.remove(self.request_index);
                        if self.request_index > 0 {
                            self.request_index -= 1;
                        }
                        self.message = Some("Request deleted".to_string());
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn handle_history_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Request;
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.history_index < self.history.len().saturating_sub(1) {
                    self.history_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.history_index > 0 {
                    self.history_index -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(entry) = self.history.get(self.history_index) {
                    self.current_request.method = entry.method;
                    self.current_request.url = entry.url.clone();
                    self.response = None;
                    self.view = View::Request;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                self.view = View::Request;
            }
            _ => {}
        }
        false
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        match mode {
            Mode::EditUrl(mut text) => match key.code {
                KeyCode::Enter => {
                    self.current_request.url = text;
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::EditUrl(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::EditUrl(text);
                }
                _ => self.mode = Mode::EditUrl(text),
            },
            Mode::EditBody(mut text) => match key.code {
                KeyCode::Esc => {
                    self.current_request.body = if text.is_empty() { None } else { Some(text) };
                }
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::EditBody(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::EditBody(text);
                }
                KeyCode::Enter => {
                    text.push('\n');
                    self.mode = Mode::EditBody(text);
                }
                _ => self.mode = Mode::EditBody(text),
            },
            Mode::AddHeader(mut key_text, mut value_text) => match key.code {
                KeyCode::Enter => {
                    if !key_text.is_empty() {
                        self.current_request.headers.push(Header::new(key_text, value_text));
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Tab => {
                    // Switch between key and value
                    self.mode = Mode::AddHeader(key_text, value_text);
                }
                KeyCode::Backspace => {
                    if value_text.is_empty() {
                        key_text.pop();
                    } else {
                        value_text.pop();
                    }
                    self.mode = Mode::AddHeader(key_text, value_text);
                }
                KeyCode::Char(':') if value_text.is_empty() => {
                    // Don't add colon to key, just switch to value
                    self.mode = Mode::AddHeader(key_text, value_text);
                }
                KeyCode::Char(c) => {
                    // Simple logic: if key is reasonable length, add to value
                    if key_text.len() < 40 && value_text.is_empty() && c != ' ' {
                        key_text.push(c);
                    } else {
                        value_text.push(c);
                    }
                    self.mode = Mode::AddHeader(key_text, value_text);
                }
                _ => self.mode = Mode::AddHeader(key_text, value_text),
            },
            Mode::SaveRequest(mut name) => match key.code {
                KeyCode::Enter => {
                    if !name.is_empty() {
                        self.current_request.name = name;
                        self.current_request.updated_at = Utc::now();
                        if let Err(e) = self.db.save_request(&self.current_request) {
                            self.error = Some(format!("Save failed: {}", e));
                        } else {
                            self.message = Some("Request saved".to_string());
                            self.load_data();
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    name.pop();
                    self.mode = Mode::SaveRequest(name);
                }
                KeyCode::Char(c) => {
                    name.push(c);
                    self.mode = Mode::SaveRequest(name);
                }
                _ => self.mode = Mode::SaveRequest(name),
            },
            Mode::NewCollection(mut name) => match key.code {
                KeyCode::Enter => {
                    if !name.is_empty() {
                        let collection = Collection::new(name);
                        if let Err(e) = self.db.save_collection(&collection) {
                            self.error = Some(format!("Save failed: {}", e));
                        } else {
                            self.message = Some("Collection created".to_string());
                            self.load_data();
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    name.pop();
                    self.mode = Mode::NewCollection(name);
                }
                KeyCode::Char(c) => {
                    name.push(c);
                    self.mode = Mode::NewCollection(name);
                }
                _ => self.mode = Mode::NewCollection(name),
            },
            Mode::EditHeader(_, _) | Mode::Normal => {}
        }
        false
    }
}
