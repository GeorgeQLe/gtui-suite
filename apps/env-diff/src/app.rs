use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStatus {
    Same,
    Different,
    OnlyLeft,
    OnlyRight,
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub key: String,
    pub left_value: Option<String>,
    pub right_value: Option<String>,
    pub status: DiffStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Different,
    Same,
    OnlyLeft,
    OnlyRight,
}

pub struct App {
    pub left_env: Option<Environment>,
    pub right_env: Option<Environment>,
    pub diff_entries: Vec<DiffEntry>,
    pub filtered_entries: Vec<usize>,
    pub selected: usize,
    pub filter: Filter,
    pub show_values: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            left_env: None,
            right_env: None,
            diff_entries: Vec::new(),
            filtered_entries: Vec::new(),
            selected: 0,
            filter: Filter::All,
            show_values: true,
            status_message: None,
        }
    }

    pub fn load_environments(&mut self) {
        self.left_env = Some(create_demo_env("Development"));
        self.right_env = Some(create_demo_env("Production"));
        self.compute_diff();
    }

    fn compute_diff(&mut self) {
        self.diff_entries.clear();

        let Some(left) = &self.left_env else { return };
        let Some(right) = &self.right_env else { return };

        let mut all_keys: Vec<&String> = left.vars.keys().chain(right.vars.keys()).collect();
        all_keys.sort();
        all_keys.dedup();

        for key in all_keys {
            let left_value = left.vars.get(key).cloned();
            let right_value = right.vars.get(key).cloned();

            let status = match (&left_value, &right_value) {
                (Some(l), Some(r)) if l == r => DiffStatus::Same,
                (Some(_), Some(_)) => DiffStatus::Different,
                (Some(_), None) => DiffStatus::OnlyLeft,
                (None, Some(_)) => DiffStatus::OnlyRight,
                (None, None) => continue,
            };

            self.diff_entries.push(DiffEntry {
                key: key.clone(),
                left_value,
                right_value,
                status,
            });
        }

        self.update_filtered();
    }

    fn update_filtered(&mut self) {
        self.filtered_entries = self
            .diff_entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| match self.filter {
                Filter::All => true,
                Filter::Different => entry.status == DiffStatus::Different,
                Filter::Same => entry.status == DiffStatus::Same,
                Filter::OnlyLeft => entry.status == DiffStatus::OnlyLeft,
                Filter::OnlyRight => entry.status == DiffStatus::OnlyRight,
            })
            .map(|(i, _)| i)
            .collect();

        self.selected = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.filtered_entries.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('a') => {
                self.filter = Filter::All;
                self.update_filtered();
                self.status_message = Some("Filter: All".to_string());
            }
            KeyCode::Char('d') => {
                self.filter = Filter::Different;
                self.update_filtered();
                self.status_message = Some("Filter: Different".to_string());
            }
            KeyCode::Char('s') => {
                self.filter = Filter::Same;
                self.update_filtered();
                self.status_message = Some("Filter: Same".to_string());
            }
            KeyCode::Char('l') => {
                self.filter = Filter::OnlyLeft;
                self.update_filtered();
                self.status_message = Some("Filter: Only Left".to_string());
            }
            KeyCode::Char('r') => {
                self.filter = Filter::OnlyRight;
                self.update_filtered();
                self.status_message = Some("Filter: Only Right".to_string());
            }
            KeyCode::Char('v') => {
                self.show_values = !self.show_values;
                self.status_message = Some(format!(
                    "Values: {}",
                    if self.show_values { "shown" } else { "hidden" }
                ));
            }
            KeyCode::Char('y') => {
                if let Some(entry) = self.selected_entry() {
                    self.status_message = Some(format!("Copied: {}", entry.key));
                }
            }
            _ => {}
        }
        false
    }

    pub fn selected_entry(&self) -> Option<&DiffEntry> {
        self.filtered_entries
            .get(self.selected)
            .and_then(|&i| self.diff_entries.get(i))
    }

    pub fn visible_entries(&self) -> Vec<&DiffEntry> {
        self.filtered_entries
            .iter()
            .filter_map(|&i| self.diff_entries.get(i))
            .collect()
    }

    pub fn count_by_status(&self, status: DiffStatus) -> usize {
        self.diff_entries
            .iter()
            .filter(|e| e.status == status)
            .count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        format!(
            "{} vars | a:all d:diff s:same l:left r:right v:toggle y:copy",
            self.filtered_entries.len()
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_env(name: &str) -> Environment {
    let mut vars = HashMap::new();

    // Common variables with same values
    vars.insert("PATH".to_string(), "/usr/local/bin:/usr/bin:/bin".to_string());
    vars.insert("SHELL".to_string(), "/bin/bash".to_string());
    vars.insert("LANG".to_string(), "en_US.UTF-8".to_string());

    // Variables with different values based on environment
    if name == "Development" {
        vars.insert("NODE_ENV".to_string(), "development".to_string());
        vars.insert("DEBUG".to_string(), "true".to_string());
        vars.insert("LOG_LEVEL".to_string(), "debug".to_string());
        vars.insert("DATABASE_URL".to_string(), "postgres://localhost:5432/dev_db".to_string());
        vars.insert("API_URL".to_string(), "http://localhost:3000".to_string());
        vars.insert("DEV_ONLY_VAR".to_string(), "some_dev_value".to_string());
    } else {
        vars.insert("NODE_ENV".to_string(), "production".to_string());
        vars.insert("DEBUG".to_string(), "false".to_string());
        vars.insert("LOG_LEVEL".to_string(), "warn".to_string());
        vars.insert("DATABASE_URL".to_string(), "postgres://prod-db.example.com:5432/prod_db".to_string());
        vars.insert("API_URL".to_string(), "https://api.example.com".to_string());
        vars.insert("PROD_ONLY_VAR".to_string(), "some_prod_value".to_string());
    }

    Environment {
        name: name.to_string(),
        vars,
    }
}
