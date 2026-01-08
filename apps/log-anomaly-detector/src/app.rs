use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::config::Config;
use crate::detector::Detector;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Alerts,
    Rules,
    Training,
}

impl View {
    pub fn all() -> &'static [View] {
        &[View::Dashboard, View::Alerts, View::Rules, View::Training]
    }

    pub fn name(&self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Alerts => "Alerts",
            View::Rules => "Rules",
            View::Training => "Training",
        }
    }

    pub fn next(&self) -> View {
        let views = Self::all();
        let idx = views.iter().position(|v| v == self).unwrap_or(0);
        views[(idx + 1) % views.len()]
    }

    pub fn prev(&self) -> View {
        let views = Self::all();
        let idx = views.iter().position(|v| v == self).unwrap_or(0);
        views[(idx + views.len() - 1) % views.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    AlertDetail,
    RuleEdit,
    Search,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,
    pub auto_scan: bool,

    // Detection
    detector: Detector,
    pub rules: Vec<PatternRule>,

    // Alerts
    pub alerts: Vec<Alert>,
    pub selected: usize,
    pub selected_alert: Option<usize>,

    // Baseline
    pub baseline: BaselineStats,

    // UI state
    pub search_query: String,
    pub error: Option<String>,

    // Stats
    pub lines_scanned: u64,
    pub last_scan_matches: usize,
}

impl App {
    pub fn new(config: Config) -> Self {
        let rules = if config.rules.builtin {
            Detector::default_rules()
        } else {
            Vec::new()
        };
        let detector = Detector::new(rules.clone());

        Self {
            config,
            view: View::Dashboard,
            input_mode: InputMode::Normal,
            auto_scan: true,
            detector,
            rules,
            alerts: Vec::new(),
            selected: 0,
            selected_alert: None,
            baseline: BaselineStats::default(),
            search_query: String::new(),
            error: None,
            lines_scanned: 0,
            last_scan_matches: 0,
        }
    }

    pub fn scan_logs(&mut self) {
        self.last_scan_matches = 0;
        self.error = None;

        for file_path in &self.config.input.files.clone() {
            if let Err(e) = self.scan_file(file_path) {
                self.error = Some(format!("Error scanning {}: {}", file_path, e));
            }
        }

        // Update baseline
        self.baseline.total_lines = self.lines_scanned;
        self.baseline.error_count = self
            .alerts
            .iter()
            .filter(|a| a.severity >= Severity::Error)
            .map(|a| a.count as u64)
            .sum();
        self.baseline.warning_count = self
            .alerts
            .iter()
            .filter(|a| a.severity == Severity::Warning)
            .map(|a| a.count as u64)
            .sum();
    }

    fn scan_file(&mut self, path: &str) -> anyhow::Result<()> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Ok(()), // Skip files that don't exist
        };
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let content = line?;
            self.lines_scanned += 1;

            let entry = LogEntry::new(path, line_num + 1, &content);
            let matches = self.detector.check_line(&entry);

            for (rule, _groups) in matches {
                self.last_scan_matches += 1;

                // Check if we should add to existing alert or create new
                if let Some(existing) = self
                    .alerts
                    .iter_mut()
                    .find(|a| a.rule_name == rule.name && !a.acknowledged)
                {
                    existing.add_entry(entry.clone());
                } else {
                    let alert =
                        Alert::new(&rule.name, rule.severity, &rule.description, entry.clone());
                    self.alerts.push(alert);
                }
            }
        }

        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::AlertDetail => self.handle_alert_detail_key(key),
            InputMode::RuleEdit => self.handle_rule_edit_key(key),
            InputMode::Search => self.handle_search_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c') if is_ctrl => return true,
            KeyCode::Char('q') => return true,

            KeyCode::Tab => {
                self.view = self.view.next();
                self.selected = 0;
            }
            KeyCode::BackTab => {
                self.view = self.view.prev();
                self.selected = 0;
            }

            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),

            KeyCode::Char('r') => {
                self.view = View::Rules;
                self.selected = 0;
            }
            KeyCode::Char('R') => {
                self.scan_logs();
            }

            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }

            KeyCode::Enter => match self.view {
                View::Alerts => {
                    if !self.alerts.is_empty() {
                        self.selected_alert = Some(self.selected);
                        self.input_mode = InputMode::AlertDetail;
                    }
                }
                View::Rules => {
                    self.input_mode = InputMode::RuleEdit;
                }
                _ => {}
            },

            KeyCode::Char('a') => {
                if self.view == View::Alerts {
                    if let Some(alert) = self.alerts.get_mut(self.selected) {
                        alert.acknowledged = true;
                    }
                }
            }

            KeyCode::Char('d') => {
                if self.view == View::Alerts && !self.alerts.is_empty() {
                    self.alerts.remove(self.selected);
                    if self.selected > 0 && self.selected >= self.alerts.len() {
                        self.selected = self.alerts.len().saturating_sub(1);
                    }
                }
            }

            KeyCode::Char('t') => {
                self.view = View::Training;
            }

            KeyCode::Char('e') => {
                // Export would be implemented here
            }

            _ => {}
        }

        false
    }

    fn handle_alert_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
                self.selected_alert = None;
            }
            KeyCode::Char('a') => {
                if let Some(idx) = self.selected_alert {
                    if let Some(alert) = self.alerts.get_mut(idx) {
                        alert.acknowledged = true;
                    }
                }
                self.input_mode = InputMode::Normal;
                self.selected_alert = None;
            }
            _ => {}
        }
        false
    }

    fn handle_rule_edit_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(' ') => {
                // Toggle rule enabled
                if let Some(rule) = self.rules.get_mut(self.selected) {
                    rule.enabled = !rule.enabled;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        false
    }

    fn move_selection(&mut self, delta: i32) {
        let count = self.item_count();
        if count == 0 {
            return;
        }

        let new_selected = if delta > 0 {
            (self.selected + delta as usize).min(count - 1)
        } else {
            self.selected.saturating_sub((-delta) as usize)
        };

        self.selected = new_selected;
    }

    fn item_count(&self) -> usize {
        match self.view {
            View::Dashboard => 0,
            View::Alerts => self.alerts.len(),
            View::Rules => self.rules.len(),
            View::Training => 0,
        }
    }

    pub fn active_alerts_count(&self) -> usize {
        self.alerts.iter().filter(|a| !a.acknowledged).count()
    }

    pub fn critical_alerts_count(&self) -> usize {
        self.alerts
            .iter()
            .filter(|a| !a.acknowledged && a.severity == Severity::Critical)
            .count()
    }

    pub fn status_text(&self) -> String {
        let active = self.active_alerts_count();
        let critical = self.critical_alerts_count();

        if critical > 0 {
            format!(
                "{} active alerts ({} critical) | {} lines scanned",
                active, critical, self.lines_scanned
            )
        } else if active > 0 {
            format!(
                "{} active alerts | {} lines scanned",
                active, self.lines_scanned
            )
        } else {
            format!("No active alerts | {} lines scanned", self.lines_scanned)
        }
    }
}
