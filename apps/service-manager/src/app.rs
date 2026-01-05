//! Application state and logic.

use crate::config::Config;
use crate::services::{self, Service, ServiceStatus};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

pub struct App {
    pub config: Config,
    pub services: Vec<Service>,
    pub selected_index: usize,
    pub filter: Filter,
    pub search: String,
    pub searching: bool,
    pub message: Option<String>,
    pub show_help: bool,
    last_refresh: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Running,
    Stopped,
    Failed,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let services = services::list_services();
        Ok(Self {
            config,
            services,
            selected_index: 0,
            filter: Filter::All,
            search: String::new(),
            searching: false,
            message: None,
            show_help: false,
            last_refresh: Instant::now(),
        })
    }

    pub fn can_quit(&self) -> bool { !self.searching }

    pub fn refresh_if_needed(&mut self) {
        if self.last_refresh.elapsed().as_secs() >= 5 {
            self.services = services::list_services();
            self.last_refresh = Instant::now();
        }
    }

    pub fn filtered_services(&self) -> Vec<&Service> {
        self.services.iter()
            .filter(|s| match self.filter {
                Filter::All => true,
                Filter::Running => s.status == ServiceStatus::Running,
                Filter::Stopped => s.status == ServiceStatus::Stopped,
                Filter::Failed => s.status == ServiceStatus::Failed,
            })
            .filter(|s| self.search.is_empty() || s.name.to_lowercase().contains(&self.search.to_lowercase()))
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.message = None;

        if self.show_help { self.show_help = false; return; }

        if self.searching {
            match key.code {
                KeyCode::Esc => { self.searching = false; self.search.clear(); }
                KeyCode::Enter => { self.searching = false; }
                KeyCode::Backspace => { self.search.pop(); }
                KeyCode::Char(c) => { self.search.push(c); }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('1') => self.filter = Filter::All,
            KeyCode::Char('2') => self.filter = Filter::Running,
            KeyCode::Char('3') => self.filter = Filter::Stopped,
            KeyCode::Char('4') => self.filter = Filter::Failed,
            KeyCode::Char('/') => { self.searching = true; self.search.clear(); }
            KeyCode::Char('s') => self.start_selected(),
            KeyCode::Char('S') => self.stop_selected(),
            KeyCode::Char('r') => self.restart_selected(),
            KeyCode::Char('R') => { self.services = services::list_services(); self.message = Some("Refreshed".into()); }
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.filtered_services().len();
        if len == 0 { return; }
        let new_idx = (self.selected_index as i32 + delta).clamp(0, len as i32 - 1) as usize;
        self.selected_index = new_idx;
    }

    fn start_selected(&mut self) {
        if let Some(svc) = self.filtered_services().get(self.selected_index) {
            match services::start_service(&svc.name) {
                Ok(_) => self.message = Some(format!("Started {}", svc.name)),
                Err(e) => self.message = Some(e),
            }
        }
    }

    fn stop_selected(&mut self) {
        if let Some(svc) = self.filtered_services().get(self.selected_index) {
            match services::stop_service(&svc.name) {
                Ok(_) => self.message = Some(format!("Stopped {}", svc.name)),
                Err(e) => self.message = Some(e),
            }
        }
    }

    fn restart_selected(&mut self) {
        if let Some(svc) = self.filtered_services().get(self.selected_index) {
            match services::restart_service(&svc.name) {
                Ok(_) => self.message = Some(format!("Restarted {}", svc.name)),
                Err(e) => self.message = Some(e),
            }
        }
    }
}
