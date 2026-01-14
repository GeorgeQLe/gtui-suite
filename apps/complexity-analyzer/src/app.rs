use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub lines: u32,
    pub params: u32,
}

impl FunctionMetrics {
    pub fn complexity_level(&self) -> ComplexityLevel {
        if self.cyclomatic > 20 || self.cognitive > 25 {
            ComplexityLevel::High
        } else if self.cyclomatic > 10 || self.cognitive > 15 {
            ComplexityLevel::Medium
        } else {
            ComplexityLevel::Low
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
}

impl ComplexityLevel {
    pub fn name(&self) -> &'static str {
        match self {
            ComplexityLevel::Low => "Low",
            ComplexityLevel::Medium => "Medium",
            ComplexityLevel::High => "High",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Cyclomatic,
    Cognitive,
    Lines,
    Name,
}

pub struct App {
    pub functions: Vec<FunctionMetrics>,
    pub selected: usize,
    pub sort_by: SortBy,
    pub filter_level: Option<ComplexityLevel>,
    pub filtered_indices: Vec<usize>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let functions = create_demo_metrics();
        let filtered_indices: Vec<usize> = (0..functions.len()).collect();

        let mut app = Self {
            functions,
            selected: 0,
            sort_by: SortBy::Cyclomatic,
            filter_level: None,
            filtered_indices,
            status_message: None,
        };
        app.sort_and_filter();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.filtered_indices.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('c') => {
                self.sort_by = SortBy::Cyclomatic;
                self.sort_and_filter();
                self.status_message = Some("Sorted by cyclomatic complexity".to_string());
            }
            KeyCode::Char('o') => {
                self.sort_by = SortBy::Cognitive;
                self.sort_and_filter();
                self.status_message = Some("Sorted by cognitive complexity".to_string());
            }
            KeyCode::Char('l') => {
                self.sort_by = SortBy::Lines;
                self.sort_and_filter();
                self.status_message = Some("Sorted by lines".to_string());
            }
            KeyCode::Char('n') => {
                self.sort_by = SortBy::Name;
                self.sort_and_filter();
                self.status_message = Some("Sorted by name".to_string());
            }
            KeyCode::Char('1') => {
                self.filter_level = Some(ComplexityLevel::High);
                self.sort_and_filter();
                self.status_message = Some("Filter: High complexity".to_string());
            }
            KeyCode::Char('2') => {
                self.filter_level = Some(ComplexityLevel::Medium);
                self.sort_and_filter();
                self.status_message = Some("Filter: Medium+ complexity".to_string());
            }
            KeyCode::Char('0') => {
                self.filter_level = None;
                self.sort_and_filter();
                self.status_message = Some("Filter: All".to_string());
            }
            _ => {}
        }
        false
    }

    fn sort_and_filter(&mut self) {
        self.filtered_indices = self.functions
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                match self.filter_level {
                    Some(ComplexityLevel::High) => f.complexity_level() == ComplexityLevel::High,
                    Some(ComplexityLevel::Medium) => f.complexity_level() != ComplexityLevel::Low,
                    _ => true,
                }
            })
            .map(|(i, _)| i)
            .collect();

        self.filtered_indices.sort_by(|&a, &b| {
            let fa = &self.functions[a];
            let fb = &self.functions[b];
            match self.sort_by {
                SortBy::Cyclomatic => fb.cyclomatic.cmp(&fa.cyclomatic),
                SortBy::Cognitive => fb.cognitive.cmp(&fa.cognitive),
                SortBy::Lines => fb.lines.cmp(&fa.lines),
                SortBy::Name => fa.name.cmp(&fb.name),
            }
        });

        self.selected = self.selected.min(self.filtered_indices.len().saturating_sub(1));
    }

    pub fn high_complexity_count(&self) -> usize {
        self.functions.iter().filter(|f| f.complexity_level() == ComplexityLevel::High).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav c:cyclomatic o:cognitive l:lines n:name 1:high 2:medium 0:all q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_metrics() -> Vec<FunctionMetrics> {
    vec![
        FunctionMetrics {
            name: "process_request".to_string(),
            file: "src/handler.rs".to_string(),
            line: 45,
            cyclomatic: 25,
            cognitive: 30,
            lines: 120,
            params: 5,
        },
        FunctionMetrics {
            name: "parse_config".to_string(),
            file: "src/config.rs".to_string(),
            line: 12,
            cyclomatic: 15,
            cognitive: 18,
            lines: 80,
            params: 2,
        },
        FunctionMetrics {
            name: "validate_input".to_string(),
            file: "src/validator.rs".to_string(),
            line: 100,
            cyclomatic: 12,
            cognitive: 14,
            lines: 45,
            params: 3,
        },
        FunctionMetrics {
            name: "simple_helper".to_string(),
            file: "src/utils.rs".to_string(),
            line: 20,
            cyclomatic: 2,
            cognitive: 1,
            lines: 10,
            params: 1,
        },
        FunctionMetrics {
            name: "format_output".to_string(),
            file: "src/formatter.rs".to_string(),
            line: 55,
            cyclomatic: 8,
            cognitive: 10,
            lines: 35,
            params: 4,
        },
        FunctionMetrics {
            name: "main_loop".to_string(),
            file: "src/main.rs".to_string(),
            line: 10,
            cyclomatic: 18,
            cognitive: 22,
            lines: 95,
            params: 0,
        },
    ]
}
