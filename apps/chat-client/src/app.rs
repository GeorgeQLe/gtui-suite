use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
    ServerList,
    ChannelList,
}

pub struct App {
    pub config: Config,
    pub input_mode: InputMode,

    // Servers and channels
    pub servers: Vec<Server>,
    pub active_server: Option<usize>,
    pub active_channel: Option<usize>,

    // UI state
    pub server_list_selected: usize,
    pub channel_list_selected: usize,
    pub user_list_selected: usize,
    pub message_scroll: usize,
    pub show_user_list: bool,

    // Input
    pub input_buffer: String,
    pub input_cursor: usize,
    pub command_buffer: String,

    // History
    pub input_history: Vec<String>,
    pub history_index: Option<usize>,

    // Status
    pub status_message: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let show_user_list = config.display.show_user_list;

        // Create demo servers
        let mut servers = Vec::new();

        // IRC server with channels
        let mut irc = Server::new("Libera Chat", Protocol::Irc, "irc.libera.chat", 6697, "rustuser");
        irc.connected = true;

        let mut rust_channel = Channel::new("rust", false);
        rust_channel.topic = Some("The Rust Programming Language - https://rust-lang.org".to_string());
        rust_channel.users = vec![
            User::new("ferris"),
            User::new("rustacean"),
            User::new("crab_lover"),
        ];
        rust_channel.messages = vec![
            Message::system("You have joined #rust"),
            Message::chat("ferris", "Hello! Welcome to #rust"),
            Message::chat("rustacean", "Anyone tried the new async features?"),
            Message::chat("crab_lover", "Yes! They're great for concurrent programming"),
        ];

        let mut help_channel = Channel::new("rust-beginners", false);
        help_channel.topic = Some("New to Rust? Ask here!".to_string());
        help_channel.users = vec![User::new("helper"), User::new("newbie")];
        help_channel.unread_count = 3;

        irc.channels = vec![rust_channel, help_channel];
        servers.push(irc);

        // Matrix server
        let mut matrix = Server::new("Matrix", Protocol::Matrix, "matrix.org", 443, "@rustuser:matrix.org");
        matrix.connected = false;
        let matrix_room = Channel::new("Rust Community", false);
        matrix.channels = vec![matrix_room];
        servers.push(matrix);

        Self {
            config,
            input_mode: InputMode::Normal,
            servers,
            active_server: Some(0),
            active_channel: Some(0),
            server_list_selected: 0,
            channel_list_selected: 0,
            user_list_selected: 0,
            message_scroll: 0,
            show_user_list,
            input_buffer: String::new(),
            input_cursor: 0,
            command_buffer: String::new(),
            input_history: Vec::new(),
            history_index: None,
            status_message: None,
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Insert => self.handle_insert_key(key).await,
            InputMode::Command => self.handle_command_key(key).await,
            InputMode::ServerList => self.handle_server_list_key(key),
            InputMode::ChannelList => self.handle_channel_list_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') => return true,

            KeyCode::Char('i') => {
                self.input_mode = InputMode::Insert;
            }
            KeyCode::Char(':') => {
                self.input_mode = InputMode::Command;
                self.command_buffer.clear();
            }

            KeyCode::Char('s') => {
                self.input_mode = InputMode::ServerList;
            }
            KeyCode::Char('c') => {
                self.input_mode = InputMode::ChannelList;
            }

            KeyCode::Char('u') => {
                self.show_user_list = !self.show_user_list;
            }

            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_messages(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_messages(-1);
            }
            KeyCode::Char('G') => {
                self.message_scroll = 0;
            }
            KeyCode::Char('g') => {
                if let Some(channel) = self.current_channel() {
                    self.message_scroll = channel.messages.len().saturating_sub(1);
                }
            }

            KeyCode::Tab => {
                self.next_channel();
            }
            KeyCode::BackTab => {
                self.prev_channel();
            }

            _ => {}
        }

