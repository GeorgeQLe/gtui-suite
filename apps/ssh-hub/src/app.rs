use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::config::Config;
use crate::database::Database;
use crate::host::{HostProfile, Snippet};
use crate::ssh_config;

/// Current view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Hosts,
    History,
    Snippets,
    Help,
}

impl View {
    pub fn label(&self) -> &'static str {
        match self {
            View::Hosts => "Hosts",
            View::History => "History",
            View::Snippets => "Snippets",
            View::Help => "Help",
        }
    }
}

/// Application mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search(String),
    FilterTag(String),
    AddHost(HostFormState),
    EditHost(HostFormState),
    Confirm(ConfirmAction),
}

/// Host form state for add/edit
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HostFormState {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: String,
    pub identity_file: String,
    pub proxy_jump: String,
    pub tags: String,
    pub notes: String,
    pub field: usize,
}

impl HostFormState {
    pub fn from_host(host: &HostProfile) -> Self {
        Self {
            name: host.name.clone(),
            host: host.host.clone(),
            user: host.user.clone().unwrap_or_default(),
            port: host.port.map(|p| p.to_string()).unwrap_or_default(),
            identity_file: host.identity_file.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
            proxy_jump: host.proxy_jump.clone().unwrap_or_default(),
            tags: host.tags.join(", "),
            notes: host.notes.clone().unwrap_or_default(),
            field: 0,
        }
    }

    pub fn field_count() -> usize { 8 }

    pub fn field_label(idx: usize) -> &'static str {
        match idx {
            0 => "Name",
            1 => "Host",
            2 => "User",
            3 => "Port",
            4 => "Identity File",
            5 => "Proxy Jump",
            6 => "Tags",
            7 => "Notes",
            _ => "",
        }
    }

    pub fn field_value(&self, idx: usize) -> &str {
        match idx {
            0 => &self.name,
            1 => &self.host,
            2 => &self.user,
            3 => &self.port,
            4 => &self.identity_file,
            5 => &self.proxy_jump,
            6 => &self.tags,
            7 => &self.notes,
            _ => "",
        }
    }

    pub fn field_value_mut(&mut self, idx: usize) -> &mut String {
        match idx {
            0 => &mut self.name,
            1 => &mut self.host,
            2 => &mut self.user,
            3 => &mut self.port,
            4 => &mut self.identity_file,
            5 => &mut self.proxy_jump,
            6 => &mut self.tags,
            7 => &mut self.notes,
            _ => &mut self.name,
        }
    }
}

/// Action requiring confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteHost(String),
    Connect(String),
}

/// Application state
pub struct App {
    pub config: Config,
    pub db: Database,
    pub view: View,
    pub mode: Mode,
    pub hosts: Vec<HostProfile>,
    pub filtered_hosts: Vec<usize>,
    pub snippets: Vec<Snippet>,
    pub selected: usize,
    pub snippet_selected: usize,
    pub search_query: String,
    pub tag_filter: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let db = Database::open()?;

        let mut app = Self {
            config,
            db,
            view: View::Hosts,
            mode: Mode::Normal,
            hosts: Vec::new(),
            filtered_hosts: Vec::new(),
            snippets: Vec::new(),
            selected: 0,
            snippet_selected: 0,
            search_query: String::new(),
            tag_filter: None,
            message: None,
            error: None,
            matcher: SkimMatcherV2::default(),
        };

