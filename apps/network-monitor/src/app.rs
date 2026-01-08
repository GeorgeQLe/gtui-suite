use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

use crate::connections::{Connection, ConnectionState, Protocol, get_connections};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Connections,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Protocol,
    LocalAddr,
    RemoteAddr,
    State,
    Pid,
}

pub struct App {
    pub view: View,
    pub connections: Vec<Connection>,
    pub selected: usize,
    pub scroll: usize,

    // Filters
    pub filter_protocol: Option<Protocol>,
    pub filter_state: Option<ConnectionState>,
    pub show_listening: bool,

    // Sort
    pub sort_by: SortBy,
    pub sort_ascending: bool,

    // Refresh
    pub auto_refresh: bool,
    pub refresh_interval_secs: u64,
    pub last_refresh: Instant,

    pub message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Connections,
            connections: Vec::new(),
            selected: 0,
            scroll: 0,
            filter_protocol: None,
            filter_state: None,
            show_listening: true,
            sort_by: SortBy::Protocol,
            sort_ascending: true,
            auto_refresh: true,
            refresh_interval_secs: 2,
            last_refresh: Instant::now(),
            message: None,
        }
    }

    pub fn refresh(&mut self) {
        self.connections = get_connections();
        self.apply_filters();
        self.apply_sort();
        self.last_refresh = Instant::now();

        if self.selected >= self.connections.len() && !self.connections.is_empty() {
            self.selected = self.connections.len() - 1;
        }
    }

    pub fn tick(&mut self) {
        if self.last_refresh.elapsed().as_secs() >= self.refresh_interval_secs {
            self.refresh();
        }
    }

    fn apply_filters(&mut self) {
        self.connections.retain(|conn| {
            if let Some(proto) = self.filter_protocol {
                if conn.protocol != proto {
                    return false;
                }
            }

            if let Some(state) = &self.filter_state {
                if &conn.state != state {
                    return false;
                }
            }

            if !self.show_listening && conn.state == ConnectionState::Listen {
                return false;
            }

            true
        });
    }

    fn apply_sort(&mut self) {
        self.connections.sort_by(|a, b| {
            let cmp = match self.sort_by {
                SortBy::Protocol => a.protocol.cmp(&b.protocol),
                SortBy::LocalAddr => a.local_addr.cmp(&b.local_addr),
                SortBy::RemoteAddr => a.remote_addr.cmp(&b.remote_addr),
                SortBy::State => a.state.cmp(&b.state),
                SortBy::Pid => a.pid.cmp(&b.pid),
            };

            if self.sort_ascending { cmp } else { cmp.reverse() }
        });
    }

    pub fn filtered_count(&self) -> usize {
        self.connections.len()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;

        match self.view {
            View::Connections => self.handle_connections_key(key),
            View::Help => self.handle_help_key(key),
        }
    }

    fn handle_connections_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < self.connections.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::PageDown => {
                self.selected = (self.selected + 20).min(self.connections.len().saturating_sub(1));
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(20);
            }
            KeyCode::Home | KeyCode::Char('g') => self.selected = 0,
            KeyCode::End | KeyCode::Char('G') => {
                self.selected = self.connections.len().saturating_sub(1);
            }

            // Refresh
            KeyCode::Char('r') | KeyCode::F(5) => {
                self.refresh();
                self.message = Some("Refreshed".to_string());
            }

            // Toggle auto-refresh
            KeyCode::Char('a') => {
                self.auto_refresh = !self.auto_refresh;
                self.message = Some(format!("Auto-refresh: {}", if self.auto_refresh { "ON" } else { "OFF" }));
            }

            // Protocol filter
            KeyCode::Char('t') => {
                self.filter_protocol = match self.filter_protocol {
                    None => Some(Protocol::Tcp),
                    Some(Protocol::Tcp) => Some(Protocol::Udp),
                    Some(Protocol::Udp) => None,
                };
                self.refresh();
            }

            // Toggle listening
            KeyCode::Char('l') => {
                self.show_listening = !self.show_listening;
                self.refresh();
            }

            // Sort
            KeyCode::Char('s') => {
                self.sort_by = match self.sort_by {
                    SortBy::Protocol => SortBy::LocalAddr,
                    SortBy::LocalAddr => SortBy::RemoteAddr,
                    SortBy::RemoteAddr => SortBy::State,
                    SortBy::State => SortBy::Pid,
                    SortBy::Pid => SortBy::Protocol,
                };
                self.apply_sort();
            }
            KeyCode::Char('S') => {
                self.sort_ascending = !self.sort_ascending;
                self.apply_sort();
            }

            // State filter
            KeyCode::Char('e') => {
                self.filter_state = match &self.filter_state {
                    None => Some(ConnectionState::Established),
                    Some(ConnectionState::Established) => Some(ConnectionState::Listen),
                    Some(ConnectionState::Listen) => Some(ConnectionState::TimeWait),
                    Some(ConnectionState::TimeWait) => None,
                    Some(_) => None,
                };
                self.refresh();
            }

            _ => {}
        }

        false
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                self.view = View::Connections;
            }
            _ => {}
        }
        false
    }
}
