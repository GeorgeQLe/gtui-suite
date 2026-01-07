use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::container::{Container, Image, Network, Volume};
use crate::docker_client::{DockerClient, Runtime};

/// Current view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Containers,
    Images,
    Volumes,
    Networks,
    Logs,
    Help,
}

impl View {
    pub fn label(&self) -> &'static str {
        match self {
            View::Containers => "Containers",
            View::Images => "Images",
            View::Volumes => "Volumes",
            View::Networks => "Networks",
            View::Logs => "Logs",
            View::Help => "Help",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            View::Containers => View::Images,
            View::Images => View::Volumes,
            View::Volumes => View::Networks,
            View::Networks => View::Containers,
            View::Logs => View::Containers,
            View::Help => View::Containers,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            View::Containers => View::Networks,
            View::Images => View::Containers,
            View::Volumes => View::Images,
            View::Networks => View::Volumes,
            View::Logs => View::Containers,
            View::Help => View::Containers,
        }
    }
}

/// Application mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search(String),
    Confirm(ConfirmAction),
}

/// Action requiring confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    RemoveContainer(String),
    RemoveImage(String),
    StopContainer(String),
}

/// Application state
pub struct App {
    pub config: Config,
    pub client: Option<DockerClient>,
    pub view: View,
    pub mode: Mode,
    pub containers: Vec<Container>,
    pub images: Vec<Image>,
    pub volumes: Vec<Volume>,
    pub networks: Vec<Network>,
    pub container_selected: usize,
    pub image_selected: usize,
    pub volume_selected: usize,
    pub network_selected: usize,
    pub show_all_containers: bool,
    pub logs: Vec<String>,
    pub log_container: Option<String>,
    pub logs_scroll: usize,
    pub message: Option<String>,
    pub error: Option<String>,
    pub log_tx: Option<mpsc::Sender<String>>,
    pub log_rx: Option<mpsc::Receiver<String>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let show_all = config.display.show_all_containers;

