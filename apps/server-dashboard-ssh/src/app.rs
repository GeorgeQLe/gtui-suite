use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;

use crate::config::Config;
use crate::database::Database;
use crate::server::{Server, ServerMetrics, ServerStatus};

/// Current view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    ServerDetail,
    Help,
}

/// Application mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AddServer(ServerFormState),
    EditServer(ServerFormState),
    Confirm(ConfirmAction),
    FilterTag(String),
}

/// Server form state
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerFormState {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: String,
    pub tags: String,
    pub field: usize,
}

impl ServerFormState {
    pub fn from_server(server: &Server) -> Self {
        Self {
            name: server.name.clone(),
            host: server.host.clone(),
            user: server.user.clone().unwrap_or_default(),
            port: server.port.to_string(),
            tags: server.tags.join(", "),
            field: 0,
        }
    }

    pub fn field_count() -> usize { 5 }

    pub fn field_label(idx: usize) -> &'static str {
        match idx {
            0 => "Name",
            1 => "Host",
            2 => "User",
            3 => "Port",
            4 => "Tags (comma-separated)",
            _ => "",
        }
    }

    pub fn field_value(&self, idx: usize) -> &str {
        match idx {
            0 => &self.name,
            1 => &self.host,
            2 => &self.user,
            3 => &self.port,
            4 => &self.tags,
            _ => "",
        }
    }

    pub fn field_value_mut(&mut self, idx: usize) -> &mut String {
        match idx {
            0 => &mut self.name,
            1 => &mut self.host,
            2 => &mut self.user,
            3 => &mut self.port,
            4 => &mut self.tags,
            _ => &mut self.name,
        }
    }
}

/// Action requiring confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteServer(String),
}

/// Application state
pub struct App {
    pub config: Config,
    pub db: Database,
    pub view: View,
    pub mode: Mode,
    pub servers: Vec<Server>,
    pub metrics: HashMap<String, ServerMetrics>,
    pub selected: usize,
    pub paused: bool,
    pub show_graphs: bool,
    pub tag_filter: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let db = Database::open()?;
        let show_graphs = config.display.show_graphs;

        let mut app = Self {
            config,
            db,
            view: View::Dashboard,
            mode: Mode::Normal,
            servers: Vec::new(),
            metrics: HashMap::new(),
            selected: 0,
            paused: false,
            show_graphs,
            tag_filter: None,
            message: None,
            error: None,
        };

        app.refresh()?;
        Ok(app)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.servers = self.db.list_servers()?;

        // Load latest metrics for each server
        for server in &self.servers {
            if let Ok(Some(m)) = self.db.get_latest_metrics(&server.id) {
                self.metrics.insert(server.id.clone(), m);
            }
        }

        if self.selected >= self.servers.len() {
            self.selected = self.servers.len().saturating_sub(1);
        }

