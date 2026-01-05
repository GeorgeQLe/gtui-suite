//! Application state and logic.

use crate::config::Config;
use crate::db::Database;
use crate::models::{RunStatus, Schedule, ScheduledTask, TaskRun};
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};

pub struct App {
    pub db: Database,
    pub config: Config,
    pub tasks: Vec<ScheduledTask>,
    pub selected_index: usize,
    pub current_task: Option<ScheduledTask>,
    pub task_runs: Vec<TaskRun>,
    pub mode: Mode,
    pub pane: Pane,
    pub input_buffer: String,
    pub input_mode: InputMode,
    pub input_step: usize,
    pub new_task: NewTaskBuilder,
    pub message: Option<String>,
    pub show_help: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NewTaskBuilder {
    pub name: String,
    pub command: String,
    pub schedule_type: usize,
    pub interval_mins: u32,
    pub daily_hour: u32,
    pub daily_minute: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Creating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Tasks,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    TaskName,
    TaskCommand,
    ScheduleType,
    ScheduleValue,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "scheduler.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;
        let tasks = db.list_tasks()?;

        Ok(Self {
            db,
            config,
            tasks,
            selected_index: 0,
            current_task: None,
            task_runs: Vec::new(),
            mode: Mode::Normal,
            pane: Pane::Tasks,
            input_buffer: String::new(),
            input_mode: InputMode::None,
            input_step: 0,
            new_task: NewTaskBuilder::default(),
            message: None,
            show_help: false,
        })
    }

    pub fn can_quit(&self) -> bool {
        self.mode == Mode::Normal && self.input_mode == InputMode::None
    }

    pub fn refresh(&mut self) {
        self.tasks = self.db.list_tasks().unwrap_or_default();
        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }
    }

    pub fn check_scheduled_tasks(&mut self) {
        if let Ok(due_tasks) = self.db.get_due_tasks() {
            for mut task in due_tasks {
                self.run_task(&mut task);
            }
        }
    }

    fn run_task(&mut self, task: &mut ScheduledTask) {
        let run = TaskRun {
            id: 0,
            task_id: task.id,
            started_at: Utc::now(),
            finished_at: Some(Utc::now()),
            status: RunStatus::Success,
            exit_code: Some(0),
            output: format!("Simulated run of: {}", task.command),
        };

        let _ = self.db.insert_run(&run);

        task.last_run = Some(Utc::now());
        task.run_count += 1;
        task.update_next_run();
        let _ = self.db.update_task(task);

        self.message = Some(format!("Ran: {}", task.name));
        self.refresh();
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.message = None;

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.input_mode != InputMode::None {
            self.handle_input_key(key);
            return;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::Tasks => Pane::History,
                    Pane::History => Pane::Tasks,
                };
            }
            KeyCode::Enter => self.select_task(),
            KeyCode::Char('n') => self.start_new_task(),
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('e') => self.toggle_enabled(),
            KeyCode::Char('r') => self.run_selected_now(),
            KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::None;
                self.mode = Mode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Enter => self.advance_input(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn start_new_task(&mut self) {
        self.mode = Mode::Creating;
        self.input_mode = InputMode::TaskName;
        self.input_buffer.clear();
        self.new_task = NewTaskBuilder::default();
    }

    fn advance_input(&mut self) {
        match self.input_mode {
            InputMode::TaskName => {
                self.new_task.name = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_mode = InputMode::TaskCommand;
            }
            InputMode::TaskCommand => {
                self.new_task.command = self.input_buffer.clone();
                self.input_buffer.clear();
                self.input_mode = InputMode::ScheduleType;
            }
            InputMode::ScheduleType => {
                self.new_task.schedule_type = self.input_buffer.parse().unwrap_or(1);
                self.input_buffer.clear();
                self.input_mode = InputMode::ScheduleValue;
            }
            InputMode::ScheduleValue => {
                self.new_task.interval_mins = self.input_buffer.parse().unwrap_or(60);
                self.create_task();
                self.input_mode = InputMode::None;
                self.mode = Mode::Normal;
                self.input_buffer.clear();
            }
            InputMode::None => {}
        }
    }

    fn create_task(&mut self) {
        let schedule = match self.new_task.schedule_type {
            2 => Schedule::Daily { hour: 9, minute: 0 },
            _ => Schedule::Interval { minutes: self.new_task.interval_mins },
        };

        let task = ScheduledTask::new(&self.new_task.name, &self.new_task.command, schedule);
        if self.db.insert_task(&task).is_ok() {
            self.refresh();
            self.message = Some(format!("Created: {}", task.name));
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.tasks.len();
        if len == 0 { return; }
        let new_idx = self.selected_index as i32 + delta;
        self.selected_index = new_idx.clamp(0, len as i32 - 1) as usize;
    }

    fn select_task(&mut self) {
        if let Some(task) = self.tasks.get(self.selected_index) {
            self.current_task = Some(task.clone());
            self.task_runs = self.db.get_task_runs(task.id, 20).unwrap_or_default();
        }
    }

    fn delete_selected(&mut self) {
        if let Some(task) = self.tasks.get(self.selected_index) {
            if self.db.delete_task(task.id).is_ok() {
                self.refresh();
                self.message = Some("Deleted".to_string());
            }
        }
    }

    fn toggle_enabled(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_index) {
            task.enabled = !task.enabled;
            let _ = self.db.update_task(task);
            let enabled = task.enabled;
            self.refresh();
            self.message = Some(if enabled { "Enabled" } else { "Disabled" }.to_string());
        }
    }

    fn run_selected_now(&mut self) {
        if let Some(mut task) = self.tasks.get(self.selected_index).cloned() {
            self.run_task(&mut task);
        }
    }
}
