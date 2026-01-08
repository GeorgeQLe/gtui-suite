use chrono::{DateTime, Local, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::config::Config;

pub type AppId = u32;

#[derive(Debug, Clone)]
pub struct RunningApp {
    pub id: AppId,
    pub name: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
}

impl RunningApp {
    pub fn new(id: AppId, name: &str, title: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            title: title.to_string(),
            started_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    App,
    Switcher,
}

pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub running_apps: Vec<RunningApp>,
    pub active_app: Option<AppId>,
    pub recent_order: Vec<AppId>,
    pub next_app_id: AppId,

    // Switcher state
    pub switcher_query: String,
    pub switcher_selected: usize,
    pub switcher_results: Vec<SwitcherResult>,

    pub show_status_bar: bool,
    pub message: Option<String>,
    pub last_switch_time: Option<std::time::Instant>,
}

#[derive(Debug, Clone)]
pub struct SwitcherResult {
    pub app_id: Option<AppId>,
    pub name: String,
    pub title: String,
    pub is_running: bool,
    pub score: i64,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut app = Self {
            show_status_bar: config.general.show_status_bar,
            config,
            mode: Mode::App,
            running_apps: Vec::new(),
            active_app: None,
            recent_order: Vec::new(),
            next_app_id: 1,
            switcher_query: String::new(),
            switcher_selected: 0,
            switcher_results: Vec::new(),
            message: None,
            last_switch_time: None,
        };

        // Start with a default app
        app.launch_app("task-manager", "Task Manager");

        app
    }

    fn next_id(&mut self) -> AppId {
        let id = self.next_app_id;
        self.next_app_id += 1;
        id
    }

    pub fn current_app(&self) -> Option<&RunningApp> {
        self.active_app.and_then(|id| self.running_apps.iter().find(|a| a.id == id))
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;

        match self.mode {
            Mode::App => self.handle_app_key(key),
            Mode::Switcher => self.handle_switcher_key(key),
        }
    }

    fn handle_app_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            // Exit
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('c') if is_ctrl => return true,

            // Open switcher
            KeyCode::Char(' ') if is_ctrl => {
                // Double-tap to switch to last app
                if self.config.general.double_tap_switch {
                    if let Some(last_time) = self.last_switch_time {
                        if last_time.elapsed().as_millis() < 300 {
                            self.switch_to_last_app();
                            self.last_switch_time = None;
                            return false;
                        }
                    }
                    self.last_switch_time = Some(std::time::Instant::now());
                }

                self.open_switcher();
            }

            // Toggle status bar
            KeyCode::Char('b') if is_ctrl => {
                self.show_status_bar = !self.show_status_bar;
            }

            // Close current app
            KeyCode::Char('w') if is_ctrl => {
                self.close_current_app();
            }

            // Quick slots (Ctrl+1-9)
            KeyCode::Char(c @ '1'..='9') if is_ctrl && !is_shift => {
                let slot = (c as usize) - ('1' as usize);
                self.activate_quick_slot(slot);
            }

            _ => {}
        }

        false
    }

    fn handle_switcher_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::App;
            }
            KeyCode::Enter => {
                self.select_switcher_result();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.switcher_selected > 0 {
                    self.switcher_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.switcher_selected < self.switcher_results.len().saturating_sub(1) {
                    self.switcher_selected += 1;
                }
            }
            KeyCode::Char('x') => {
                // Close selected running app
                if let Some(result) = self.switcher_results.get(self.switcher_selected) {
                    if let Some(app_id) = result.app_id {
                        self.close_app(app_id);
                        self.update_switcher_results();
                    }
                }
            }
            KeyCode::Backspace => {
                self.switcher_query.pop();
                self.update_switcher_results();
            }
            KeyCode::Char(c) => {
                self.switcher_query.push(c);
                self.update_switcher_results();
            }
            _ => {}
        }
        false
    }

    fn open_switcher(&mut self) {
        self.mode = Mode::Switcher;
        self.switcher_query.clear();
        self.switcher_selected = 0;
        self.update_switcher_results();
    }

    fn update_switcher_results(&mut self) {
        let matcher = SkimMatcherV2::default();
        let mut results = Vec::new();

        // Available apps to launch
        let available_apps = [
            ("task-manager", "Task Manager"),
            ("note-manager", "Note Manager"),
            ("file-manager", "File Manager"),
            ("time-tracker", "Time Tracker"),
            ("habit-tracker", "Habit Tracker"),
            ("log-viewer", "Log Viewer"),
            ("docker-manager", "Docker Manager"),
            ("git-client", "Git Client"),
            ("hex-editor", "Hex Editor"),
            ("csv-viewer", "CSV Viewer"),
        ];

        // Add running apps
        for app in &self.running_apps {
            let score = if self.switcher_query.is_empty() {
                100
            } else {
                matcher.fuzzy_match(&app.name, &self.switcher_query).unwrap_or(0)
            };

            if score > 0 || self.switcher_query.is_empty() {
                results.push(SwitcherResult {
                    app_id: Some(app.id),
                    name: app.name.clone(),
                    title: app.title.clone(),
                    is_running: true,
                    score,
                });
            }
        }

        // Add available apps (not running)
        for (name, title) in available_apps {
            if self.running_apps.iter().any(|a| a.name == name) {
                continue;
            }

            let score = if self.switcher_query.is_empty() {
                50
            } else {
                matcher.fuzzy_match(name, &self.switcher_query).unwrap_or(0)
            };

            if score > 0 || self.switcher_query.is_empty() {
                results.push(SwitcherResult {
                    app_id: None,
                    name: name.to_string(),
                    title: title.to_string(),
                    is_running: false,
                    score,
                });
            }
        }

        // Sort by: running apps first (if recent_first), then by score
        if self.config.switcher.recent_first {
            results.sort_by(|a, b| {
                match (a.is_running, b.is_running) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => b.score.cmp(&a.score),
                }
            });
        } else {
            results.sort_by(|a, b| b.score.cmp(&a.score));
        }

        // Limit results
        results.truncate(self.config.switcher.max_results);

        self.switcher_results = results;
        self.switcher_selected = self.switcher_selected.min(
            self.switcher_results.len().saturating_sub(1)
        );
    }

    fn select_switcher_result(&mut self) {
        if let Some(result) = self.switcher_results.get(self.switcher_selected).cloned() {
            if let Some(app_id) = result.app_id {
                // Switch to running app
                self.switch_to_app(app_id);
            } else {
                // Launch new app
                self.launch_app(&result.name, &result.title);
            }
        }
        self.mode = Mode::App;
    }

    fn launch_app(&mut self, name: &str, title: &str) {
        let id = self.next_id();
        let app = RunningApp::new(id, name, title);
        self.running_apps.push(app);
        self.switch_to_app(id);
    }

    fn switch_to_app(&mut self, app_id: AppId) {
        if self.running_apps.iter().any(|a| a.id == app_id) {
            // Update recent order
            self.recent_order.retain(|&id| id != app_id);
            if let Some(current) = self.active_app {
                if !self.recent_order.contains(&current) {
                    self.recent_order.insert(0, current);
                }
            }

            self.active_app = Some(app_id);
        }
    }

    fn switch_to_last_app(&mut self) {
        if let Some(&last_id) = self.recent_order.first() {
            self.switch_to_app(last_id);
        }
    }

    fn close_app(&mut self, app_id: AppId) {
        self.running_apps.retain(|a| a.id != app_id);
        self.recent_order.retain(|&id| id != app_id);

        if self.active_app == Some(app_id) {
            self.active_app = self.recent_order.first().copied()
                .or_else(|| self.running_apps.first().map(|a| a.id));
        }
    }

    fn close_current_app(&mut self) {
        if let Some(app_id) = self.active_app {
            self.close_app(app_id);
        }
    }

    fn activate_quick_slot(&mut self, slot: usize) {
        let name = match self.config.quick_slots.slots.get(slot) {
            Some(Some(n)) => n.clone(),
            _ => return,
        };

        // Check if already running
        if let Some(app_id) = self.running_apps.iter().find(|a| a.name == name).map(|a| a.id) {
            self.switch_to_app(app_id);
        } else {
            // Launch it
            let title = name.replace('-', " ");
            let title = title
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            self.launch_app(&name, &title);
        }
    }

    pub fn current_time(&self) -> String {
        Local::now().format(&self.config.status_bar.clock_format).to_string()
    }
}
