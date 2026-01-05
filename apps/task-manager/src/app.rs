//! Application state and logic.

use crate::config::Config;
use crate::db::{Database, DbResult};
use crate::models::{Context, Project, Status, Task, TaskStats};
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};

pub struct App {
    pub db: Database,
    pub config: Config,
    pub view: View,
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub contexts: Vec<Context>,
    pub selected_index: usize,
    pub filter: Filter,
    pub editing: bool,
    pub input_buffer: String,
    pub input_field: InputField,
    pub message: Option<String>,
    pub show_help: bool,
    pub show_completed: bool,
    pub stats: TaskStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Board,
    Projects,
    Today,
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub project_id: Option<i64>,
    pub status: Option<Status>,
    pub search: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    None,
    TaskTitle,
    TaskDescription,
    ProjectName,
    Search,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load();
        let db_path = Config::db_path().unwrap_or_else(|| "tasks.db".into());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path)?;

        let mut app = Self {
            db,
            config,
            view: View::List,
            tasks: Vec::new(),
            projects: Vec::new(),
            contexts: Vec::new(),
            selected_index: 0,
            filter: Filter::default(),
            editing: false,
            input_buffer: String::new(),
            input_field: InputField::None,
            message: None,
            show_help: false,
            show_completed: false,
            stats: TaskStats::default(),
        };

        app.refresh()?;
        Ok(app)
    }

    pub fn refresh(&mut self) -> DbResult<()> {
        self.projects = self.db.list_projects(false)?;
        self.contexts = self.db.list_contexts()?;
        self.reload_tasks()?;
        self.update_stats();

        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }

        Ok(())
    }

    fn reload_tasks(&mut self) -> DbResult<()> {
        self.tasks = if let Some(project_id) = self.filter.project_id {
            self.db.list_tasks_by_project(project_id)?
        } else if let Some(status) = self.filter.status {
            self.db.list_tasks_by_status(status)?
        } else {
            self.db.list_tasks(self.show_completed)?
        };

        if !self.filter.search.is_empty() {
            let search = self.filter.search.to_lowercase();
            self.tasks.retain(|t| {
                t.title.to_lowercase().contains(&search)
                    || t.description.to_lowercase().contains(&search)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&search))
            });
        }

        Ok(())
    }

    fn update_stats(&mut self) {
        self.stats = TaskStats::default();
        for task in &self.tasks {
            self.stats.total += 1;
            match task.status {
                Status::Todo => self.stats.todo += 1,
                Status::InProgress => self.stats.in_progress += 1,
                Status::Done => self.stats.done += 1,
                _ => {}
            }
            if task.is_overdue() {
                self.stats.overdue += 1;
            }
            if task.is_due_today() {
                self.stats.due_today += 1;
            }
        }
    }

    pub fn can_quit(&self) -> bool {
        !self.editing
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
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
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') => self.selected_index = 0,
            KeyCode::Char('G') => {
                if !self.tasks.is_empty() {
                    self.selected_index = self.tasks.len() - 1;
                }
            }
            KeyCode::Char('1') => {
                self.view = View::List;
                self.filter = Filter::default();
                let _ = self.refresh();
            }
            KeyCode::Char('2') => self.view = View::Board,
            KeyCode::Char('3') => self.view = View::Projects,
            KeyCode::Char('4') => {
                self.view = View::Today;
                if let Ok(tasks) = self.db.list_tasks_due_today() {
                    self.tasks = tasks;
                    self.selected_index = 0;
                }
            }
            KeyCode::Char('a') => self.start_add_task(),
            KeyCode::Char('A') => self.start_add_project(),
            KeyCode::Enter | KeyCode::Char(' ') => self.toggle_task_status(),
            KeyCode::Char('e') => self.start_edit_task(),
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('p') => self.cycle_priority(),
            KeyCode::Char('P') => self.assign_project(),
            KeyCode::Char('/') => {
                self.editing = true;
                self.input_field = InputField::Search;
                self.input_buffer.clear();
            }
            KeyCode::Char('c') => self.toggle_show_completed(),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Esc => {
                self.filter = Filter::default();
                let _ = self.refresh();
            }
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
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                if self.input_field == InputField::Search {
                    self.filter.search = self.input_buffer.clone();
                    let _ = self.reload_tasks();
                }
            }
            _ => {}
        }
    }

    fn finish_editing(&mut self) {
        match self.input_field {
            InputField::TaskTitle => {
                if !self.input_buffer.is_empty() {
                    let task = Task::new(&self.input_buffer);
                    if self.db.insert_task(&task).is_ok() {
                        self.message = Some("Task created".to_string());
                        let _ = self.refresh();
                    }
                }
            }
            InputField::TaskDescription => {
                if let Some(task) = self.tasks.get_mut(self.selected_index) {
                    task.description = self.input_buffer.clone();
                    let _ = self.db.update_task(task);
                    let _ = self.refresh();
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
            InputField::Search => {}
            InputField::None => {}
        }

        self.editing = false;
        self.input_buffer.clear();
        self.input_field = InputField::None;
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.tasks.len();
        if len == 0 {
            return;
        }
        let new_idx = self.selected_index as i32 + delta;
        self.selected_index = new_idx.clamp(0, len as i32 - 1) as usize;
    }

    fn start_add_task(&mut self) {
        self.editing = true;
        self.input_field = InputField::TaskTitle;
        self.input_buffer.clear();
    }

    fn start_add_project(&mut self) {
        self.editing = true;
        self.input_field = InputField::ProjectName;
        self.input_buffer.clear();
    }

    fn start_edit_task(&mut self) {
        if let Some(task) = self.tasks.get(self.selected_index) {
            self.editing = true;
            self.input_field = InputField::TaskDescription;
            self.input_buffer = task.description.clone();
        }
    }

    fn toggle_task_status(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_index) {
            let new_status = task.status.cycle();
            task.status = new_status;
            task.completed_at = if new_status == Status::Done {
                Some(Utc::now())
            } else {
                None
            };
            let _ = self.db.update_task(task);
            let _ = self.refresh();
            self.message = Some(format!("Status: {}", new_status.label()));
        }
    }

    fn delete_selected(&mut self) {
        if let Some(task) = self.tasks.get(self.selected_index) {
            if self.db.delete_task(task.id).is_ok() {
                self.message = Some("Task deleted".to_string());
                let _ = self.refresh();
            }
        }
    }

    fn cycle_priority(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_index) {
            task.priority = task.priority.next();
            let _ = self.db.update_task(task);
            let priority_label = task.priority.label().to_string();
            let _ = self.refresh();
            self.message = Some(format!("Priority: {}", priority_label));
        }
    }

    fn assign_project(&mut self) {
        if self.projects.is_empty() {
            self.message = Some("No projects. Press Shift+A to create one.".to_string());
            return;
        }
        if let Some(task) = self.tasks.get_mut(self.selected_index) {
            let current_idx = task.project_id
                .and_then(|id| self.projects.iter().position(|p| p.id == id));
            let next_idx = match current_idx {
                Some(i) if i + 1 < self.projects.len() => Some(i + 1),
                Some(_) => None,
                None => Some(0),
            };
            task.project_id = next_idx.map(|i| self.projects[i].id);
            let _ = self.db.update_task(task);
            let project_name = task.project_id
                .and_then(|id| self.projects.iter().find(|p| p.id == id))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "None".to_string());
            let _ = self.refresh();
            self.message = Some(format!("Project: {}", project_name));
        }
    }

    fn toggle_show_completed(&mut self) {
        self.show_completed = !self.show_completed;
        let _ = self.refresh();
        self.message = Some(if self.show_completed {
            "Showing completed tasks".to_string()
        } else {
            "Hiding completed tasks".to_string()
        });
    }

    pub fn get_project_name(&self, project_id: Option<i64>) -> Option<&str> {
        project_id.and_then(|id| {
            self.projects.iter().find(|p| p.id == id).map(|p| p.name.as_str())
        })
    }

    pub fn get_tasks_by_status(&self, status: Status) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.status == status).collect()
    }
}
