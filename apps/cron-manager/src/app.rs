use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Create,
    Edit,
    Presets,
    Details,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    EditExpression,
    EditCommand,
    EditDescription,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Expression,
    Command,
    Description,
}

pub struct App {
    pub view: View,
    pub input_mode: InputMode,
    pub jobs: Vec<CronJob>,
    pub selected: usize,
    pub selected_preset: usize,

    // Edit state
    pub edit_field: EditField,
    pub edit_buffer: String,
    pub edit_job: Option<CronJob>,
    pub validation_error: Option<String>,

    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::List,
            input_mode: InputMode::Normal,
            jobs: Vec::new(),
            selected: 0,
            selected_preset: 0,
            edit_field: EditField::Expression,
            edit_buffer: String::new(),
            edit_job: None,
            validation_error: None,
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        self.jobs = create_demo_jobs();
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') if self.input_mode == InputMode::Normal && self.view == View::List => {
                return true
            }
            _ => {}
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::EditExpression | InputMode::EditCommand | InputMode::EditDescription => {
                self.handle_edit_key(key)
            }
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match self.view {
            View::List => self.handle_list_key(key),
            View::Create | View::Edit => self.handle_create_key(key),
            View::Presets => self.handle_presets_key(key),
            View::Details => {
                if key.code == KeyCode::Esc {
                    self.view = View::List;
                }
                false
            }
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.jobs.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.view = View::Details;
            }
            KeyCode::Char('n') => {
                self.start_create();
            }
            KeyCode::Char('e') => {
                self.start_edit();
            }
            KeyCode::Char('d') => {
                self.delete_selected();
            }
            KeyCode::Char('t') => {
                self.toggle_enabled();
            }
            KeyCode::Char('p') => {
                self.view = View::Presets;
                self.selected_preset = 0;
            }
            _ => {}
        }
        false
    }

    fn handle_create_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
                self.edit_job = None;
                self.validation_error = None;
            }
            KeyCode::Tab => {
                self.edit_field = match self.edit_field {
                    EditField::Expression => EditField::Command,
                    EditField::Command => EditField::Description,
                    EditField::Description => EditField::Expression,
                };
            }
            KeyCode::Enter => {
                self.input_mode = match self.edit_field {
                    EditField::Expression => InputMode::EditExpression,
                    EditField::Command => InputMode::EditCommand,
                    EditField::Description => InputMode::EditDescription,
                };
                if let Some(ref job) = self.edit_job {
                    self.edit_buffer = match self.edit_field {
                        EditField::Expression => job.expression.to_string(),
                        EditField::Command => job.command.clone(),
                        EditField::Description => job.description.clone().unwrap_or_default(),
                    };
                }
            }
            KeyCode::Char('p') => {
                self.view = View::Presets;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_job();
            }
            _ => {}
        }
        false
    }

    fn handle_presets_key(&mut self, key: KeyEvent) -> bool {
        let presets = CronPreset::all();
        match key.code {
            KeyCode::Esc => {
                self.view = if self.edit_job.is_some() {
                    View::Create
                } else {
                    View::List
                };
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_preset < presets.len().saturating_sub(1) {
                    self.selected_preset += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_preset = self.selected_preset.saturating_sub(1);
            }
            KeyCode::Enter => {
                let preset = presets[self.selected_preset];
                if let Some(ref mut job) = self.edit_job {
                    if let Some(expr) = CronExpression::parse(preset.expression()) {
                        job.expression = expr;
                    }
                }
                self.view = View::Create;
            }
            _ => {}
        }
        false
    }

    fn handle_edit_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                // Apply changes
                if let Some(ref mut job) = self.edit_job {
                    match self.edit_field {
                        EditField::Expression => {
                            if let Some(expr) = CronExpression::parse(&self.edit_buffer) {
                                job.expression = expr;
                                self.validation_error = None;
                            } else {
                                self.validation_error = Some("Invalid cron expression".to_string());
                            }
                        }
                        EditField::Command => {
                            job.command = self.edit_buffer.clone();
                        }
                        EditField::Description => {
                            job.description = if self.edit_buffer.is_empty() {
                                None
                            } else {
                                Some(self.edit_buffer.clone())
                            };
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
                self.edit_buffer.clear();
            }
            KeyCode::Enter => {
                // Apply and move to next field
                if let Some(ref mut job) = self.edit_job {
                    match self.edit_field {
                        EditField::Expression => {
                            if let Some(expr) = CronExpression::parse(&self.edit_buffer) {
                                job.expression = expr;
                                self.validation_error = None;
                            } else {
                                self.validation_error = Some("Invalid cron expression".to_string());
                            }
                        }
                        EditField::Command => {
                            job.command = self.edit_buffer.clone();
                        }
                        EditField::Description => {
                            job.description = if self.edit_buffer.is_empty() {
                                None
                            } else {
                                Some(self.edit_buffer.clone())
                            };
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
                self.edit_buffer.clear();
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.edit_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn start_create(&mut self) {
        let default_expr = CronExpression::parse("0 * * * *").unwrap();
        self.edit_job = Some(CronJob::new(default_expr, ""));
        self.edit_field = EditField::Expression;
        self.view = View::Create;
    }

    fn start_edit(&mut self) {
        if let Some(job) = self.jobs.get(self.selected) {
            self.edit_job = Some(job.clone());
            self.edit_field = EditField::Expression;
            self.view = View::Edit;
        }
    }

    fn save_job(&mut self) {
        if let Some(job) = self.edit_job.take() {
            if job.command.is_empty() {
                self.validation_error = Some("Command is required".to_string());
                self.edit_job = Some(job);
                return;
            }

            if self.view == View::Edit {
                if let Some(pos) = self.jobs.iter().position(|j| j.id == job.id) {
                    self.jobs[pos] = job;
                }
            } else {
                self.jobs.insert(0, job);
            }
            self.status_message = Some("Job saved!".to_string());
            self.validation_error = None;
        }
        self.view = View::List;
    }

    fn delete_selected(&mut self) {
        if self.selected < self.jobs.len() {
            self.jobs.remove(self.selected);
            if self.selected >= self.jobs.len() {
                self.selected = self.jobs.len().saturating_sub(1);
            }
            self.status_message = Some("Job deleted".to_string());
        }
    }

    fn toggle_enabled(&mut self) {
        if let Some(job) = self.jobs.get_mut(self.selected) {
            job.enabled = !job.enabled;
            self.status_message = Some(if job.enabled {
                "Job enabled".to_string()
            } else {
                "Job disabled".to_string()
            });
        }
    }

    pub fn selected_job(&self) -> Option<&CronJob> {
        self.jobs.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        if let Some(ref err) = self.validation_error {
            return format!("Error: {}", err);
        }

        match self.view {
            View::List => format!(
                "{} jobs | n:new e:edit d:delete t:toggle p:presets",
                self.jobs.len()
            ),
            View::Create | View::Edit => {
                "Tab:next field Enter:edit p:presets Ctrl+s:save Esc:cancel".to_string()
            }
            View::Presets => "j/k:navigate Enter:select Esc:cancel".to_string(),
            View::Details => "Esc:back".to_string(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_jobs() -> Vec<CronJob> {
    vec![
        {
            let expr = CronExpression::parse("0 * * * *").unwrap();
            let mut job = CronJob::new(expr, "/usr/local/bin/backup.sh");
            job.description = Some("Hourly backup".to_string());
            job
        },
        {
            let expr = CronExpression::parse("0 2 * * *").unwrap();
            let mut job = CronJob::new(expr, "/opt/scripts/cleanup.sh");
            job.description = Some("Daily cleanup at 2 AM".to_string());
            job
        },
        {
            let expr = CronExpression::parse("*/5 * * * *").unwrap();
            let mut job = CronJob::new(expr, "curl -s https://api.example.com/health");
            job.description = Some("Health check every 5 minutes".to_string());
            job
        },
        {
            let expr = CronExpression::parse("0 0 * * 0").unwrap();
            let mut job = CronJob::new(expr, "/usr/local/bin/weekly-report.sh");
            job.description = Some("Weekly report on Sundays".to_string());
            job
        },
        {
            let expr = CronExpression::parse("0 0 1 * *").unwrap();
            let mut job = CronJob::new(expr, "/opt/scripts/monthly-archive.sh");
            job.description = Some("Monthly archive on the 1st".to_string());
            job.enabled = false;
            job
        },
    ]
}
