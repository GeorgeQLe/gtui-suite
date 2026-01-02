//! Application state and logic.

use crate::config::Config;
use crate::db::{Database, DbResult};
use crate::models::{DailySummary, Project, ProjectId, TimeEntry};
use crate::pomodoro::PomodoroTimer;
use chrono::{Duration, NaiveDate, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;
use std::time::Instant;

pub struct App {
    pub db: Database,
    pub config: Config,
    pub view: View,
    pub selected_date: NaiveDate,
    pub projects: Vec<Project>,
    pub entries: Vec<TimeEntry>,
    pub selected_index: usize,
    pub running_entry: Option<TimeEntry>,
    pub pomodoro: PomodoroTimer,
    pub pomodoro_mode: bool,
    pub editing: bool,
    pub input_buffer: String,
    pub input_field: InputField,
    pub message: Option<String>,
    pub show_help: bool,
    pub last_input: Instant,
    pub project_hours: HashMap<ProjectId, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Timer,
    Entries,
    Reports,
    Projects,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    None,
    Description,
    ProjectName,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "time.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;

        let pomodoro = PomodoroTimer::new(config.pomodoro.clone());
        let today = Utc::now().date_naive();

        let mut app = Self {
            db,
            config,
            view: View::Timer,
            selected_date: today,
            projects: Vec::new(),
            entries: Vec::new(),
            selected_index: 0,
            running_entry: None,
            pomodoro,
            pomodoro_mode: false,
            editing: false,
            input_buffer: String::new(),
            input_field: InputField::None,
            message: None,
            show_help: false,
            last_input: Instant::now(),
            project_hours: HashMap::new(),
        };

        app.refresh()?;
        Ok(app)
    }

    pub fn refresh(&mut self) -> DbResult<()> {
        self.projects = self.db.list_projects(false)?;
        self.entries = self.db.get_entries_for_date(self.selected_date)?;
        self.running_entry = self.db.get_running_entry()?;

        self.project_hours.clear();
        for project in &self.projects {
            if let Ok(hours) = self.db.get_project_hours(project.id) {
                self.project_hours.insert(project.id, hours);
            }
        }

        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        }

        Ok(())
    }

    pub fn can_quit(&self) -> bool {
        !self.editing
    }

    pub fn tick(&mut self) {
        // Update pomodoro timer
        if self.pomodoro_mode && self.pomodoro.tick() {
            // Session completed - notify
            self.message = Some(format!("{} complete!", self.pomodoro.session_type().name()));
            // Terminal bell
            print!("\x07");
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.last_input = Instant::now();
        self.message = None;

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.editing {
            self.handle_edit_key(key);
            return;
        }

        match key.code {
            KeyCode::Char('s') => self.toggle_timer(),
            KeyCode::Char('p') => self.toggle_pomodoro(),
            KeyCode::Char('e') if self.view == View::Entries => self.start_edit_description(),
            KeyCode::Char('d') if self.view == View::Entries => self.delete_selected(),
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('h') | KeyCode::Left => self.change_date(-1),
            KeyCode::Char('l') | KeyCode::Right => self.change_date(1),
            KeyCode::Char('t') => {
                self.selected_date = Utc::now().date_naive();
                let _ = self.refresh();
            }
            KeyCode::Char('1') => self.view = View::Timer,
            KeyCode::Char('2') => self.view = View::Entries,
            KeyCode::Char('r') => self.view = View::Reports,
            KeyCode::Char('P') => self.view = View::Projects,
            KeyCode::Char('a') if self.view == View::Projects => {
                self.editing = true;
                self.input_field = InputField::ProjectName;
                self.input_buffer.clear();
            }
            KeyCode::Enter if self.running_entry.is_some() => {
                self.editing = true;
                self.input_field = InputField::Description;
                self.input_buffer = self.running_entry.as_ref()
                    .map(|e| e.description.clone())
                    .unwrap_or_default();
            }
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.editing = false;
                self.input_buffer.clear();
                self.input_field = InputField::None;
            }
            KeyCode::Enter => self.finish_editing(),
            KeyCode::Backspace => { self.input_buffer.pop(); }
            KeyCode::Char(c) => self.input_buffer.push(c),
            _ => {}
        }
    }

    fn finish_editing(&mut self) {
        match self.input_field {
            InputField::Description => {
                if let Some(entry) = &mut self.running_entry {
                    entry.description = self.input_buffer.clone();
                    let _ = self.db.update_entry(entry);
                }
            }
            InputField::ProjectName => {
                if !self.input_buffer.is_empty() {
                    let project = Project::new(&self.input_buffer);
                    if self.db.insert_project(&project).is_ok() {
                        self.message = Some("Project created".to_string());
                        let _ = self.refresh();
                    }
                }
            }
            InputField::None => {}
        }

        self.editing = false;
        self.input_buffer.clear();
        self.input_field = InputField::None;
    }

    fn toggle_timer(&mut self) {
        if let Some(mut entry) = self.running_entry.take() {
            // Stop timer
            entry.stop();
            let _ = self.db.update_entry(&entry);
            self.message = Some(format!("Stopped: {}", entry.format_duration()));
            let _ = self.refresh();
        } else {
            // Start timer
            let entry = TimeEntry::start("");
            if self.db.insert_entry(&entry).is_ok() {
                self.running_entry = Some(entry);
                self.message = Some("Timer started".to_string());
            }
        }
    }

    pub fn stop_timer(&mut self) {
        if let Some(mut entry) = self.running_entry.take() {
            entry.stop();
            let _ = self.db.update_entry(&entry);
        }
    }

    fn toggle_pomodoro(&mut self) {
        self.pomodoro_mode = !self.pomodoro_mode;
        if self.pomodoro_mode {
            self.pomodoro.start();
            self.message = Some("Pomodoro mode enabled".to_string());
        } else {
            self.pomodoro.pause();
            self.message = Some("Pomodoro mode disabled".to_string());
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = match self.view {
            View::Entries => self.entries.len(),
            View::Projects => self.projects.len(),
            _ => 0,
        };

        if len == 0 { return; }

        let new_idx = self.selected_index as i32 + delta;
        self.selected_index = new_idx.clamp(0, len as i32 - 1) as usize;
    }

    fn change_date(&mut self, delta: i64) {
        self.selected_date = self.selected_date + Duration::days(delta);
        let _ = self.refresh();
    }

    fn start_edit_description(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            self.editing = true;
            self.input_field = InputField::Description;
            self.input_buffer = entry.description.clone();
        }
    }

    fn delete_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if self.db.delete_entry(entry.id).is_ok() {
                self.message = Some("Entry deleted".to_string());
                let _ = self.refresh();
            }
        }
    }

    pub fn today_total(&self) -> chrono::Duration {
        self.entries
            .iter()
            .map(|e| e.duration())
            .fold(Duration::zero(), |acc, d| acc + d)
    }

    pub fn format_duration(dur: Duration) -> String {
        let hours = dur.num_hours();
        let mins = dur.num_minutes() % 60;
        let secs = dur.num_seconds() % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }
}
