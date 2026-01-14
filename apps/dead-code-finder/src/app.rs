use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadCodeType {
    UnusedFunction,
    UnusedVariable,
    UnusedImport,
    UnusedStruct,
    UnusedEnum,
    UnusedConst,
}

impl DeadCodeType {
    pub fn name(&self) -> &'static str {
        match self {
            DeadCodeType::UnusedFunction => "Function",
            DeadCodeType::UnusedVariable => "Variable",
            DeadCodeType::UnusedImport => "Import",
            DeadCodeType::UnusedStruct => "Struct",
            DeadCodeType::UnusedEnum => "Enum",
            DeadCodeType::UnusedConst => "Const",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            DeadCodeType::UnusedFunction => "fn",
            DeadCodeType::UnusedVariable => "let",
            DeadCodeType::UnusedImport => "use",
            DeadCodeType::UnusedStruct => "struct",
            DeadCodeType::UnusedEnum => "enum",
            DeadCodeType::UnusedConst => "const",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeadCodeItem {
    pub name: String,
    pub code_type: DeadCodeType,
    pub file: String,
    pub line: usize,
    pub confidence: u8,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    All,
    Functions,
    Variables,
    Imports,
    Structs,
}

pub struct App {
    pub items: Vec<DeadCodeItem>,
    pub selected: usize,
    pub filter: FilterType,
    pub filtered_indices: Vec<usize>,
    pub show_low_confidence: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let items = create_demo_items();
        let filtered_indices: Vec<usize> = (0..items.len()).collect();

        Self {
            items,
            selected: 0,
            filter: FilterType::All,
            filtered_indices,
            show_low_confidence: true,
            status_message: None,
        }
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
            KeyCode::Char('1') => {
                self.filter = FilterType::All;
                self.apply_filter();
                self.status_message = Some("Filter: All".to_string());
            }
            KeyCode::Char('2') => {
                self.filter = FilterType::Functions;
                self.apply_filter();
                self.status_message = Some("Filter: Functions".to_string());
            }
            KeyCode::Char('3') => {
                self.filter = FilterType::Variables;
                self.apply_filter();
                self.status_message = Some("Filter: Variables".to_string());
            }
            KeyCode::Char('4') => {
                self.filter = FilterType::Imports;
                self.apply_filter();
                self.status_message = Some("Filter: Imports".to_string());
            }
            KeyCode::Char('5') => {
                self.filter = FilterType::Structs;
                self.apply_filter();
                self.status_message = Some("Filter: Structs/Enums".to_string());
            }
            KeyCode::Char('c') => {
                self.show_low_confidence = !self.show_low_confidence;
                self.apply_filter();
                self.status_message = Some(if self.show_low_confidence {
                    "Showing all confidence levels".to_string()
                } else {
                    "Showing high confidence only".to_string()
                });
            }
            KeyCode::Char('d') => {
                if let Some(&idx) = self.filtered_indices.get(self.selected) {
                    let item = &self.items[idx];
                    self.status_message = Some(format!("Would delete: {} in {}", item.name, item.file));
                }
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Re-scanning project...".to_string());
            }
            _ => {}
        }
        false
    }

    fn apply_filter(&mut self) {
        self.filtered_indices = self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                let type_match = match self.filter {
                    FilterType::All => true,
                    FilterType::Functions => item.code_type == DeadCodeType::UnusedFunction,
                    FilterType::Variables => item.code_type == DeadCodeType::UnusedVariable,
                    FilterType::Imports => item.code_type == DeadCodeType::UnusedImport,
                    FilterType::Structs => matches!(item.code_type, DeadCodeType::UnusedStruct | DeadCodeType::UnusedEnum),
                };

                let confidence_match = self.show_low_confidence || item.confidence >= 80;

                type_match && confidence_match
            })
            .map(|(i, _)| i)
            .collect();

        self.selected = self.selected.min(self.filtered_indices.len().saturating_sub(1));
    }

    pub fn total_items(&self) -> usize {
        self.items.len()
    }

    pub fn high_confidence_count(&self) -> usize {
        self.items.iter().filter(|i| i.confidence >= 80).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav 1-5:filter c:confidence d:delete r:rescan q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_items() -> Vec<DeadCodeItem> {
    vec![
        DeadCodeItem {
            name: "unused_helper".to_string(),
            code_type: DeadCodeType::UnusedFunction,
            file: "src/utils.rs".to_string(),
            line: 45,
            confidence: 95,
            suggestion: "Remove function or add #[allow(dead_code)]".to_string(),
        },
        DeadCodeItem {
            name: "old_config".to_string(),
            code_type: DeadCodeType::UnusedVariable,
            file: "src/config.rs".to_string(),
            line: 12,
            confidence: 100,
            suggestion: "Remove unused variable".to_string(),
        },
        DeadCodeItem {
            name: "std::collections::BTreeSet".to_string(),
            code_type: DeadCodeType::UnusedImport,
            file: "src/main.rs".to_string(),
            line: 5,
            confidence: 100,
            suggestion: "Remove unused import".to_string(),
        },
        DeadCodeItem {
            name: "LegacyConfig".to_string(),
            code_type: DeadCodeType::UnusedStruct,
            file: "src/legacy.rs".to_string(),
            line: 20,
            confidence: 85,
            suggestion: "Remove struct or mark as deprecated".to_string(),
        },
        DeadCodeItem {
            name: "deprecated_process".to_string(),
            code_type: DeadCodeType::UnusedFunction,
            file: "src/processor.rs".to_string(),
            line: 100,
            confidence: 90,
            suggestion: "Remove deprecated function".to_string(),
        },
        DeadCodeItem {
            name: "MAX_RETRIES".to_string(),
            code_type: DeadCodeType::UnusedConst,
            file: "src/constants.rs".to_string(),
            line: 8,
            confidence: 100,
            suggestion: "Remove unused constant".to_string(),
        },
        DeadCodeItem {
            name: "OldStatus".to_string(),
            code_type: DeadCodeType::UnusedEnum,
            file: "src/types.rs".to_string(),
            line: 55,
            confidence: 75,
            suggestion: "Verify enum is not used via dynamic dispatch".to_string(),
        },
        DeadCodeItem {
            name: "temp_value".to_string(),
            code_type: DeadCodeType::UnusedVariable,
            file: "src/handler.rs".to_string(),
            line: 33,
            confidence: 100,
            suggestion: "Remove or use the variable".to_string(),
        },
        DeadCodeItem {
            name: "debug_print".to_string(),
            code_type: DeadCodeType::UnusedFunction,
            file: "src/debug.rs".to_string(),
            line: 10,
            confidence: 60,
            suggestion: "May be used in debug builds only".to_string(),
        },
        DeadCodeItem {
            name: "serde_json".to_string(),
            code_type: DeadCodeType::UnusedImport,
            file: "src/api.rs".to_string(),
            line: 3,
            confidence: 100,
            suggestion: "Remove unused crate import".to_string(),
        },
    ]
}