        Ok(Self {
            config,
            client: None,
            view: View::Containers,
            mode: Mode::Normal,
            containers: Vec::new(),
            images: Vec::new(),
            volumes: Vec::new(),
            networks: Vec::new(),
            container_selected: 0,
            image_selected: 0,
            volume_selected: 0,
            network_selected: 0,
            show_all_containers: show_all,
            logs: Vec::new(),
            log_container: None,
            logs_scroll: 0,
            message: None,
            error: None,
            log_tx: None,
            log_rx: None,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        let runtime = if self.config.runtime.prefer == "docker" {
            Runtime::Docker
        } else if self.config.runtime.prefer == "podman" {
            Runtime::Podman
        } else {
            // Auto-detect
            DockerClient::detect_runtime().await
                .ok_or_else(|| anyhow::anyhow!("No Docker or Podman runtime found"))?
        };

        self.client = Some(DockerClient::connect(runtime)?);
        self.refresh_all().await?;
        Ok(())
    }

    pub async fn refresh_all(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            self.containers = client.list_containers(self.show_all_containers).await?;
            self.images = client.list_images().await?;
            self.volumes = client.list_volumes().await?;
            self.networks = client.list_networks().await?;

            // Ensure selections are valid
            if self.container_selected >= self.containers.len() {
                self.container_selected = self.containers.len().saturating_sub(1);
            }
            if self.image_selected >= self.images.len() {
                self.image_selected = self.images.len().saturating_sub(1);
            }
            if self.volume_selected >= self.volumes.len() {
                self.volume_selected = self.volumes.len().saturating_sub(1);
            }
            if self.network_selected >= self.networks.len() {
                self.network_selected = self.networks.len().saturating_sub(1);
            }
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Search(_) => self.handle_search_key(key),
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
                if self.view == View::Logs || self.view == View::Help {
                    self.view = View::Containers;
                }
            }
            KeyCode::Tab => self.view = self.view.next(),
            KeyCode::BackTab => self.view = self.view.prev(),

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Home | KeyCode::Char('g') => self.move_to_top(),
            KeyCode::End | KeyCode::Char('G') => self.move_to_bottom(),

            // Container actions
            KeyCode::Char('s') if self.view == View::Containers => {
                if let Some(container) = self.selected_container() {
                    if container.state != crate::container::ContainerState::Running {
                        self.start_container();
                    }
                }
            }
            KeyCode::Char('S') if self.view == View::Containers => {
                if let Some(container) = self.selected_container() {
                    self.mode = Mode::Confirm(ConfirmAction::StopContainer(container.id.clone()));
                }
            }
            KeyCode::Char('r') if self.view == View::Containers => {
                self.restart_container();
            }
            KeyCode::Char('R') if self.view == View::Containers => {
                if let Some(container) = self.selected_container() {
                    self.mode = Mode::Confirm(ConfirmAction::RemoveContainer(container.id.clone()));
                }
            }
            KeyCode::Char('l') if self.view == View::Containers => {
                self.view_logs();
            }

            // Image actions
            KeyCode::Char('R') if self.view == View::Images => {
                if let Some(image) = self.selected_image() {
                    self.mode = Mode::Confirm(ConfirmAction::RemoveImage(image.id.clone()));
                }
            }

            // Toggle all containers
            KeyCode::Char('a') if self.view == View::Containers => {
                self.show_all_containers = !self.show_all_containers;
            }

            // Search
            KeyCode::Char('/') => {
                self.mode = Mode::Search(String::new());
            }

            // Refresh
            KeyCode::F(5) => {
                // Refresh is handled by main loop
            }

            _ => {}
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        if let Mode::Search(ref mut query) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let query = query.clone().to_lowercase();
                    self.search(&query);
                    self.mode = Mode::Normal;
                }
                KeyCode::Esc => {
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

    fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Mode::Confirm(action) = &self.mode {
                    match action {
                        ConfirmAction::RemoveContainer(id) => {
                            let id = id.clone();
                            self.do_remove_container(&id);
                        }
                        ConfirmAction::RemoveImage(id) => {
                            let id = id.clone();
                            self.do_remove_image(&id);
                        }
                        ConfirmAction::StopContainer(id) => {
                            let id = id.clone();
                            self.do_stop_container(&id);
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
            View::Containers => {
                if self.container_selected < self.containers.len().saturating_sub(1) {
                    self.container_selected += 1;
                }
            }
            View::Images => {
                if self.image_selected < self.images.len().saturating_sub(1) {
                    self.image_selected += 1;
                }
            }
            View::Volumes => {
                if self.volume_selected < self.volumes.len().saturating_sub(1) {
                    self.volume_selected += 1;
                }
            }
            View::Networks => {
                if self.network_selected < self.networks.len().saturating_sub(1) {
                    self.network_selected += 1;
                }
            }
            View::Logs => {
                if self.logs_scroll < self.logs.len().saturating_sub(1) {
                    self.logs_scroll += 1;
                }
            }
            View::Help => {}
        }
    }

    fn move_up(&mut self) {
        match self.view {
            View::Containers => {
                if self.container_selected > 0 {
                    self.container_selected -= 1;
                }
            }
            View::Images => {
                if self.image_selected > 0 {
                    self.image_selected -= 1;
                }
            }
            View::Volumes => {
                if self.volume_selected > 0 {
                    self.volume_selected -= 1;
                }
            }
            View::Networks => {
                if self.network_selected > 0 {
                    self.network_selected -= 1;
                }
            }
            View::Logs => {
                if self.logs_scroll > 0 {
                    self.logs_scroll -= 1;
                }
            }
            View::Help => {}
        }
    }

    fn move_to_top(&mut self) {
        match self.view {
            View::Containers => self.container_selected = 0,
            View::Images => self.image_selected = 0,
            View::Volumes => self.volume_selected = 0,
            View::Networks => self.network_selected = 0,
            View::Logs => self.logs_scroll = 0,
            View::Help => {}
        }
    }

    fn move_to_bottom(&mut self) {
        match self.view {
            View::Containers => self.container_selected = self.containers.len().saturating_sub(1),
            View::Images => self.image_selected = self.images.len().saturating_sub(1),
            View::Volumes => self.volume_selected = self.volumes.len().saturating_sub(1),
            View::Networks => self.network_selected = self.networks.len().saturating_sub(1),
            View::Logs => self.logs_scroll = self.logs.len().saturating_sub(1),
            View::Help => {}
        }
    }

    pub fn selected_container(&self) -> Option<&Container> {
        self.containers.get(self.container_selected)
    }

    pub fn selected_image(&self) -> Option<&Image> {
        self.images.get(self.image_selected)
    }

    fn search(&mut self, query: &str) {
        match self.view {
            View::Containers => {
                for (i, c) in self.containers.iter().enumerate() {
                    if c.primary_name().to_lowercase().contains(query) ||
                       c.image.to_lowercase().contains(query) {
                        self.container_selected = i;
                        break;
                    }
                }
            }
            View::Images => {
                for (i, img) in self.images.iter().enumerate() {
                    if img.primary_tag().to_lowercase().contains(query) {
                        self.image_selected = i;
                        break;
                    }
                }
            }
            View::Volumes => {
                for (i, v) in self.volumes.iter().enumerate() {
                    if v.name.to_lowercase().contains(query) {
                        self.volume_selected = i;
                        break;
                    }
                }
            }
            View::Networks => {
                for (i, n) in self.networks.iter().enumerate() {
                    if n.name.to_lowercase().contains(query) {
                        self.network_selected = i;
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn start_container(&mut self) {
        // Will be executed via async command queue
        self.message = Some("Starting container...".to_string());
    }

    fn restart_container(&mut self) {
        self.message = Some("Restarting container...".to_string());
    }

    fn view_logs(&mut self) {
        if let Some(container) = self.selected_container() {
            self.log_container = Some(container.id.clone());
            self.logs.clear();
            self.logs_scroll = 0;
            self.view = View::Logs;
        }
    }

    fn do_stop_container(&mut self, _id: &str) {
        self.message = Some("Stopping container...".to_string());
    }

    fn do_remove_container(&mut self, _id: &str) {
        self.message = Some("Removing container...".to_string());
    }

    fn do_remove_image(&mut self, _id: &str) {
        self.message = Some("Removing image...".to_string());
    }

    pub fn runtime_label(&self) -> &str {
        self.client.as_ref()
            .map(|c| c.runtime.label())
            .unwrap_or("Not Connected")
    }
}
