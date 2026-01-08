use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::client::RabbitClient;
use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Queues,
    Exchanges,
    Bindings,
    Connections,
    Consumers,
}

impl View {
    pub fn all() -> &'static [View] {
        &[
            View::Overview,
            View::Queues,
            View::Exchanges,
            View::Bindings,
            View::Connections,
            View::Consumers,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            View::Overview => "Overview",
            View::Queues => "Queues",
            View::Exchanges => "Exchanges",
            View::Bindings => "Bindings",
            View::Connections => "Connections",
            View::Consumers => "Consumers",
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
    VhostSelect,
    MessageView,
    Confirm,
    Publish,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,
    pub auto_refresh: bool,

    // Connection state
    pub connected: bool,
    pub error: Option<String>,
    pub reconnect_count: u32,

    // Current vhost
    pub vhost: String,
    pub vhosts: Vec<Vhost>,

    // Data
    pub overview: Option<Overview>,
    pub queues: Vec<Queue>,
    pub exchanges: Vec<Exchange>,
    pub bindings: Vec<Binding>,
    pub connections: Vec<Connection>,
    pub consumers: Vec<Consumer>,

    // UI state
    pub selected: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,

    // Message viewing
    pub messages: Vec<Message>,
    pub message_selected: usize,

    // Confirm dialog
    pub confirm_action: Option<ConfirmAction>,
    pub confirm_input: String,

    // Publish dialog
    pub publish_exchange: String,
    pub publish_routing_key: String,
    pub publish_payload: String,
    pub publish_field: usize,

    client: Option<RabbitClient>,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    PurgeQueue(String),
    DeleteQueue(String),
}

impl App {
    pub fn new(config: Config) -> Self {
        let vhost = config.rabbitmq.default_vhost.clone();
        let client = RabbitClient::new(&config.rabbitmq).ok();

        Self {
            config,
            view: View::Overview,
            input_mode: InputMode::Normal,
            auto_refresh: true,
            connected: false,
            error: None,
            reconnect_count: 0,
            vhost,
            vhosts: Vec::new(),
            overview: None,
            queues: Vec::new(),
            exchanges: Vec::new(),
            bindings: Vec::new(),
            connections: Vec::new(),
            consumers: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            messages: Vec::new(),
            message_selected: 0,
            confirm_action: None,
            confirm_input: String::new(),
            publish_exchange: String::new(),
            publish_routing_key: String::new(),
            publish_payload: String::new(),
            publish_field: 0,
            client,
        }
    }

