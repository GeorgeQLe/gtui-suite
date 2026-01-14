use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use handlebars::Handlebars;
use regex::Regex;

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    WizardList,
    RunWizard,
    Preview,
    CreateWizard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Input,
    Select,
    MultiSelect,
    Confirm,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,

    // Wizard list
    pub wizards: Vec<WizardDefinition>,
    pub selected_wizard: usize,

    // Running wizard
    pub session: Option<WizardSession>,
    pub input_buffer: String,
    pub cursor_position: usize,
    pub selected_option: usize,
    pub selected_options: Vec<bool>,
    pub validation_error: Option<String>,

    // Answer history for undo
    pub answer_history: Vec<Answer>,

    // Preview
    pub preview_content: Option<String>,
    pub preview_scroll: usize,

    // Status
    pub status_message: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            view: View::WizardList,
            input_mode: InputMode::Normal,
            wizards: Vec::new(),
            selected_wizard: 0,
            session: None,
            input_buffer: String::new(),
            cursor_position: 0,
            selected_option: 0,
            selected_options: Vec::new(),
            validation_error: None,
            answer_history: Vec::new(),
            preview_content: None,
            preview_scroll: 0,
            status_message: None,
        }
    }

    pub async fn refresh(&mut self) {
        // Load demo wizards
        self.wizards = vec![
            create_docker_wizard(),
            create_rust_project_wizard(),
            create_config_wizard(),
        ];
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Global shortcuts
        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('p') if is_ctrl => {
                self.show_preview();
                return false;
            }
            KeyCode::Char('z') if is_ctrl => {
                self.undo_answer();
                return false;
            }
            _ => {}
        }

        match self.view {
            View::WizardList => self.handle_wizard_list_key(key),
            View::RunWizard => self.handle_run_wizard_key(key),
            View::Preview => self.handle_preview_key(key),
            View::CreateWizard => self.handle_create_wizard_key(key),
        }
    }

    fn handle_wizard_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_wizard < self.wizards.len().saturating_sub(1) {
                    self.selected_wizard += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_wizard = self.selected_wizard.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.start_wizard();
            }
            KeyCode::Char('n') => {
                self.view = View::CreateWizard;
                self.input_buffer.clear();
            }
            _ => {}
        }
        false
    }

    fn handle_run_wizard_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_wizard_normal_key(key),
            InputMode::Input => self.handle_input_key(key),
            InputMode::Select => self.handle_select_key(key),
            InputMode::MultiSelect => self.handle_multi_select_key(key),
            InputMode::Confirm => self.handle_confirm_key(key),
        }
    }

    fn handle_wizard_normal_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::WizardList;
                self.session = None;
                self.answer_history.clear();
            }
            KeyCode::Enter => {
                self.start_input_for_current_question();
            }
            _ => {}
        }
        false
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.validation_error = None;
            }
            KeyCode::Enter => {
                self.submit_answer();
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
                self.validate_input();
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input_buffer.len() {
                    self.input_buffer.remove(self.cursor_position);
                }
                self.validate_input();
            }
            KeyCode::Left => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.cursor_position < self.input_buffer.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.input_buffer.len();
            }
            KeyCode::Char(c) => {
                self.input_buffer.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.validate_input();
            }
            _ => {}
        }
        false
    }

    fn handle_select_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(session) = &self.session {
                    if let Some(question) = session.current_question() {
                        if self.selected_option < question.options.len().saturating_sub(1) {
                            self.selected_option += 1;
                        }
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_option = self.selected_option.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.submit_select_answer();
            }
            _ => {}
        }
        false
    }

    fn handle_multi_select_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_option < self.selected_options.len().saturating_sub(1) {
                    self.selected_option += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_option = self.selected_option.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                if self.selected_option < self.selected_options.len() {
                    self.selected_options[self.selected_option] =
                        !self.selected_options[self.selected_option];
                }
            }
            KeyCode::Enter => {
                self.submit_multi_select_answer();
            }
            _ => {}
        }
        false
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.submit_confirm_answer(true);
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.submit_confirm_answer(false);
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.selected_option = if self.selected_option == 0 { 1 } else { 0 };
            }
            KeyCode::Enter => {
                self.submit_confirm_answer(self.selected_option == 0);
            }
            _ => {}
        }
        false
    }

    fn handle_preview_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = if self.session.is_some() {
                    View::RunWizard
                } else {
                    View::WizardList
                };
                self.preview_content = None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.preview_scroll += 1;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.preview_scroll = self.preview_scroll.saturating_sub(1);
            }
            KeyCode::Char('w') => {
                self.write_output();
            }
            _ => {}
        }
        false
    }

    fn handle_create_wizard_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::WizardList;
                self.input_buffer.clear();
            }
            _ => {}
        }
        false
    }

    fn start_wizard(&mut self) {
        if let Some(wizard) = self.wizards.get(self.selected_wizard).cloned() {
            self.session = Some(WizardSession::new(wizard));
            self.view = View::RunWizard;
            self.answer_history.clear();
            self.skip_to_next_applicable_question();
            self.start_input_for_current_question();
        }
    }

    fn start_input_for_current_question(&mut self) {
        if let Some(session) = &self.session {
            if let Some(question) = session.current_question() {
                self.input_buffer.clear();
                self.cursor_position = 0;
                self.selected_option = 0;
                self.validation_error = None;

                // Set default value if available
                if let Some(ref default) = question.default {
                    if let Some(s) = default.as_str() {
                        self.input_buffer = s.to_string();
                        self.cursor_position = self.input_buffer.len();
                    }
                }

                self.input_mode = match question.question_type {
                    QuestionType::Text | QuestionType::Password | QuestionType::Number | QuestionType::Path => {
                        InputMode::Input
                    }
                    QuestionType::Select => InputMode::Select,
                    QuestionType::MultiSelect => {
                        self.selected_options = vec![false; question.options.len()];
                        InputMode::MultiSelect
                    }
                    QuestionType::Confirm => {
                        // Set default selection based on default value
                        if let Some(ref default) = question.default {
                            self.selected_option = if default.as_bool().unwrap_or(true) { 0 } else { 1 };
                        }
                        InputMode::Confirm
                    }
                };
            }
        }
    }

    fn validate_input(&mut self) {
        self.validation_error = None;

        if let Some(session) = &self.session {
            if let Some(question) = session.current_question() {
                if let Some(ref validation) = question.validation {
                    // Check pattern
                    if let Some(ref pattern) = validation.pattern {
                        if let Ok(re) = Regex::new(pattern) {
                            if !re.is_match(&self.input_buffer) {
                                self.validation_error = Some(
                                    validation
                                        .message
                                        .clone()
                                        .unwrap_or_else(|| "Invalid input format".to_string()),
                                );
                                return;
                            }
                        }
                    }

                    // Check min/max length
                    if let Some(min) = validation.min_length {
                        if self.input_buffer.len() < min {
                            self.validation_error = Some(format!("Minimum {} characters required", min));
                            return;
                        }
                    }

                    if let Some(max) = validation.max_length {
                        if self.input_buffer.len() > max {
                            self.validation_error = Some(format!("Maximum {} characters allowed", max));
                            return;
                        }
                    }

                    // Check number range
                    if question.question_type == QuestionType::Number {
                        if let Ok(num) = self.input_buffer.parse::<f64>() {
                            if let Some(min) = validation.min {
                                if num < min {
                                    self.validation_error = Some(format!("Minimum value is {}", min));
                                    return;
                                }
                            }
                            if let Some(max) = validation.max {
                                if num > max {
                                    self.validation_error = Some(format!("Maximum value is {}", max));
                                    return;
                                }
                            }
                        } else if !self.input_buffer.is_empty() {
                            self.validation_error = Some("Please enter a valid number".to_string());
                        }
                    }
                }
            }
        }
    }

    fn submit_answer(&mut self) {
        if self.validation_error.is_some() {
            return;
        }

        let answer_value = if let Some(session) = &self.session {
            if let Some(question) = session.current_question() {
                match question.question_type {
                    QuestionType::Text | QuestionType::Password | QuestionType::Path => {
                        Some(AnswerValue::Text(self.input_buffer.clone()))
                    }
                    QuestionType::Number => {
                        if let Ok(num) = self.input_buffer.parse::<f64>() {
                            Some(AnswerValue::Number(num))
                        } else {
                            self.validation_error = Some("Please enter a valid number".to_string());
                            return;
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(value) = answer_value {
            self.record_answer(value);
        }
    }

    fn submit_select_answer(&mut self) {
        if let Some(session) = &self.session {
            if let Some(question) = session.current_question() {
                if let Some(option) = question.options.get(self.selected_option) {
                    let value = AnswerValue::Selected(option.value.clone());
                    self.record_answer(value);
                }
            }
        }
    }

    fn submit_multi_select_answer(&mut self) {
        if let Some(session) = &self.session {
            if let Some(question) = session.current_question() {
                let selected: Vec<String> = question
                    .options
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| self.selected_options.get(*i).copied().unwrap_or(false))
                    .map(|(_, opt)| opt.value.clone())
                    .collect();
                let value = AnswerValue::MultiSelected(selected);
                self.record_answer(value);
            }
        }
    }

    fn submit_confirm_answer(&mut self, confirmed: bool) {
        let value = AnswerValue::Boolean(confirmed);
        self.record_answer(value);
    }

    fn record_answer(&mut self, value: AnswerValue) {
        let should_check_completion = {
            let Some(session) = &mut self.session else {
                return;
            };
            let Some(question) = session.current_question().cloned() else {
                return;
            };
            let answer = Answer::new(&question.id, value);
            self.answer_history.push(answer.clone());
            session.answers.push(answer);

            // Move to next question
            session.current_question += 1;
            true
        };

        if should_check_completion {
            self.skip_to_next_applicable_question();

            let is_completed = self.session.as_ref()
                .map(|s| s.current_question >= s.wizard.questions.len())
                .unwrap_or(false);

            if is_completed {
                if let Some(session) = &mut self.session {
                    session.completed = true;
                }
                self.show_preview();
            } else {
                self.start_input_for_current_question();
            }
        }
    }

    fn skip_to_next_applicable_question(&mut self) {
        if let Some(session) = &mut self.session {
            while session.current_question < session.wizard.questions.len() {
                let question = &session.wizard.questions[session.current_question];
                if session.should_show_question(question) {
                    break;
                }
                session.current_question += 1;
            }
        }
    }

    fn undo_answer(&mut self) {
        if let Some(session) = &mut self.session {
            if let Some(answer) = self.answer_history.pop() {
                // Remove from session answers
                session.answers.retain(|a| a.question_id != answer.question_id);

                // Find the question index
                for (i, q) in session.wizard.questions.iter().enumerate() {
                    if q.id == answer.question_id {
                        session.current_question = i;
                        session.completed = false;
                        break;
                    }
                }

                self.start_input_for_current_question();
                self.status_message = Some("Undid last answer".to_string());
            }
        }
    }

    fn show_preview(&mut self) {
        if let Some(session) = &self.session {
            let context = session.get_context();
            let content = self.generate_output(&session.wizard, &context);
            self.preview_content = Some(content);
            self.preview_scroll = 0;
            self.view = View::Preview;
        }
    }

    fn generate_output(&self, wizard: &WizardDefinition, context: &serde_json::Map<String, serde_json::Value>) -> String {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let outputs = wizard.get_outputs();
        if outputs.is_empty() {
            // Default: output context as YAML
            return serde_yaml::to_string(context).unwrap_or_else(|_| "Error generating output".to_string());
        }

        let mut result = String::new();
        for output in outputs {
            if let Some(ref template) = output.template {
                // For demo, use template name as content
                let demo_template = get_demo_template(template);
                if let Ok(rendered) = handlebars.render_template(&demo_template, context) {
                    if !result.is_empty() {
                        result.push_str("\n---\n");
                    }
                    if let Some(ref path) = output.path {
                        result.push_str(&format!("# File: {}\n", path));
                    }
                    result.push_str(&rendered);
                }
            }
        }

        if result.is_empty() {
            serde_yaml::to_string(context).unwrap_or_else(|_| "Error generating output".to_string())
        } else {
            result
        }
    }

    fn write_output(&mut self) {
        if self.config.output.dry_run {
            self.status_message = Some("Dry run - no files written".to_string());
        } else {
            self.status_message = Some("Output written successfully".to_string());
        }
        self.view = View::WizardList;
        self.session = None;
        self.preview_content = None;
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::WizardList => {
                format!("{} wizards | j/k:navigate Enter:run n:new q:quit", self.wizards.len())
            }
            View::RunWizard => {
                if let Some(session) = &self.session {
                    let (current, total) = session.progress();
                    format!(
                        "Question {}/{} | Enter:submit Ctrl+z:undo Ctrl+p:preview Esc:cancel",
                        current, total
                    )
                } else {
                    "No wizard running".to_string()
                }
            }
            View::Preview => "j/k:scroll w:write q:cancel".to_string(),
            View::CreateWizard => "Enter:save Esc:cancel".to_string(),
        }
    }
}

fn create_docker_wizard() -> WizardDefinition {
    let mut wizard = WizardDefinition::new("Docker Setup Wizard");
    wizard.description = "Configure a Docker development environment".to_string();

    wizard.questions = vec![
        {
            let mut q = Question::new("project_name", QuestionType::Text, "Project name?");
            q.validation = Some(Validation {
                pattern: Some("^[a-z][a-z0-9-]*$".to_string()),
                message: Some("Must start with letter, only lowercase and hyphens".to_string()),
                ..Default::default()
            });
            q
        },
        {
            let mut q = Question::new("framework", QuestionType::Select, "Which framework?");
            q.options = vec![
                QuestionOption::new("Node.js", "node"),
                QuestionOption::new("Python", "python"),
                QuestionOption::new("Rust", "rust"),
                QuestionOption::new("Go", "go"),
            ];
            q
        },
        {
            let mut q = Question::new("with_database", QuestionType::Confirm, "Include database?");
            q.default = Some(serde_json::Value::Bool(true));
            q
        },
        {
            let mut q = Question::new("database_type", QuestionType::Select, "Which database?");
            q.when = Some("with_database".to_string());
            q.options = vec![
                QuestionOption::new("PostgreSQL", "postgres"),
                QuestionOption::new("MySQL", "mysql"),
                QuestionOption::new("MongoDB", "mongo"),
            ];
            q
        },
        {
            let mut q = Question::new("ports", QuestionType::MultiSelect, "Expose which ports?");
            q.options = vec![
                QuestionOption::new("HTTP (80)", "80"),
                QuestionOption::new("HTTPS (443)", "443"),
                QuestionOption::new("App (3000)", "3000"),
                QuestionOption::new("API (8080)", "8080"),
            ];
            q
        },
    ];

    wizard.output = Some({
        let mut output = OutputConfig::new(OutputType::File);
        output.template = Some("docker-compose.yml.hbs".to_string());
        output.path = Some("{{project_name}}/docker-compose.yml".to_string());
        output
    });

    wizard
}

fn create_rust_project_wizard() -> WizardDefinition {
    let mut wizard = WizardDefinition::new("Rust Project Wizard");
    wizard.description = "Create a new Rust project with common configurations".to_string();

    wizard.questions = vec![
        {
            let mut q = Question::new("name", QuestionType::Text, "Project name?");
            q.validation = Some(Validation {
                pattern: Some("^[a-z][a-z0-9_-]*$".to_string()),
                message: Some("Must be a valid crate name".to_string()),
                ..Default::default()
            });
            q
        },
        {
            let mut q = Question::new("project_type", QuestionType::Select, "Project type?");
            q.options = vec![
                QuestionOption::new("Binary", "bin"),
                QuestionOption::new("Library", "lib"),
                QuestionOption::new("Workspace", "workspace"),
            ];
            q
        },
        {
            let mut q = Question::new("edition", QuestionType::Select, "Rust edition?");
            q.options = vec![
                QuestionOption::new("2021", "2021"),
                QuestionOption::new("2018", "2018"),
            ];
            q.default = Some(serde_json::Value::String("2021".to_string()));
            q
        },
        {
            let mut q = Question::new("features", QuestionType::MultiSelect, "Include features?");
            q.options = vec![
                QuestionOption::new("Async (tokio)", "async"),
                QuestionOption::new("Serialization (serde)", "serde"),
                QuestionOption::new("CLI (clap)", "cli"),
                QuestionOption::new("Error handling (anyhow)", "errors"),
            ];
            q
        },
    ];

    wizard.output = Some({
        let mut output = OutputConfig::new(OutputType::File);
        output.template = Some("cargo.toml.hbs".to_string());
        output.path = Some("{{name}}/Cargo.toml".to_string());
        output
    });

    wizard
}

fn create_config_wizard() -> WizardDefinition {
    let mut wizard = WizardDefinition::new("Config File Generator");
    wizard.description = "Generate configuration files for various tools".to_string();

    wizard.questions = vec![
        {
            let mut q = Question::new("tool", QuestionType::Select, "Which tool?");
            q.options = vec![
                QuestionOption::new("ESLint", "eslint"),
                QuestionOption::new("Prettier", "prettier"),
                QuestionOption::new("TypeScript", "typescript"),
                QuestionOption::new("Webpack", "webpack"),
            ];
            q
        },
        {
            let mut q = Question::new("format", QuestionType::Select, "Config format?");
            q.options = vec![
                QuestionOption::new("JSON", "json"),
                QuestionOption::new("YAML", "yaml"),
                QuestionOption::new("JavaScript", "js"),
            ];
            q
        },
    ];

    wizard.output = Some({
        let mut output = OutputConfig::new(OutputType::Stdout);
        output.template = Some("config.hbs".to_string());
        output
    });

    wizard
}

fn get_demo_template(template_name: &str) -> String {
    match template_name {
        "docker-compose.yml.hbs" => r#"version: "3.8"
services:
  {{project_name}}:
    image: {{framework}}:latest
    ports:
{{#each ports}}
      - "{{this}}:{{this}}"
{{/each}}
{{#if with_database}}
    depends_on:
      - db

  db:
    image: {{database_type}}:latest
    environment:
      - POSTGRES_PASSWORD=postgres
{{/if}}
"#.to_string(),
        "cargo.toml.hbs" => r#"[package]
name = "{{name}}"
version = "0.1.0"
edition = "{{edition}}"

[dependencies]
{{#each features}}
{{#if (eq this "async")}}
tokio = { version = "1", features = ["full"] }
{{/if}}
{{#if (eq this "serde")}}
serde = { version = "1", features = ["derive"] }
serde_json = "1"
{{/if}}
{{#if (eq this "cli")}}
clap = { version = "4", features = ["derive"] }
{{/if}}
{{#if (eq this "errors")}}
anyhow = "1"
thiserror = "2"
{{/if}}
{{/each}}
"#.to_string(),
        _ => "# Generated configuration\n{{#each this}}{{@key}}: {{this}}\n{{/each}}".to_string(),
    }
}