        app.refresh()?;
        Ok(app)
    }

    pub fn refresh(&mut self) -> Result<()> {
        // Load hosts from database
        self.hosts = self.db.list_hosts()?;

        // Optionally import from SSH config
        if self.config.ssh.parse_config {
            let ssh_hosts = ssh_config::parse_ssh_config()?;
            for ssh_host in ssh_hosts {
                // Only add if not already in database
                if !self.hosts.iter().any(|h| h.name == ssh_host.name) {
                    self.db.insert_host(&ssh_host)?;
                    self.hosts.push(ssh_host);
                }
            }
        }

        // Sort hosts by name
        self.hosts.sort_by(|a, b| a.name.cmp(&b.name));

        // Load snippets
        self.snippets = self.db.list_snippets(None)?;

        // Update filtered list
        self.update_filter();

        Ok(())
    }

    fn update_filter(&mut self) {
        self.filtered_hosts = self.hosts.iter()
            .enumerate()
            .filter(|(_, host)| {
                // Apply tag filter
                if let Some(tag) = &self.tag_filter {
                    if !host.tags.iter().any(|t| t.to_lowercase().contains(&tag.to_lowercase())) {
                        return false;
                    }
                }

                // Apply search filter
                if !self.search_query.is_empty() {
                    let searchable = format!("{} {} {}", host.name, host.host, host.user.as_deref().unwrap_or(""));
                    if self.matcher.fuzzy_match(&searchable, &self.search_query).is_none() {
                        return false;
                    }
                }

                true
            })
            .map(|(i, _)| i)
            .collect();

        // Ensure selection is valid
        if self.selected >= self.filtered_hosts.len() {
            self.selected = self.filtered_hosts.len().saturating_sub(1);
        }
    }

    pub fn selected_host(&self) -> Option<&HostProfile> {
        self.filtered_hosts.get(self.selected)
            .and_then(|&idx| self.hosts.get(idx))
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Search(_) => self.handle_search_key(key),
            Mode::FilterTag(_) => self.handle_tag_filter_key(key),
            Mode::AddHost(_) | Mode::EditHost(_) => self.handle_form_key(key),
            Mode::Confirm(_) => self.handle_confirm_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Esc => {
                if self.view == View::Help {
                    self.view = View::Hosts;
                } else {
                    self.search_query.clear();
                    self.tag_filter = None;
                    self.update_filter();
                }
            }

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Home | KeyCode::Char('g') => self.selected = 0,
            KeyCode::End | KeyCode::Char('G') => {
                self.selected = self.filtered_hosts.len().saturating_sub(1);
            }

            // View switching
            KeyCode::Char('h') if self.view == View::Hosts => {
                self.view = View::History;
            }
            KeyCode::Char('s') if self.view == View::Hosts => {
                self.view = View::Snippets;
            }
            KeyCode::Tab => {
                self.view = match self.view {
                    View::Hosts => View::History,
                    View::History => View::Snippets,
                    View::Snippets => View::Hosts,
                    View::Help => View::Hosts,
                };
            }

            // Search
            KeyCode::Char('/') => {
                self.mode = Mode::Search(String::new());
            }

            // Tag filter
            KeyCode::Char('t') => {
                self.mode = Mode::FilterTag(String::new());
            }

            // Host actions
            KeyCode::Enter if self.view == View::Hosts => {
                if let Some(host) = self.selected_host() {
                    self.mode = Mode::Confirm(ConfirmAction::Connect(host.id.clone()));
                }
            }
            KeyCode::Char('a') if self.view == View::Hosts => {
                self.mode = Mode::AddHost(HostFormState::default());
            }
            KeyCode::Char('e') if self.view == View::Hosts => {
                if let Some(host) = self.selected_host() {
                    self.mode = Mode::EditHost(HostFormState::from_host(host));
                }
            }
            KeyCode::Char('d') if self.view == View::Hosts => {
                if let Some(host) = self.selected_host() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteHost(host.id.clone()));
                }
            }

            // Refresh
            KeyCode::F(5) => {
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

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        if let Mode::Search(ref mut query) = self.mode {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.search_query = query.clone();
                    self.update_filter();
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    query.pop();
                }
                KeyCode::Char(c) => {
                    query.push(c);
                }
                _ => {}
            }
        }
        false
    }

    fn handle_tag_filter_key(&mut self, key: KeyEvent) -> bool {
        if let Mode::FilterTag(ref mut tag) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    self.tag_filter = if tag.is_empty() { None } else { Some(tag.clone()) };
                    self.update_filter();
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    tag.pop();
                }
                KeyCode::Char(c) => {
                    tag.push(c);
                }
                _ => {}
            }
        }
        false
    }

    fn handle_form_key(&mut self, key: KeyEvent) -> bool {
        let is_add = matches!(self.mode, Mode::AddHost(_));

        // Check if we need to submit
        let should_submit = match &self.mode {
            Mode::AddHost(form) | Mode::EditHost(form) => {
                key.code == KeyCode::Enter && form.field == HostFormState::field_count() - 1
            }
            _ => false,
        };

        if should_submit {
            let form = match &self.mode {
                Mode::AddHost(f) | Mode::EditHost(f) => f.clone(),
                _ => return false,
            };
            if let Err(e) = self.submit_form(&form, is_add) {
                self.error = Some(format!("Save failed: {}", e));
            } else {
                self.message = Some(if is_add { "Host added" } else { "Host updated" }.to_string());
                let _ = self.refresh();
            }
            self.mode = Mode::Normal;
            return false;
        }

        match &mut self.mode {
            Mode::AddHost(ref mut form) | Mode::EditHost(ref mut form) => {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Enter => {
                        form.field += 1;
                    }
                    KeyCode::Tab | KeyCode::Down => {
                        form.field = (form.field + 1) % HostFormState::field_count();
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        form.field = form.field.checked_sub(1).unwrap_or(HostFormState::field_count() - 1);
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

    fn submit_form(&mut self, form: &HostFormState, is_add: bool) -> Result<()> {
        let mut host = if is_add {
            HostProfile::new(form.name.clone(), form.host.clone())
        } else {
            self.selected_host().cloned().ok_or_else(|| anyhow::anyhow!("No host selected"))?
        };

        host.name = form.name.clone();
        host.host = form.host.clone();
        host.user = if form.user.is_empty() { None } else { Some(form.user.clone()) };
        host.port = form.port.parse().ok();
        host.identity_file = if form.identity_file.is_empty() { None } else { Some(form.identity_file.clone().into()) };
        host.proxy_jump = if form.proxy_jump.is_empty() { None } else { Some(form.proxy_jump.clone()) };
        host.tags = form.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        host.notes = if form.notes.is_empty() { None } else { Some(form.notes.clone()) };

        if is_add {
            self.db.insert_host(&host)?;
        } else {
            self.db.update_host(&host)?;
        }

        Ok(())
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Mode::Confirm(action) = &self.mode {
                    match action {
                        ConfirmAction::DeleteHost(id) => {
                            let id = id.clone();
                            if let Err(e) = self.db.delete_host(&id) {
                                self.error = Some(format!("Delete failed: {}", e));
                            } else {
                                self.message = Some("Host deleted".to_string());
                                let _ = self.refresh();
                            }
                        }
                        ConfirmAction::Connect(id) => {
                            let _ = self.db.update_last_connected(id);
                            self.message = Some("Connecting... (spawning SSH)".to_string());
                            // In a real implementation, we would spawn SSH here
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

    fn move_down(&mut self) {
        match self.view {
            View::Hosts | View::History => {
                if self.selected < self.filtered_hosts.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            View::Snippets => {
                if self.snippet_selected < self.snippets.len().saturating_sub(1) {
                    self.snippet_selected += 1;
                }
            }
            View::Help => {}
        }
    }

    fn move_up(&mut self) {
        match self.view {
            View::Hosts | View::History => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            View::Snippets => {
                if self.snippet_selected > 0 {
                    self.snippet_selected -= 1;
                }
            }
            View::Help => {}
        }
    }

    /// Get all unique tags from hosts
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.hosts.iter()
            .flat_map(|h| h.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
}
