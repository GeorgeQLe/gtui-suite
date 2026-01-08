use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Streams,
    Keys,
    PubSub,
}

impl View {
    pub fn all() -> &'static [View] {
        &[View::Overview, View::Streams, View::Keys, View::PubSub]
    }

    pub fn name(&self) -> &'static str {
        match self {
            View::Overview => "Overview",
            View::Streams => "Streams",
            View::Keys => "Keys",
            View::PubSub => "Pub/Sub",
        }
    }

    pub fn next(&self) -> View {
        let views = Self::all();
        let idx = views.iter().position(|v| v == self).unwrap_or(0);
        views[(idx + 1) % views.len()]
    }

    pub fn prev(&self) -> View {
        let views = Self::all();
        let idx = views.iter().position(|v| v == self).unwrap_or(0);
        views[(idx + views.len() - 1) % views.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    StreamDetail,
    KeyDetail,
    AddEntry,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,
    pub auto_refresh: bool,

    // Connection state
    pub connected: bool,
    pub error: Option<String>,

    // Data
    pub info: RedisInfo,
    pub streams: Vec<Stream>,
    pub keys: Vec<RedisKey>,
    pub channels: Vec<PubSubChannel>,

    // UI state
    pub selected: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub key_pattern: String,
    pub filtered_indices: Vec<usize>,

    // Detail views
    pub selected_stream: Option<String>,
    pub stream_entries: Vec<StreamEntry>,
    pub selected_key: Option<String>,
    pub key_value: Option<String>,

    // Add entry dialog
    pub add_stream: String,
    pub add_fields: Vec<(String, String)>,
    pub add_field_idx: usize,
}

impl App {
    pub fn new(config: Config) -> Self {
        let key_pattern = config.display.default_key_pattern.clone();

        Self {
            config,
            view: View::Overview,
            input_mode: InputMode::Normal,
            auto_refresh: true,
            connected: false,
            error: None,
            info: RedisInfo::default(),
            streams: Vec::new(),
            keys: Vec::new(),
            channels: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            search_query: String::new(),
            key_pattern,
            filtered_indices: Vec::new(),
            selected_stream: None,
            stream_entries: Vec::new(),
            selected_key: None,
            key_value: None,
            add_stream: String::new(),
            add_fields: vec![(String::new(), String::new())],
            add_field_idx: 0,
        }
    }

    pub async fn refresh(&mut self) {
        // Simulated data - in production, would use redis crate
        self.connected = true;
        self.error = None;

        // Demo info
        self.info = RedisInfo {
            version: "7.2.0".to_string(),
            connected_clients: 5,
            used_memory: 1024 * 1024 * 50,
            used_memory_human: "50MB".to_string(),
            total_commands_processed: 1_234_567,
            uptime_in_seconds: 86400 * 3 + 3600 * 5,
            keyspace_hits: 95000,
            keyspace_misses: 5000,
        };

        // Demo streams
        self.streams = vec![
            Stream {
                name: "orders:stream".to_string(),
                length: 15234,
                first_entry_id: Some("1700000000000-0".to_string()),
                last_entry_id: Some("1700001000000-0".to_string()),
                groups: vec![ConsumerGroup {
                    name: "order-processor".to_string(),
                    pending: 42,
                    last_delivered_id: "1700000999000-0".to_string(),
                    consumers: vec![Consumer {
                        name: "processor-1".to_string(),
                        pending: 20,
                        idle: Duration::from_secs(5),
                    }],
                }],
            },
            Stream {
                name: "events:stream".to_string(),
                length: 5000,
                first_entry_id: Some("1700000000000-0".to_string()),
                last_entry_id: Some("1700000500000-0".to_string()),
                groups: vec![],
            },
        ];

        // Demo keys
        self.keys = vec![
            RedisKey {
                name: "user:1001".to_string(),
                key_type: KeyType::Hash,
                ttl: None,
                memory: Some(256),
            },
            RedisKey {
                name: "session:abc123".to_string(),
                key_type: KeyType::String,
                ttl: Some(3600),
                memory: Some(128),
            },
            RedisKey {
                name: "cache:products".to_string(),
                key_type: KeyType::ZSet,
                ttl: Some(300),
                memory: Some(1024),
            },
            RedisKey {
                name: "queue:jobs".to_string(),
                key_type: KeyType::List,
                ttl: None,
                memory: Some(512),
            },
        ];

        // Demo pub/sub
        self.channels = vec![
            PubSubChannel {
                name: "notifications".to_string(),
                subscribers: 3,
                pattern: false,
            },
            PubSubChannel {
                name: "events.*".to_string(),
                subscribers: 2,
                pattern: true,
            },
        ];

        self.update_filter();
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::StreamDetail => self.handle_stream_detail_key(key),
            InputMode::KeyDetail => self.handle_key_detail_key(key),
            InputMode::AddEntry => self.handle_add_entry_key(key).await,
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c') if is_ctrl => return true,
            KeyCode::Char('q') => return true,

            KeyCode::Tab => {
                self.view = self.view.next();
                self.selected = 0;
                self.scroll_offset = 0;
            }
            KeyCode::BackTab => {
                self.view = self.view.prev();
                self.selected = 0;
                self.scroll_offset = 0;
            }

            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => self.selected = self.item_count().saturating_sub(1),

            KeyCode::Char('r') => self.refresh().await,
            KeyCode::Char('R') => self.auto_refresh = !self.auto_refresh,

            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }

            KeyCode::Char('s') => {
                self.view = View::Streams;
                self.selected = 0;
            }
            KeyCode::Char('p') => {
                self.view = View::PubSub;
                self.selected = 0;
            }

            KeyCode::Enter => match self.view {
                View::Streams => {
                    if let Some(stream) = self.selected_item(&self.streams) {
                        self.selected_stream = Some(stream.name.clone());
                        self.load_stream_entries().await;
                        self.input_mode = InputMode::StreamDetail;
                    }
                }
                View::Keys => {
                    if let Some(redis_key) = self.selected_item(&self.keys) {
                        self.selected_key = Some(redis_key.name.clone());
                        self.load_key_value().await;
                        self.input_mode = InputMode::KeyDetail;
                    }
                }
                _ => {}
            },

            KeyCode::Char('a') => {
                if self.view == View::Streams {
                    self.input_mode = InputMode::AddEntry;
                    self.add_stream.clear();
                    self.add_fields = vec![(String::new(), String::new())];
                    self.add_field_idx = 0;
                }
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
                self.update_filter();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.update_filter();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_filter();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_filter();
            }
            _ => {}
        }
        false
    }

    fn handle_stream_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.selected_stream = None;
                self.stream_entries.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected = (self.selected + 1).min(self.stream_entries.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            _ => {}
        }
        false
    }

    fn handle_key_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.selected_key = None;
                self.key_value = None;
            }
            _ => {}
        }
        false
    }

    async fn handle_add_entry_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                self.add_field_idx = (self.add_field_idx + 1) % (self.add_fields.len() * 2 + 1);
            }
            KeyCode::Enter => {
                // Would add entry to stream
                self.input_mode = InputMode::Normal;
                self.refresh().await;
            }
            KeyCode::Backspace => {
                if self.add_field_idx == 0 {
                    self.add_stream.pop();
                } else {
                    let field_idx = (self.add_field_idx - 1) / 2;
                    let is_key = (self.add_field_idx - 1) % 2 == 0;
                    if let Some(field) = self.add_fields.get_mut(field_idx) {
                        if is_key {
                            field.0.pop();
                        } else {
                            field.1.pop();
                        }
                    }
                }
            }
            KeyCode::Char(c) => {
                if self.add_field_idx == 0 {
                    self.add_stream.push(c);
                } else {
                    let field_idx = (self.add_field_idx - 1) / 2;
                    let is_key = (self.add_field_idx - 1) % 2 == 0;
                    if let Some(field) = self.add_fields.get_mut(field_idx) {
                        if is_key {
                            field.0.push(c);
                        } else {
                            field.1.push(c);
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    async fn load_stream_entries(&mut self) {
        // Demo entries
        self.stream_entries = vec![
            StreamEntry {
                id: "1700001000000-0".to_string(),
                fields: [("type".to_string(), "order".to_string()), ("id".to_string(), "12345".to_string())]
                    .into_iter()
                    .collect(),
            },
            StreamEntry {
                id: "1700000999000-0".to_string(),
                fields: [("type".to_string(), "order".to_string()), ("id".to_string(), "12344".to_string())]
                    .into_iter()
                    .collect(),
            },
        ];
        self.selected = 0;
    }

    async fn load_key_value(&mut self) {
        // Demo value
        self.key_value = Some("{\"name\": \"John\", \"email\": \"john@example.com\"}".to_string());
    }

    fn move_selection(&mut self, delta: i32) {
        let count = self.item_count();
        if count == 0 {
            return;
        }

        let new_selected = if delta > 0 {
            (self.selected + delta as usize).min(count - 1)
        } else {
            self.selected.saturating_sub((-delta) as usize)
        };

        self.selected = new_selected;
    }

    fn item_count(&self) -> usize {
        if !self.search_query.is_empty() {
            return self.filtered_indices.len();
        }

        match self.view {
            View::Overview => 0,
            View::Streams => self.streams.len(),
            View::Keys => self.keys.len(),
            View::PubSub => self.channels.len(),
        }
    }

    fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices.clear();
            return;
        }

        let query = self.search_query.to_lowercase();
        self.filtered_indices = match self.view {
            View::Overview => vec![],
            View::Streams => self
                .streams
                .iter()
                .enumerate()
                .filter(|(_, s)| s.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
            View::Keys => self
                .keys
                .iter()
                .enumerate()
                .filter(|(_, k)| k.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
            View::PubSub => self
                .channels
                .iter()
                .enumerate()
                .filter(|(_, c)| c.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
        };

        self.selected = 0;
    }

    fn selected_item<'a, T>(&self, items: &'a [T]) -> Option<&'a T> {
        let idx = if self.search_query.is_empty() {
            self.selected
        } else {
            *self.filtered_indices.get(self.selected)?
        };
        items.get(idx)
    }

    pub fn status_text(&self) -> String {
        if !self.connected {
            "Disconnected".to_string()
        } else if self.auto_refresh {
            format!(
                "Connected | v{} | {} clients | Auto-refresh ON",
                self.info.version, self.info.connected_clients
            )
        } else {
            format!(
                "Connected | v{} | {} clients | Auto-refresh OFF",
                self.info.version, self.info.connected_clients
            )
        }
    }
}
