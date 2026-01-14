use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Pattern,
    TestString,
    Replacement,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub groups: Vec<(usize, usize, String)>,
}

pub struct App {
    pub pattern: String,
    pub test_string: String,
    pub replacement: String,
    pub active_field: InputField,
    pub matches: Vec<Match>,
    pub error: Option<String>,
    pub case_insensitive: bool,
    pub multiline: bool,
    pub dot_matches_newline: bool,
    pub replaced_text: Option<String>,
    pub history: Vec<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            pattern: String::new(),
            test_string: "Hello, World!\nTest 123\nfoo@example.com".to_string(),
            replacement: String::new(),
            active_field: InputField::Pattern,
            matches: Vec::new(),
            error: None,
            case_insensitive: false,
            multiline: false,
            dot_matches_newline: false,
            replaced_text: None,
            history: Vec::new(),
            status_message: None,
        };
        app.update_matches();
        app
    }

    pub fn update_matches(&mut self) {
        self.matches.clear();
        self.error = None;
        self.replaced_text = None;

        if self.pattern.is_empty() {
            return;
        }

        let pattern = self.build_pattern();
        match Regex::new(&pattern) {
            Ok(re) => {
                for caps in re.captures_iter(&self.test_string) {
                    let full_match = caps.get(0).unwrap();
                    let mut groups = Vec::new();

                    for i in 1..caps.len() {
                        if let Some(g) = caps.get(i) {
                            groups.push((g.start(), g.end(), g.as_str().to_string()));
                        }
                    }

                    self.matches.push(Match {
                        start: full_match.start(),
                        end: full_match.end(),
                        text: full_match.as_str().to_string(),
                        groups,
                    });
                }

                if !self.replacement.is_empty() {
                    self.replaced_text = Some(re.replace_all(&self.test_string, &self.replacement).to_string());
                }
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
    }

    fn build_pattern(&self) -> String {
        let mut flags = String::new();

        if self.case_insensitive {
            flags.push('i');
        }
        if self.multiline {
            flags.push('m');
        }
        if self.dot_matches_newline {
            flags.push('s');
        }

        if flags.is_empty() {
            self.pattern.clone()
        } else {
            format!("(?{}){}", flags, self.pattern)
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => return true,
                KeyCode::Char('i') => {
                    self.case_insensitive = !self.case_insensitive;
                    self.update_matches();
                    return false;
                }
                KeyCode::Char('m') => {
                    self.multiline = !self.multiline;
                    self.update_matches();
                    return false;
                }
                KeyCode::Char('s') => {
                    self.dot_matches_newline = !self.dot_matches_newline;
                    self.update_matches();
                    return false;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Tab => {
                self.active_field = match self.active_field {
                    InputField::Pattern => InputField::TestString,
                    InputField::TestString => InputField::Replacement,
                    InputField::Replacement => InputField::Pattern,
                };
            }
            KeyCode::BackTab => {
                self.active_field = match self.active_field {
                    InputField::Pattern => InputField::Replacement,
                    InputField::TestString => InputField::Pattern,
                    InputField::Replacement => InputField::TestString,
                };
            }
            KeyCode::Backspace => {
                let field = self.active_field_mut();
                field.pop();
                self.update_matches();
            }
            KeyCode::Char(c) => {
                let field = self.active_field_mut();
                field.push(c);
                self.update_matches();
            }
            KeyCode::Enter => {
                if self.active_field == InputField::TestString {
                    self.test_string.push('\n');
                    self.update_matches();
                }
            }
            _ => {}
        }
        false
    }

    fn active_field_mut(&mut self) -> &mut String {
        match self.active_field {
            InputField::Pattern => &mut self.pattern,
            InputField::TestString => &mut self.test_string,
            InputField::Replacement => &mut self.replacement,
        }
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    pub fn group_count(&self) -> usize {
        self.matches.first().map(|m| m.groups.len()).unwrap_or(0)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        format!(
            "Tab:switch fields | Ctrl+i:case({}) Ctrl+m:multiline({}) Ctrl+s:dotall({})",
            if self.case_insensitive { "on" } else { "off" },
            if self.multiline { "on" } else { "off" },
            if self.dot_matches_newline { "on" } else { "off" }
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
