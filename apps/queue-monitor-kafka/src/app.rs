use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Topics,
    Partitions,
    ConsumerGroups,
    Lag,
    Brokers,
}

impl View {
    pub fn all() -> &'static [View] {
        &[
            View::Topics,
            View::Partitions,
            View::ConsumerGroups,
            View::Lag,
            View::Brokers,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            View::Topics => "Topics",
            View::Partitions => "Partitions",
            View::ConsumerGroups => "Consumer Groups",
            View::Lag => "Lag",
            View::Brokers => "Brokers",
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
    ClusterSelect,
    TopicDetail,
    GroupDetail,
    Confirm,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,
    pub auto_refresh: bool,

    // Connection state
    pub connected: bool,
    pub error: Option<String>,
    pub current_cluster: String,

    // Data
    pub topics: Vec<Topic>,
    pub partitions: Vec<Partition>,
    pub consumer_groups: Vec<ConsumerGroup>,
    pub lag_data: Vec<ConsumerLag>,
    pub brokers: Vec<Broker>,

    // UI state
    pub selected: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,

    // Detail views
    pub selected_topic: Option<String>,
    pub selected_group: Option<String>,

    // Confirm dialog
    pub confirm_action: Option<ConfirmAction>,
    pub confirm_input: String,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteTopic(String),
    ResetOffsets(String, String), // group, topic
}

impl App {
    pub fn new(config: Config) -> Self {
        let current_cluster = config
            .display
            .default_cluster
            .clone()
            .or_else(|| config.clusters.first().map(|c| c.name.clone()))
            .unwrap_or_else(|| "local".to_string());

        Self {
            config,
            view: View::Topics,
            input_mode: InputMode::Normal,
            auto_refresh: true,
            connected: false,
            error: None,
            current_cluster,
            topics: Vec::new(),
            partitions: Vec::new(),
            consumer_groups: Vec::new(),
            lag_data: Vec::new(),
            brokers: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            selected_topic: None,
            selected_group: None,
            confirm_action: None,
            confirm_input: String::new(),
        }
    }