        Ok(())
    }

    pub fn filtered_servers(&self) -> Vec<&Server> {
        self.servers.iter()
            .filter(|s| {
                if let Some(tag) = &self.tag_filter {
                    s.tags.iter().any(|t| t.to_lowercase().contains(&tag.to_lowercase()))
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn selected_server(&self) -> Option<&Server> {
        let filtered = self.filtered_servers();
        filtered.get(self.selected).copied()
    }

    pub fn get_metrics(&self, server_id: &str) -> Option<&ServerMetrics> {
        self.metrics.get(server_id)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::AddServer(_) | Mode::EditServer(_) => self.handle_form_key(key),
            Mode::Confirm(_) => self.handle_confirm_key(key),
            Mode::FilterTag(_) => self.handle_filter_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Esc => {
                if self.view != View::Dashboard {
                    self.view = View::Dashboard;
                } else {
                    self.tag_filter = None;
                }
            }

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                let count = self.filtered_servers().len();
                if self.selected < count.saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => self.selected = 0,
            KeyCode::End | KeyCode::Char('G') => {
                self.selected = self.filtered_servers().len().saturating_sub(1);
            }

            // View details
            KeyCode::Enter => {
                if self.view == View::Dashboard && self.selected_server().is_some() {
                    self.view = View::ServerDetail;
                }
            }

            // Server actions
            KeyCode::Char('a') => {
                self.mode = Mode::AddServer(ServerFormState::default());
            }
            KeyCode::Char('e') => {
                if let Some(server) = self.selected_server() {
                    self.mode = Mode::EditServer(ServerFormState::from_server(server));
                }
            }
            KeyCode::Char('d') => {
                if let Some(server) = self.selected_server() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteServer(server.id.clone()));
                }
            }

            // Display toggles - use 'o' for graph toggle since 'G' is used for End
            KeyCode::Char('o') => {
                self.show_graphs = !self.show_graphs;
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
                self.message = Some(if self.paused { "Paused" } else { "Resumed" }.to_string());
            }

            // Filter
            KeyCode::Char('t') => {
                self.mode = Mode::FilterTag(String::new());
            }

            // Refresh
            KeyCode::Char('r') | KeyCode::F(5) => {
                if let Err(e) = self.refresh() {
                    self.error = Some(format!("Refresh failed: {}", e));
                } else {
                    self.message = Some("Refreshed".to_string());
                }
            }

            _ => {}
        }

        false
    }

    fn handle_form_key(&mut self, key: KeyEvent) -> bool {
        let is_add = matches!(self.mode, Mode::AddServer(_));

        let should_submit = match &self.mode {
            Mode::AddServer(form) | Mode::EditServer(form) => {
                key.code == KeyCode::Enter && form.field == ServerFormState::field_count() - 1
            }
            _ => false,
        };

        if should_submit {
            let form = match &self.mode {
                Mode::AddServer(f) | Mode::EditServer(f) => f.clone(),
                _ => return false,
            };
            if let Err(e) = self.submit_form(&form, is_add) {
                self.error = Some(format!("Save failed: {}", e));
            } else {
                self.message = Some(if is_add { "Server added" } else { "Server updated" }.to_string());
                let _ = self.refresh();
            }
            self.mode = Mode::Normal;
            return false;
        }

        match &mut self.mode {
            Mode::AddServer(ref mut form) | Mode::EditServer(ref mut form) => {
                match key.code {
                    KeyCode::Esc => self.mode = Mode::Normal,
                    KeyCode::Enter | KeyCode::Tab | KeyCode::Down => {
                        form.field = (form.field + 1) % ServerFormState::field_count();
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        form.field = form.field.checked_sub(1).unwrap_or(ServerFormState::field_count() - 1);
                    }
                    KeyCode::Backspace => {
                        form.field_value_mut(form.field).pop();
                    }
                    KeyCode::Char(c) => {
                        form.field_value_mut(form.field).push(c);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }

    fn submit_form(&mut self, form: &ServerFormState, is_add: bool) -> Result<()> {
        let mut server = if is_add {
            Server::new(form.name.clone(), form.host.clone())
        } else {
            self.selected_server().cloned().ok_or_else(|| anyhow::anyhow!("No server selected"))?
        };

        server.name = form.name.clone();
        server.host = form.host.clone();
        server.user = if form.user.is_empty() { None } else { Some(form.user.clone()) };
        server.port = form.port.parse().unwrap_or(22);
        server.tags = form.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

        if is_add {
            self.db.insert_server(&server)?;
        } else {
            self.db.update_server(&server)?;
        }

        Ok(())
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Mode::Confirm(action) = &self.mode {
                    match action {
                        ConfirmAction::DeleteServer(id) => {
                            let id = id.clone();
                            if let Err(e) = self.db.delete_server(&id) {
                                self.error = Some(format!("Delete failed: {}", e));
                            } else {
                                self.message = Some("Server deleted".to_string());
                                let _ = self.refresh();
                            }
                        }
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        false
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> bool {
        if let Mode::FilterTag(ref mut tag) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    self.tag_filter = if tag.is_empty() { None } else { Some(tag.clone()) };
                    self.selected = 0;
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Backspace => { tag.pop(); }
                KeyCode::Char(c) => { tag.push(c); }
                _ => {}
            }
        }
        false
    }

    pub fn overall_status(&self) -> ServerStatus {
        let mut has_warning = false;
        let mut has_critical = false;

        for server in &self.servers {
            if let Some(m) = self.metrics.get(&server.id) {
                match m.status {
                    ServerStatus::Critical | ServerStatus::Unreachable => has_critical = true,
                    ServerStatus::Warning => has_warning = true,
                    _ => {}
                }
            }
        }

        if has_critical {
            ServerStatus::Critical
        } else if has_warning {
            ServerStatus::Warning
        } else {
            ServerStatus::Ok
        }
    }
}
