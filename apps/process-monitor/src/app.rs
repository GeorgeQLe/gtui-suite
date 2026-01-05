//! Application state and logic.

use crate::config::Config;
use crate::process::{Process, ProcessCollector};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub config: Config,
    pub processes: Vec<Process>,
    pub selected_index: usize,
    pub view: View,
    pub sort: SortBy,
    pub sort_ascending: bool,
    pub search: String,
    pub searching: bool,
    pub filter_user: Option<String>,
    pub paused: bool,
    pub show_help: bool,
    pub show_detail: bool,
    pub show_io: bool,
    pub show_namespaces: bool,
    pub confirm_kill: Option<i32>,
    collector: ProcessCollector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Tree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Pid,
    User,
    Cpu,
    Memory,
    State,
    Name,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        let mut collector = ProcessCollector::new();
        let processes = collector.collect();

        let view = match config.display.default_view.as_str() {
            "tree" => View::Tree,
            _ => View::List,
        };

        let sort = match config.display.default_sort.as_str() {
            "pid" => SortBy::Pid,
            "mem" | "memory" => SortBy::Memory,
            "name" => SortBy::Name,
            "state" => SortBy::State,
            "user" => SortBy::User,
            _ => SortBy::Cpu,
        };

        Self {
            config,
            processes,
            selected_index: 0,
            view,
            sort,
            sort_ascending: false,
            search: String::new(),
            searching: false,
            filter_user: None,
            paused: false,
            show_help: false,
            show_detail: false,
            show_io: false,
            show_namespaces: false,
            confirm_kill: None,
            collector,
        }
    }

    pub fn can_quit(&self) -> bool {
        !self.searching && self.confirm_kill.is_none()
    }

    pub fn refresh_processes(&mut self) {
        self.processes = self.collector.collect();
        self.sort_processes();
    }

    pub fn filtered_processes(&self) -> Vec<&Process> {
        self.processes
            .iter()
            .filter(|p| {
                // User filter
                if let Some(ref user) = self.filter_user {
                    if &p.user != user {
                        return false;
                    }
                }
                // Search filter
                if !self.search.is_empty() {
                    let term = self.search.to_lowercase();
                    if !p.name.to_lowercase().contains(&term)
                        && !p.cmdline.to_lowercase().contains(&term)
                        && !p.pid.to_string().contains(&term)
                    {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    fn sort_processes(&mut self) {
        self.processes.sort_by(|a, b| {
            let cmp = match self.sort {
                SortBy::Pid => a.pid.cmp(&b.pid),
                SortBy::User => a.user.cmp(&b.user),
                SortBy::Cpu => a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal),
                SortBy::Memory => a.memory_rss.cmp(&b.memory_rss),
                SortBy::State => a.state.label().cmp(b.state.label()),
                SortBy::Name => a.name.cmp(&b.name),
            };
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }

    pub fn tree_processes(&self) -> Vec<(usize, &Process)> {
        let filtered = self.filtered_processes();
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Build tree starting from root processes (ppid=0 or 1)
        fn add_children<'a>(
            pid: i32,
            depth: usize,
            processes: &[&'a Process],
            result: &mut Vec<(usize, &'a Process)>,
            visited: &mut std::collections::HashSet<i32>,
        ) {
            for proc in processes {
                if proc.ppid == pid && !visited.contains(&proc.pid) {
                    visited.insert(proc.pid);
                    result.push((depth, *proc));
                    add_children(proc.pid, depth + 1, processes, result, visited);
                }
            }
        }

        // Start with init/systemd (pid 1) or orphans
        for proc in &filtered {
            if (proc.ppid == 0 || proc.ppid == 1 || !filtered.iter().any(|p| p.pid == proc.ppid))
                && !visited.contains(&proc.pid)
            {
                visited.insert(proc.pid);
                result.push((0, *proc));
                add_children(proc.pid, 1, &filtered, &mut result, &mut visited);
            }
        }

        result
    }

    pub fn selected_process(&self) -> Option<&Process> {
        let filtered = self.filtered_processes();
        filtered.get(self.selected_index).copied()
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Handle kill confirmation
        if let Some(pid) = self.confirm_kill {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.kill_process(pid);
                    self.confirm_kill = None;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm_kill = None;
                }
                _ => {}
            }
            return;
        }

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.show_detail {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    self.show_detail = false;
                }
                _ => {}
            }
            return;
        }

        if self.searching {
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    self.search.clear();
                }
                KeyCode::Enter => {
                    self.searching = false;
                }
                KeyCode::Backspace => {
                    self.search.pop();
                }
                KeyCode::Char(c) => {
                    self.search.push(c);
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('J') | KeyCode::PageDown => self.move_selection(10),
            KeyCode::Char('K') | KeyCode::PageUp => self.move_selection(-10),
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
                self.selected_index = 0;
            }
            KeyCode::Char('G') => {
                let len = self.filtered_processes().len();
                if len > 0 {
                    self.selected_index = len - 1;
                }
            }
            KeyCode::Char('t') => {
                self.view = match self.view {
                    View::List => View::Tree,
                    View::Tree => View::List,
                };
            }
            KeyCode::Char('/') => {
                self.searching = true;
                self.search.clear();
            }
            KeyCode::Char('s') => {
                self.sort = match self.sort {
                    SortBy::Cpu => SortBy::Memory,
                    SortBy::Memory => SortBy::Pid,
                    SortBy::Pid => SortBy::Name,
                    SortBy::Name => SortBy::User,
                    SortBy::User => SortBy::State,
                    SortBy::State => SortBy::Cpu,
                };
                self.sort_processes();
            }
            KeyCode::Char('S') => {
                self.sort_ascending = !self.sort_ascending;
                self.sort_processes();
            }
            KeyCode::Char('u') => {
                if let Some(proc) = self.selected_process() {
                    if self.filter_user.as_ref() == Some(&proc.user) {
                        self.filter_user = None;
                    } else {
                        self.filter_user = Some(proc.user.clone());
                    }
                }
            }
            KeyCode::Char('9') => {
                // SIGKILL - needs confirmation
                if let Some(proc) = self.selected_process() {
                    self.confirm_kill = Some(proc.pid);
                }
            }
            KeyCode::Char('T') => {
                // SIGTERM - no confirmation needed
                if let Some(proc) = self.selected_process() {
                    self.term_process(proc.pid);
                }
            }
            KeyCode::Char('i') => {
                self.show_io = !self.show_io;
            }
            KeyCode::Char('n') => {
                self.show_namespaces = !self.show_namespaces;
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
            }
            KeyCode::Enter => {
                self.show_detail = true;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.filtered_processes().len();
        if len == 0 {
            return;
        }
        let new_idx = (self.selected_index as i32 + delta).clamp(0, len as i32 - 1) as usize;
        self.selected_index = new_idx;
    }

    fn kill_process(&self, pid: i32) {
        #[cfg(unix)]
        unsafe {
            libc::kill(pid, 9); // SIGKILL
        }
        #[cfg(not(unix))]
        let _ = pid;
    }

    fn term_process(&self, pid: i32) {
        #[cfg(unix)]
        unsafe {
            libc::kill(pid, 15); // SIGTERM
        }
        #[cfg(not(unix))]
        let _ = pid;
    }

    pub fn sort_label(&self) -> &'static str {
        match self.sort {
            SortBy::Pid => "PID",
            SortBy::User => "User",
            SortBy::Cpu => "CPU%",
            SortBy::Memory => "Memory",
            SortBy::State => "State",
            SortBy::Name => "Name",
        }
    }
}