    pub async fn refresh(&mut self) {
        let Some(client) = &self.client else {
            self.error = Some("No client configured".to_string());
            return;
        };

        // Fetch overview and vhosts
        match client.get_overview().await {
            Ok(overview) => {
                self.overview = Some(overview);
                self.connected = true;
                self.error = None;
                self.reconnect_count = 0;
            }
            Err(e) => {
                self.connected = false;
                self.error = Some(e.to_string());
                self.reconnect_count += 1;
                return;
            }
        }

        // Fetch vhosts
        if let Ok(vhosts) = client.get_vhosts().await {
            self.vhosts = vhosts;
        }

        // Fetch view-specific data
        match self.view {
            View::Overview => {}
            View::Queues => {
                if let Ok(queues) = client.get_queues(&self.vhost).await {
                    self.queues = queues;
                    self.update_filter();
                }
            }
            View::Exchanges => {
                if let Ok(exchanges) = client.get_exchanges(&self.vhost).await {
                    self.exchanges = exchanges;
                    self.update_filter();
                }
            }
            View::Bindings => {
                if let Ok(bindings) = client.get_bindings(&self.vhost).await {
                    self.bindings = bindings;
                    self.update_filter();
                }
            }
            View::Connections => {
                if let Ok(connections) = client.get_connections().await {
                    self.connections = connections;
                    self.update_filter();
                }
            }
            View::Consumers => {
                if let Ok(consumers) = client.get_consumers().await {
                    self.consumers = consumers;
                    self.update_filter();
                }
            }
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::VhostSelect => self.handle_vhost_key(key),
            InputMode::MessageView => self.handle_message_key(key),
            InputMode::Confirm => self.handle_confirm_key(key).await,
            InputMode::Publish => self.handle_publish_key(key).await,
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
                self.refresh().await;
            }
            KeyCode::BackTab => {
                self.view = self.view.prev();
                self.selected = 0;
                self.scroll_offset = 0;
                self.refresh().await;
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

            KeyCode::Char('v') => {
                self.input_mode = InputMode::VhostSelect;
            }

            KeyCode::Enter => {
                if self.view == View::Queues {
                    self.view_queue_messages().await;
                }
            }

            KeyCode::Char('p') => {
                self.input_mode = InputMode::Publish;
                self.publish_exchange.clear();
                self.publish_routing_key.clear();
                self.publish_payload.clear();
                self.publish_field = 0;
            }

            KeyCode::Char('P') => {
                if self.view == View::Queues {
                    if let Some(queue) = self.selected_queue() {
                        self.confirm_action = Some(ConfirmAction::PurgeQueue(queue.name.clone()));
                        self.confirm_input.clear();
                        self.input_mode = InputMode::Confirm;
                    }
                }
            }

            KeyCode::Char('d') => {
                if self.view == View::Queues {
                    if let Some(queue) = self.selected_queue() {
                        self.confirm_action = Some(ConfirmAction::DeleteQueue(queue.name.clone()));
                        self.confirm_input.clear();
                        self.input_mode = InputMode::Confirm;
                    }
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

    fn handle_vhost_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected = (self.selected + 1).min(self.vhosts.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(vhost) = self.vhosts.get(self.selected) {
                    self.vhost = vhost.name.clone();
                    self.input_mode = InputMode::Normal;
                    self.selected = 0;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_message_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.messages.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.message_selected =
                    (self.message_selected + 1).min(self.messages.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.message_selected = self.message_selected.saturating_sub(1);
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
                if let Some(action) = &self.confirm_action {
                    let expected = match action {
                        ConfirmAction::PurgeQueue(name) | ConfirmAction::DeleteQueue(name) => name,
                    };
                    if self.confirm_input == *expected {
                        self.execute_confirm_action().await;
                    }
                }
                self.input_mode = InputMode::Normal;
                self.confirm_action = None;
            }
            KeyCode::Char(c) => {
                self.confirm_input.push(c);
            }
            _ => {}
        }
        false
    }

    async fn handle_publish_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                self.publish_field = (self.publish_field + 1) % 3;
            }
            KeyCode::BackTab => {
                self.publish_field = (self.publish_field + 2) % 3;
            }
            KeyCode::Enter => {
                if self.publish_field == 2 {
                    self.do_publish().await;
                    self.input_mode = InputMode::Normal;
                } else {
                    self.publish_field += 1;
                }
            }
            KeyCode::Backspace => {
                let field = self.current_publish_field_mut();
                field.pop();
            }
            KeyCode::Char(c) => {
                let field = self.current_publish_field_mut();
                field.push(c);
            }
            _ => {}
        }
        false
    }

    fn current_publish_field_mut(&mut self) -> &mut String {
        match self.publish_field {
            0 => &mut self.publish_exchange,
            1 => &mut self.publish_routing_key,
            _ => &mut self.publish_payload,
        }
    }

    async fn execute_confirm_action(&mut self) {
        let Some(client) = &self.client else { return };

        if let Some(action) = self.confirm_action.take() {
            let result = match action {
                ConfirmAction::PurgeQueue(name) => client.purge_queue(&self.vhost, &name).await,
                ConfirmAction::DeleteQueue(name) => client.delete_queue(&self.vhost, &name).await,
            };

            if let Err(e) = result {
                self.error = Some(e.to_string());
            } else {
                self.refresh().await;
            }
        }
    }

    async fn view_queue_messages(&mut self) {
        let Some(client) = &self.client else { return };

        if let Some(queue) = self.selected_queue() {
            let queue_name = queue.name.clone();
            match client
                .get_messages(
                    &self.vhost,
                    &queue_name,
                    self.config.display.max_messages_preview,
                    "ack_requeue_true",
                )
                .await
            {
                Ok(messages) => {
                    self.messages = messages;
                    self.message_selected = 0;
                    self.input_mode = InputMode::MessageView;
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                }
            }
        }
    }

    async fn do_publish(&mut self) {
        let Some(client) = &self.client else { return };

        let exchange = if self.publish_exchange.is_empty() {
            "amq.default"
        } else {
            &self.publish_exchange
        };

        match client
            .publish_message(
                &self.vhost,
                exchange,
                &self.publish_routing_key,
                &self.publish_payload,
            )
            .await
        {
            Ok(()) => {
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
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
            View::Queues => self.queues.len(),
            View::Exchanges => self.exchanges.len(),
            View::Bindings => self.bindings.len(),
            View::Connections => self.connections.len(),
            View::Consumers => self.consumers.len(),
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
            View::Queues => self
                .queues
                .iter()
                .enumerate()
                .filter(|(_, q)| q.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
            View::Exchanges => self
                .exchanges
                .iter()
                .enumerate()
                .filter(|(_, e)| e.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
            View::Bindings => self
                .bindings
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    b.source.to_lowercase().contains(&query)
                        || b.destination.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect(),
            View::Connections => self
                .connections
                .iter()
                .enumerate()
                .filter(|(_, c)| c.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
            View::Consumers => self
                .consumers
                .iter()
                .enumerate()
                .filter(|(_, c)| c.consumer_tag.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect(),
        };

        self.selected = 0;
    }

    fn selected_queue(&self) -> Option<&Queue> {
        let idx = if self.search_query.is_empty() {
            self.selected
        } else {
            *self.filtered_indices.get(self.selected)?
        };
        self.queues.get(idx)
    }

    pub fn status_text(&self) -> String {
        if !self.connected {
            if self.reconnect_count > 0 {
                format!("Disconnected (retry #{})", self.reconnect_count)
            } else {
                "Disconnected".to_string()
            }
        } else if self.auto_refresh {
            format!("Connected | vhost: {} | Auto-refresh ON", self.vhost)
        } else {
            format!("Connected | vhost: {} | Auto-refresh OFF", self.vhost)
        }
    }
}