    pub async fn refresh(&mut self) {
        // Simulated data - in production, would use rdkafka
        self.connected = true;
        self.error = None;

        // Demo topics
        self.topics = vec![
            Topic {
                name: "orders".to_string(),
                partitions: 12,
                replication_factor: 3,
                configs: Default::default(),
                internal: false,
            },
            Topic {
                name: "events".to_string(),
                partitions: 6,
                replication_factor: 3,
                configs: Default::default(),
                internal: false,
            },
            Topic {
                name: "logs".to_string(),
                partitions: 24,
                replication_factor: 2,
                configs: Default::default(),
                internal: false,
            },
            Topic {
                name: "__consumer_offsets".to_string(),
                partitions: 50,
                replication_factor: 3,
                configs: Default::default(),
                internal: true,
            },
        ];

        // Demo consumer groups
        self.consumer_groups = vec![
            ConsumerGroup {
                name: "order-processor".to_string(),
                state: GroupState::Stable,
                members: vec![GroupMember {
                    member_id: "member-1".to_string(),
                    client_id: "processor-1".to_string(),
                    client_host: "10.0.0.1".to_string(),
                    assignments: vec![MemberAssignment {
                        topic: "orders".to_string(),
                        partitions: vec![0, 1, 2, 3],
                    }],
                }],
                protocol_type: "consumer".to_string(),
            },
            ConsumerGroup {
                name: "log-aggregator".to_string(),
                state: GroupState::Stable,
                members: vec![],
                protocol_type: "consumer".to_string(),
            },
        ];

        // Demo lag data
        self.lag_data = vec![
            ConsumerLag::new("order-processor", "orders", 0, 9500, 10000),
            ConsumerLag::new("order-processor", "orders", 1, 9800, 10000),
            ConsumerLag::new("log-aggregator", "logs", 0, 5000, 15000),
        ];

        // Demo brokers
        self.brokers = vec![
            Broker {
                id: 1,
                host: "kafka-1".to_string(),
                port: 9092,
                rack: Some("rack-a".to_string()),
                is_controller: true,
            },
            Broker {
                id: 2,
                host: "kafka-2".to_string(),
                port: 9092,
                rack: Some("rack-b".to_string()),
                is_controller: false,
            },
            Broker {
                id: 3,
                host: "kafka-3".to_string(),
                port: 9092,
                rack: Some("rack-c".to_string()),
                is_controller: false,
            },
        ];

        self.update_filter();
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::ClusterSelect => self.handle_cluster_key(key),
            InputMode::TopicDetail => self.handle_topic_detail_key(key),
            InputMode::GroupDetail => self.handle_group_detail_key(key),
            InputMode::Confirm => self.handle_confirm_key(key).await,
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

            KeyCode::Enter => match self.view {
                View::Topics => {
                    if let Some(topic) = self.selected_item(&self.topics) {
                        self.selected_topic = Some(topic.name.clone());
                        self.input_mode = InputMode::TopicDetail;
                    }
                }
                View::ConsumerGroups => {
                    if let Some(group) = self.selected_item(&self.consumer_groups) {
                        self.selected_group = Some(group.name.clone());
                        self.input_mode = InputMode::GroupDetail;
                    }
                }
                _ => {}
            },

            KeyCode::Char('d') => {
                if self.view == View::Topics {
                    if let Some(topic) = self.selected_item(&self.topics) {
                        if !topic.internal {
                            self.confirm_action =
                                Some(ConfirmAction::DeleteTopic(topic.name.clone()));
                            self.confirm_input.clear();
                            self.input_mode = InputMode::Confirm;
                        }
                    }
                }
            }

            KeyCode::Char('l') => {
                self.view = View::Lag;
                self.selected = 0;
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

    fn handle_cluster_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected = (self.selected + 1).min(self.config.clusters.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(cluster) = self.config.clusters.get(self.selected) {
                    self.current_cluster = cluster.name.clone();
                    self.input_mode = InputMode::Normal;
                    self.selected = 0;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_topic_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.selected_topic = None;
            }
            _ => {}
        }
        false
    }

    fn handle_group_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.selected_group = None;
            }
            _ => {}
        }
        false
    }

    async fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.confirm_action = None;
            }
            KeyCode::Backspace => {
                self.confirm_input.pop();
            }
            KeyCode::Enter => {
                // In production, would execute the action
                self.input_mode = InputMode::Normal;
                self.confirm_action = None;
                self.refresh().await;
            }
            KeyCode::Char(c) => {
                self.confirm_input.push(c);
            }
            _ => {}
        }
        false
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
            View::Topics => self.topics.len(),
            View::Partitions => self.partitions.len(),
            View::ConsumerGroups => self.consumer_groups.len(),
            View::Lag => self.lag_data.len(),
            View::Brokers => self.brokers.len(),
        }
    }

    fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices.clear();
            return;
        }

        let query = self.search_query.to_lowercase();
        self.filtered_indices = match self.view {
            View::Topics => self
                .topics
                .iter()
                .enumerate()
                .filter(|(_, t)| t.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
            View::ConsumerGroups => self
                .consumer_groups
                .iter()
                .enumerate()
                .filter(|(_, g)| g.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
            View::Lag => self
                .lag_data
                .iter()
                .enumerate()
                .filter(|(_, l)| {
                    l.group.to_lowercase().contains(&query)
                        || l.topic.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect(),
            _ => vec![],
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
            format!("Connected to {} | Auto-refresh ON", self.current_cluster)
        } else {
            format!("Connected to {} | Auto-refresh OFF", self.current_cluster)
        }
    }

    pub fn total_lag(&self) -> i64 {
        self.lag_data.iter().map(|l| l.lag).sum()
    }
}
