use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoType {
    Todo,
    Fixme,
    Hack,
    Note,
    Bug,
    Xxx,
}

impl TodoType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TodoType::Todo => "TODO",
            TodoType::Fixme => "FIXME",
            TodoType::Hack => "HACK",
            TodoType::Note => "NOTE",
            TodoType::Bug => "BUG",
            TodoType::Xxx => "XXX",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub todo_type: TodoType,
    pub file: String,
    pub line: usize,
    pub content: String,
    pub context: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
}

pub struct App {
    pub items: Vec<TodoItem>,
    pub selected: usize,
    pub view: View,
    pub filter: Option<TodoType>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            view: View::List,
            filter: None,
            status_message: None,
        }
    }

    pub fn scan_directory(&mut self) {
        self.items = create_demo_items();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Detail => self.handle_detail_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let visible = self.visible_items();
                if self.selected < visible.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.view = View::Detail;
            }
            KeyCode::Char('1') => {
                self.toggle_filter(TodoType::Todo);
            }
            KeyCode::Char('2') => {
                self.toggle_filter(TodoType::Fixme);
            }
            KeyCode::Char('3') => {
                self.toggle_filter(TodoType::Hack);
            }
            KeyCode::Char('4') => {
                self.toggle_filter(TodoType::Note);
            }
            KeyCode::Char('5') => {
                self.toggle_filter(TodoType::Bug);
            }
            KeyCode::Char('0') => {
                self.filter = None;
                self.selected = 0;
                self.status_message = Some("Showing all".to_string());
            }
            KeyCode::Char('r') => {
                self.scan_directory();
                self.status_message = Some("Rescanned".to_string());
            }
            KeyCode::Char('o') => {
                if let Some(item) = self.selected_item() {
                    self.status_message = Some(format!("Opening {}:{}...", item.file, item.line));
                }
            }
            _ => {}
        }
        false
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::List;
            }
            KeyCode::Char('o') => {
                if let Some(item) = self.selected_item() {
                    self.status_message = Some(format!("Opening {}:{}...", item.file, item.line));
                }
            }
            _ => {}
        }
        false
    }

    fn toggle_filter(&mut self, todo_type: TodoType) {
        if self.filter == Some(todo_type) {
            self.filter = None;
            self.status_message = Some("Showing all".to_string());
        } else {
            self.filter = Some(todo_type);
            self.status_message = Some(format!("Filtering: {}", todo_type.as_str()));
        }
        self.selected = 0;
    }

    pub fn visible_items(&self) -> Vec<&TodoItem> {
        if let Some(filter) = self.filter {
            self.items
                .iter()
                .filter(|item| item.todo_type == filter)
                .collect()
        } else {
            self.items.iter().collect()
        }
    }

    pub fn selected_item(&self) -> Option<&TodoItem> {
        self.visible_items().get(self.selected).copied()
    }

    pub fn count_by_type(&self, todo_type: TodoType) -> usize {
        self.items
            .iter()
            .filter(|item| item.todo_type == todo_type)
            .count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        let visible = self.visible_items().len();
        format!(
            "{} items | Enter:detail o:open 1:TODO 2:FIXME 3:HACK 4:NOTE 5:BUG 0:all",
            visible
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_items() -> Vec<TodoItem> {
    vec![
        TodoItem {
            todo_type: TodoType::Todo,
            file: "src/main.rs".to_string(),
            line: 42,
            content: "Add error handling for edge cases".to_string(),
            context: "fn process_data() {\n    // TODO: Add error handling for edge cases\n    data.process();\n}".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Fixme,
            file: "src/lib.rs".to_string(),
            line: 128,
            content: "This causes a memory leak under certain conditions".to_string(),
            context: "fn allocate() {\n    // FIXME: This causes a memory leak under certain conditions\n    let ptr = unsafe { alloc(layout) };\n}".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Hack,
            file: "src/utils.rs".to_string(),
            line: 55,
            content: "Temporary workaround until upstream fixes the bug".to_string(),
            context: "// HACK: Temporary workaround until upstream fixes the bug\nlet value = value + 1;".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Note,
            file: "src/config.rs".to_string(),
            line: 12,
            content: "This default value is intentional for backwards compatibility".to_string(),
            context: "// NOTE: This default value is intentional for backwards compatibility\nconst DEFAULT_TIMEOUT: u64 = 30;".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Bug,
            file: "src/parser.rs".to_string(),
            line: 89,
            content: "Off-by-one error when input is empty".to_string(),
            context: "// BUG: Off-by-one error when input is empty\nfor i in 0..len {\n    process(data[i]);\n}".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Todo,
            file: "src/api.rs".to_string(),
            line: 200,
            content: "Implement rate limiting".to_string(),
            context: "async fn handle_request(req: Request) {\n    // TODO: Implement rate limiting\n    process(req).await\n}".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Fixme,
            file: "src/database.rs".to_string(),
            line: 75,
            content: "Transaction not properly rolled back on error".to_string(),
            context: "fn save_data(data: &Data) {\n    // FIXME: Transaction not properly rolled back on error\n    db.execute(query)\n}".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Xxx,
            file: "src/auth.rs".to_string(),
            line: 33,
            content: "Security review needed".to_string(),
            context: "fn verify_token(token: &str) {\n    // XXX: Security review needed\n    token.parse()\n}".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Todo,
            file: "tests/integration.rs".to_string(),
            line: 15,
            content: "Add more test cases for edge conditions".to_string(),
            context: "#[test]\nfn test_basic() {\n    // TODO: Add more test cases for edge conditions\n    assert!(true);\n}".to_string(),
        },
        TodoItem {
            todo_type: TodoType::Hack,
            file: "src/renderer.rs".to_string(),
            line: 156,
            content: "Force flush to prevent rendering glitch".to_string(),
            context: "fn render_frame() {\n    // HACK: Force flush to prevent rendering glitch\n    buffer.flush();\n    buffer.flush();\n}".to_string(),
        },
    ]
}
