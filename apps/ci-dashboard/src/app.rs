use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::models::*;
use crate::providers::{create_providers, CIProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    RunDetails,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Filter,
    Confirm,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    RetryRun(String),
    CancelRun(String),
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,

    // Providers
    providers: Vec<Box<dyn CIProvider>>,

    // Data
    pub runs: Vec<Run>,
    pub selected_run: usize,
    pub current_run: Option<Run>,
    pub logs: Option<String>,

    // System status
    pub system_status: Vec<SystemStatus>,

    // UI state
    pub show_sidebar: bool,
    pub search_query: String,
    pub filter_status: Option<Conclusion>,
    pub scroll_offset: usize,

    // Confirm dialog
    pub confirm_action: Option<ConfirmAction>,

    // Status
    pub status_message: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let providers = create_providers(&config);
        let system_status: Vec<SystemStatus> = providers
            .iter()
            .map(|p| SystemStatus::new(p.system()))
            .collect();

        Self {
            config,
            view: View::Overview,
            input_mode: InputMode::Normal,
            providers,
            runs: Vec::new(),
            selected_run: 0,
            current_run: None,
            logs: None,
            system_status,
            show_sidebar: true,
            search_query: String::new(),
            filter_status: None,
            scroll_offset: 0,
            confirm_action: None,
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        self.runs.clear();

        for (idx, provider) in self.providers.iter().enumerate() {
            let repos = match provider.list_repos().await {
                Ok(repos) => repos,
                Err(e) => {
                    if let Some(status) = self.system_status.get_mut(idx) {
                        status.connected = false;
                        status.error = Some(e.to_string());
                    }
                    continue;
                }
            };

            for repo in repos {
                match provider
                    .get_runs(&repo.full_name, self.config.display.max_runs)
                    .await
                {
                    Ok(runs) => {
                        self.runs.extend(runs);
                        if let Some(status) = self.system_status.get_mut(idx) {
                            status.connected = true;
                            status.last_updated = Some(Utc::now());
                            status.error = None;
                        }
                    }
                    Err(e) => {
                        if let Some(status) = self.system_status.get_mut(idx) {
                            status.error = Some(e.to_string());
                        }
                    }
                }
            }
        }

        // Sort runs by start time (newest first)
        self.runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        if self.selected_run >= self.runs.len() {
            self.selected_run = self.runs.len().saturating_sub(1);
        }
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::Filter => self.handle_filter_key(key),
            InputMode::Confirm => self.handle_confirm_key(key).await,
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') => {
                match self.view {
                    View::Logs => {
                        self.view = View::RunDetails;
                        self.logs = None;
                    }
                    View::RunDetails => {
                        self.view = View::Overview;
                        self.current_run = None;
                    }
                    View::Overview => return true,
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
            }
            KeyCode::Char('g') => {
                self.selected_run = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                self.selected_run = self.filtered_runs().len().saturating_sub(1);
            }

            KeyCode::Enter => {
                self.open_details().await;
            }

            KeyCode::Char('l') => {
                if self.view == View::RunDetails {
                    self.view_logs().await;
                }
            }

            KeyCode::Char('r') => {
                if let Some(run) = self.get_selected_run() {
                    self.confirm_action = Some(ConfirmAction::RetryRun(run.id.clone()));
                    self.input_mode = InputMode::Confirm;
                }
            }

            KeyCode::Char('c') => {
                if let Some(run) = self.get_selected_run() {
                    if run.status == RunStatus::InProgress {
                        self.confirm_action = Some(ConfirmAction::CancelRun(run.id.clone()));
                        self.input_mode = InputMode::Confirm;
                    }
                }
            }

            KeyCode::Char('R') => {
                self.refresh().await;
                self.status_message = Some("Refreshed".to_string());
            }

            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }

            KeyCode::Char('f') => {
                self.input_mode = InputMode::Filter;
            }

            KeyCode::Char('F') => {
                self.filter_status = None;
                self.status_message = Some("Filter cleared".to_string());
            }

            KeyCode::Char('b') => {
                self.show_sidebar = !self.show_sidebar;
            }

            KeyCode::Tab => {
                self.view = match self.view {
                    View::Overview => View::RunDetails,
                    View::RunDetails => View::Logs,
                    View::Logs => View::Overview,
                };
            }

            KeyCode::Esc => {
                self.view = View::Overview;
                self.current_run = None;
                self.logs = None;
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
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('s') | KeyCode::Char('1') => {
                self.filter_status = Some(Conclusion::Success);
                self.input_mode = InputMode::Normal;
                self.status_message = Some("Showing successful runs".to_string());
            }
            KeyCode::Char('f') | KeyCode::Char('2') => {
                self.filter_status = Some(Conclusion::Failure);
                self.input_mode = InputMode::Normal;
                self.status_message = Some("Showing failed runs".to_string());
            }
            KeyCode::Char('c') | KeyCode::Char('3') => {
                self.filter_status = Some(Conclusion::Cancelled);
                self.input_mode = InputMode::Normal;
                self.status_message = Some("Showing cancelled runs".to_string());
            }
            KeyCode::Char('a') | KeyCode::Char('0') => {
                self.filter_status = None;
                self.input_mode = InputMode::Normal;
                self.status_message = Some("Showing all runs".to_string());
            }
            _ => {}
        }
        false
    }

    async fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(action) = self.confirm_action.take() {
                    match action {
                        ConfirmAction::RetryRun(run_id) => {
                            self.retry_run(&run_id).await;
                        }
                        ConfirmAction::CancelRun(run_id) => {
                            self.cancel_run(&run_id).await;
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.confirm_action = None;
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        false
    }

    fn navigate_down(&mut self) {
        let max = self.filtered_runs().len().saturating_sub(1);
        if self.selected_run < max {
            self.selected_run += 1;
        }
    }

    fn navigate_up(&mut self) {
        self.selected_run = self.selected_run.saturating_sub(1);
    }

    async fn open_details(&mut self) {
        if let Some(run) = self.get_selected_run() {
            self.current_run = Some(run.clone());
            self.view = View::RunDetails;
        }
    }

    async fn view_logs(&mut self) {
        if let Some(run) = &self.current_run {
            if let Some(job) = run.jobs.first() {
                for provider in &self.providers {
                    if provider.system() == run.system {
                        match provider.get_job_logs(&job.id).await {
                            Ok(logs) => {
                                self.logs = Some(logs);
                                self.view = View::Logs;
                            }
                            Err(e) => {
                                self.status_message = Some(format!("Failed to fetch logs: {}", e));
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    async fn retry_run(&mut self, run_id: &str) {
        if let Some(run) = self.runs.iter().find(|r| r.id == run_id) {
            for provider in &self.providers {
                if provider.system() == run.system {
                    match provider.retry_run(run_id).await {
                        Ok(()) => {
                            self.status_message = Some("Run restarted".to_string());
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to retry: {}", e));
                        }
                    }
                    break;
                }
            }
        }
    }

    async fn cancel_run(&mut self, run_id: &str) {
        if let Some(run) = self.runs.iter().find(|r| r.id == run_id) {
            for provider in &self.providers {
                if provider.system() == run.system {
                    match provider.cancel_run(run_id).await {
                        Ok(()) => {
                            self.status_message = Some("Run cancelled".to_string());
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to cancel: {}", e));
                        }
                    }
                    break;
                }
            }
        }
    }

    pub fn filtered_runs(&self) -> Vec<&Run> {
        self.runs
            .iter()
            .filter(|run| {
                // Filter by status
                if let Some(filter) = &self.filter_status {
                    if run.conclusion.as_ref() != Some(filter) {
                        return false;
                    }
                }

                // Filter by search query
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    let matches = run.repo.to_lowercase().contains(&query)
                        || run.workflow_name.to_lowercase().contains(&query)
                        || run.branch.to_lowercase().contains(&query);
                    if !matches {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    fn get_selected_run(&self) -> Option<&Run> {
        self.filtered_runs().get(self.selected_run).copied()
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Overview => {
                let total = self.runs.len();
                let failed = self
                    .runs
                    .iter()
                    .filter(|r| r.conclusion == Some(Conclusion::Failure))
                    .count();
                let running = self
                    .runs
                    .iter()
                    .filter(|r| r.status == RunStatus::InProgress)
                    .count();
                format!(
                    "{} runs ({} failed, {} running) | j/k:nav Enter:details r:retry R:refresh q:quit",
                    total, failed, running
                )
            }
            View::RunDetails => "l:logs r:retry c:cancel q:back".to_string(),
            View::Logs => "j/k:scroll q:back".to_string(),
        }
    }
}
