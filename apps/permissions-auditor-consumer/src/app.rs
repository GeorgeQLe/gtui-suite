use crossterm::event::{KeyCode, KeyEvent};

use crate::auditor::{Auditor, Finding, Severity};
use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Findings,
    Detail,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    All,
    Critical,
    High,
    Medium,
    Low,
}

impl FilterType {
    pub fn next(&self) -> Self {
        match self {
            FilterType::All => FilterType::Critical,
            FilterType::Critical => FilterType::High,
            FilterType::High => FilterType::Medium,
            FilterType::Medium => FilterType::Low,
            FilterType::Low => FilterType::All,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FilterType::All => "All",
            FilterType::Critical => "Critical",
            FilterType::High => "High",
            FilterType::Medium => "Medium",
            FilterType::Low => "Low",
        }
    }
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub findings: Vec<Finding>,
    pub selected: usize,
    pub scroll: usize,
    pub filter: FilterType,
    pub scanning: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            view: View::Findings,
            findings: Vec::new(),
            selected: 0,
            scroll: 0,
            filter: FilterType::All,
            scanning: false,
            message: None,
            error: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match self.view {
            View::Findings => self.handle_findings_key(key),
            View::Detail => self.handle_detail_key(key),
            View::Help => self.handle_help_key(key),
        }
    }

    fn handle_findings_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                let filtered = self.filtered_findings();
                if self.selected < filtered.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Enter => {
                if !self.filtered_findings().is_empty() {
                    self.view = View::Detail;
                }
            }

            // Actions
            KeyCode::Char('s') => {
                self.start_scan();
            }
            KeyCode::Char('i') => {
                self.ignore_selected();
            }
            KeyCode::Char('f') => {
                self.show_fix_command();
            }
            KeyCode::Tab => {
                self.filter = self.filter.next();
                self.selected = 0;
            }
            KeyCode::Char('c') => {
                // Clear findings
                self.findings.clear();
                self.selected = 0;
            }

            _ => {}
        }
        false
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.view = View::Findings;
            }
            KeyCode::Char('f') => {
                self.show_fix_command();
            }
            KeyCode::Char('i') => {
                self.ignore_selected();
                self.view = View::Findings;
            }
            _ => {}
        }
        false
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Char('?') => {
                self.view = View::Findings;
            }
            _ => {}
        }
        false
    }

    fn start_scan(&mut self) {
        self.scanning = true;
        self.message = Some("Scanning...".to_string());

        let auditor = Auditor::new(self.config.clone());
        self.findings = auditor.scan();
        self.selected = 0;
        self.scanning = false;

        let count = self.findings.len();
        let critical = self.findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let high = self.findings.iter().filter(|f| f.severity == Severity::High).count();

        self.message = Some(format!(
            "Scan complete: {} findings ({} critical, {} high)",
            count, critical, high
        ));
    }

    fn ignore_selected(&mut self) {
        let filtered = self.filtered_finding_indices();
        if let Some(&idx) = filtered.get(self.selected) {
            self.findings[idx].ignored = true;
            self.message = Some("Finding ignored".to_string());
        }
    }

    fn show_fix_command(&mut self) {
        let filtered = self.filtered_finding_indices();
        if let Some(&idx) = filtered.get(self.selected) {
            if let Some(cmd) = &self.findings[idx].fix_command {
                self.message = Some(format!("Fix: {}", cmd));
            } else {
                self.message = Some("No fix command available".to_string());
            }
        }
    }

    pub fn filtered_findings(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| !f.ignored)
            .filter(|f| match self.filter {
                FilterType::All => true,
                FilterType::Critical => f.severity == Severity::Critical,
                FilterType::High => f.severity == Severity::High,
                FilterType::Medium => f.severity == Severity::Medium,
                FilterType::Low => f.severity == Severity::Low,
            })
            .collect()
    }

    fn filtered_finding_indices(&self) -> Vec<usize> {
        self.findings
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.ignored)
            .filter(|(_, f)| match self.filter {
                FilterType::All => true,
                FilterType::Critical => f.severity == Severity::Critical,
                FilterType::High => f.severity == Severity::High,
                FilterType::Medium => f.severity == Severity::Medium,
                FilterType::Low => f.severity == Severity::Low,
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn selected_finding(&self) -> Option<&Finding> {
        let filtered = self.filtered_finding_indices();
        filtered.get(self.selected).map(|&idx| &self.findings[idx])
    }

    pub fn summary(&self) -> Summary {
        let active: Vec<_> = self.findings.iter().filter(|f| !f.ignored).collect();
        Summary {
            total: active.len(),
            critical: active.iter().filter(|f| f.severity == Severity::Critical).count(),
            high: active.iter().filter(|f| f.severity == Severity::High).count(),
            medium: active.iter().filter(|f| f.severity == Severity::Medium).count(),
            low: active.iter().filter(|f| f.severity == Severity::Low).count(),
        }
    }
}

pub struct Summary {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}