        false
    }

    async fn handle_insert_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    self.send_message().await;
                }
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    self.input_buffer.remove(self.input_cursor - 1);
                    self.input_cursor -= 1;
                }
            }
            KeyCode::Delete => {
                if self.input_cursor < self.input_buffer.len() {
                    self.input_buffer.remove(self.input_cursor);
                }
            }
            KeyCode::Left => {
                self.input_cursor = self.input_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                self.input_cursor = (self.input_cursor + 1).min(self.input_buffer.len());
            }
            KeyCode::Home => {
                self.input_cursor = 0;
            }
            KeyCode::End => {
                self.input_cursor = self.input_buffer.len();
            }
            KeyCode::Up => {
                self.history_up();
            }
            KeyCode::Down => {
                self.history_down();
            }
            KeyCode::Char(c) => {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += 1;
            }
            _ => {}
        }

        false
    }

    async fn handle_command_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                self.execute_command().await;
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            _ => {}
        }

        false
    }

    fn handle_server_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.server_list_selected =
                    (self.server_list_selected + 1).min(self.servers.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.server_list_selected = self.server_list_selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.active_server = Some(self.server_list_selected);
                self.active_channel = Some(0);
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }

        false
    }

    fn handle_channel_list_key(&mut self, key: KeyEvent) -> bool {
        let channel_count = self
            .active_server
            .and_then(|i| self.servers.get(i))
            .map(|s| s.channels.len())
            .unwrap_or(0);

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.channel_list_selected =
                    (self.channel_list_selected + 1).min(channel_count.saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.channel_list_selected = self.channel_list_selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.active_channel = Some(self.channel_list_selected);
                self.input_mode = InputMode::Normal;
                self.message_scroll = 0;
            }
            _ => {}
        }

        false
    }

    async fn send_message(&mut self) {
        let content = self.input_buffer.clone();
        self.input_history.push(content.clone());
        self.history_index = None;

        if let Some(channel) = self.current_channel_mut() {
            let sender = "rustuser"; // Would come from server connection
            let msg = Message::chat(sender, &content);
            channel.messages.push(msg);
        }

        self.input_buffer.clear();
        self.input_cursor = 0;
        self.message_scroll = 0;
    }

    async fn execute_command(&mut self) {
        let command_buffer = self.command_buffer.clone();
        let parts: Vec<&str> = command_buffer.split_whitespace().collect();
        if parts.is_empty() {
            self.command_buffer.clear();
            return;
        }

        let cmd = parts[0];
        match cmd {
            "join" | "j" => {
                if let Some(channel_name) = parts.get(1) {
                    let name = channel_name.to_string();
                    self.join_channel(&name).await;
                }
            }
            "part" | "leave" => {
                self.part_channel().await;
            }
            "msg" | "m" => {
                if parts.len() >= 3 {
                    let target = parts[1].to_string();
                    let content = parts[2..].join(" ");
                    self.send_private_message(&target, &content).await;
                }
            }
            "nick" => {
                if let Some(new_nick) = parts.get(1) {
                    self.status_message = Some(format!("Nickname changed to {}", new_nick));
                }
            }
            "quit" | "q" => {
                // Would disconnect from server
            }
            _ => {
                self.status_message = Some(format!("Unknown command: {}", cmd));
            }
        }

        self.command_buffer.clear();
    }

    async fn join_channel(&mut self, name: &str) {
        if let Some(server) = self.active_server.and_then(|i| self.servers.get_mut(i)) {
            let channel = Channel::new(name.trim_start_matches('#'), false);
            server.channels.push(channel);
            self.active_channel = Some(server.channels.len() - 1);
            self.status_message = Some(format!("Joined #{}", name));
        }
    }

    async fn part_channel(&mut self) {
        if let (Some(server_idx), Some(channel_idx)) = (self.active_server, self.active_channel) {
            if let Some(server) = self.servers.get_mut(server_idx) {
                if server.channels.len() > 1 {
                    let channel_name = server.channels[channel_idx].name.clone();
                    server.channels.remove(channel_idx);
                    self.active_channel = Some(0);
                    self.status_message = Some(format!("Left #{}", channel_name));
                }
            }
        }
    }

    async fn send_private_message(&mut self, _target: &str, _content: &str) {
        // Would send private message
    }

    fn scroll_messages(&mut self, delta: i32) {
        if let Some(channel) = self.current_channel() {
            let max_scroll = channel.messages.len().saturating_sub(1);
            if delta > 0 {
                self.message_scroll = (self.message_scroll + delta as usize).min(max_scroll);
            } else {
                self.message_scroll = self.message_scroll.saturating_sub((-delta) as usize);
            }
        }
    }

    fn next_channel(&mut self) {
        if let Some(server) = self.active_server.and_then(|i| self.servers.get(i)) {
            if let Some(current) = self.active_channel {
                self.active_channel = Some((current + 1) % server.channels.len());
                self.message_scroll = 0;
            }
        }
    }

    fn prev_channel(&mut self) {
        if let Some(server) = self.active_server.and_then(|i| self.servers.get(i)) {
            if let Some(current) = self.active_channel {
                self.active_channel = Some(
                    (current + server.channels.len() - 1) % server.channels.len(),
                );
                self.message_scroll = 0;
            }
        }
    }

    fn history_up(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            Some(idx) => idx.saturating_sub(1),
            None => self.input_history.len() - 1,
        };

        self.history_index = Some(new_index);
        self.input_buffer = self.input_history[new_index].clone();
        self.input_cursor = self.input_buffer.len();
    }

    fn history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx + 1 < self.input_history.len() {
                self.history_index = Some(idx + 1);
                self.input_buffer = self.input_history[idx + 1].clone();
            } else {
                self.history_index = None;
                self.input_buffer.clear();
            }
            self.input_cursor = self.input_buffer.len();
        }
    }

    pub fn current_server(&self) -> Option<&Server> {
        self.active_server.and_then(|i| self.servers.get(i))
    }

    pub fn current_channel(&self) -> Option<&Channel> {
        self.current_server()
            .and_then(|s| self.active_channel.and_then(|i| s.channels.get(i)))
    }

    fn current_channel_mut(&mut self) -> Option<&mut Channel> {
        let server_idx = self.active_server?;
        let channel_idx = self.active_channel?;
        self.servers
            .get_mut(server_idx)
            .and_then(|s| s.channels.get_mut(channel_idx))
    }

    pub async fn check_messages(&mut self) {
        // Would check for new messages from connections
        // For demo, this is a no-op
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            return msg.clone();
        }

        match self.input_mode {
            InputMode::Normal => "NORMAL | i:insert  ::command  s:servers  c:channels  u:users  q:quit".to_string(),
            InputMode::Insert => "INSERT | Esc:normal  Enter:send  Up/Down:history".to_string(),
            InputMode::Command => format!(":{}", self.command_buffer),
            InputMode::ServerList => "SERVER LIST | j/k:navigate  Enter:select  Esc:cancel".to_string(),
            InputMode::ChannelList => "CHANNEL LIST | j/k:navigate  Enter:select  Esc:cancel".to_string(),
        }
    }
}
