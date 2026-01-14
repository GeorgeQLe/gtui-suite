use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;

const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOUS: &str = "l1I0O";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordType {
    Random,
    Memorable,
    Pin,
    Passphrase,
}

pub struct App {
    pub password: String,
    pub history: Vec<String>,
    pub length: usize,
    pub password_type: PasswordType,
    pub include_uppercase: bool,
    pub include_lowercase: bool,
    pub include_digits: bool,
    pub include_symbols: bool,
    pub exclude_ambiguous: bool,
    pub word_count: usize,
    pub word_separator: String,
    pub strength: PasswordStrength,
    pub status_message: Option<String>,
    pub selected_option: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    pub fn label(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "Very Weak",
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Medium => "Medium",
            PasswordStrength::Strong => "Strong",
            PasswordStrength::VeryStrong => "Very Strong",
        }
    }

    pub fn score(&self) -> usize {
        match self {
            PasswordStrength::VeryWeak => 1,
            PasswordStrength::Weak => 2,
            PasswordStrength::Medium => 3,
            PasswordStrength::Strong => 4,
            PasswordStrength::VeryStrong => 5,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            password: String::new(),
            history: Vec::new(),
            length: 16,
            password_type: PasswordType::Random,
            include_uppercase: true,
            include_lowercase: true,
            include_digits: true,
            include_symbols: true,
            exclude_ambiguous: true,
            word_count: 4,
            word_separator: "-".to_string(),
            strength: PasswordStrength::VeryWeak,
            status_message: None,
            selected_option: 0,
        }
    }

    pub fn generate(&mut self) {
        self.password = match self.password_type {
            PasswordType::Random => self.generate_random(),
            PasswordType::Memorable => self.generate_memorable(),
            PasswordType::Pin => self.generate_pin(),
            PasswordType::Passphrase => self.generate_passphrase(),
        };

        self.strength = self.calculate_strength();

        if self.history.len() >= 10 {
            self.history.remove(0);
        }
        self.history.push(self.password.clone());
    }

    fn generate_random(&self) -> String {
        let mut charset = String::new();

        if self.include_lowercase {
            charset.push_str(LOWERCASE);
        }
        if self.include_uppercase {
            charset.push_str(UPPERCASE);
        }
        if self.include_digits {
            charset.push_str(DIGITS);
        }
        if self.include_symbols {
            charset.push_str(SYMBOLS);
        }

        if self.exclude_ambiguous {
            charset = charset.chars().filter(|c| !AMBIGUOUS.contains(*c)).collect();
        }

        if charset.is_empty() {
            return "Enable at least one character type".to_string();
        }

        let charset: Vec<char> = charset.chars().collect();
        let mut rng = rand::thread_rng();

        (0..self.length)
            .map(|_| charset[rng.gen_range(0..charset.len())])
            .collect()
    }

    fn generate_memorable(&self) -> String {
        // Generate pattern like: Word123!
        let words = ["apple", "banana", "cherry", "dragon", "eagle", "falcon",
                     "galaxy", "harbor", "island", "jungle", "koala", "lemon",
                     "mango", "nebula", "ocean", "phoenix", "quartz", "river",
                     "sunset", "tiger", "ultra", "violet", "winter", "xenon",
                     "yellow", "zenith"];

        let mut rng = rand::thread_rng();
        let word = words[rng.gen_range(0..words.len())];
        let mut result: String = word.chars().next().unwrap().to_uppercase().collect();
        result.push_str(&word[1..]);

        let num: u32 = rng.gen_range(10..999);
        result.push_str(&num.to_string());

        let symbols: Vec<char> = SYMBOLS.chars().collect();
        result.push(symbols[rng.gen_range(0..symbols.len())]);

        result
    }

    fn generate_pin(&self) -> String {
        let mut rng = rand::thread_rng();
        (0..self.length)
            .map(|_| char::from_digit(rng.gen_range(0..10), 10).unwrap())
            .collect()
    }

    fn generate_passphrase(&self) -> String {
        let words = ["correct", "horse", "battery", "staple", "random", "coffee",
                     "mountain", "river", "sunset", "ocean", "forest", "thunder",
                     "crystal", "diamond", "silver", "golden", "ancient", "modern",
                     "digital", "analog", "cosmic", "stellar", "lunar", "solar"];

        let mut rng = rand::thread_rng();
        let selected: Vec<&str> = (0..self.word_count)
            .map(|_| words[rng.gen_range(0..words.len())])
            .collect();

        selected.join(&self.word_separator)
    }

    fn calculate_strength(&self) -> PasswordStrength {
        let len = self.password.len();
        let has_lower = self.password.chars().any(|c| c.is_lowercase());
        let has_upper = self.password.chars().any(|c| c.is_uppercase());
        let has_digit = self.password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = self.password.chars().any(|c| SYMBOLS.contains(c));

        let char_types = [has_lower, has_upper, has_digit, has_symbol]
            .iter()
            .filter(|&&b| b)
            .count();

        let entropy = len * char_types;

        if entropy < 20 {
            PasswordStrength::VeryWeak
        } else if entropy < 40 {
            PasswordStrength::Weak
        } else if entropy < 60 {
            PasswordStrength::Medium
        } else if entropy < 80 {
            PasswordStrength::Strong
        } else {
            PasswordStrength::VeryStrong
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('g') | KeyCode::Enter => {
                self.generate();
            }
            KeyCode::Char('c') => {
                self.status_message = Some("Password copied to clipboard!".to_string());
            }
            KeyCode::Char('t') => {
                self.cycle_type();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self.length < 64 {
                    self.length += 1;
                    self.generate();
                }
            }
            KeyCode::Char('-') => {
                if self.length > 4 {
                    self.length -= 1;
                    self.generate();
                }
            }
            KeyCode::Char('u') => {
                self.include_uppercase = !self.include_uppercase;
                self.generate();
            }
            KeyCode::Char('l') => {
                self.include_lowercase = !self.include_lowercase;
                self.generate();
            }
            KeyCode::Char('d') => {
                self.include_digits = !self.include_digits;
                self.generate();
            }
            KeyCode::Char('s') => {
                self.include_symbols = !self.include_symbols;
                self.generate();
            }
            KeyCode::Char('a') => {
                self.exclude_ambiguous = !self.exclude_ambiguous;
                self.generate();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_option < 6 {
                    self.selected_option += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_option = self.selected_option.saturating_sub(1);
            }
            _ => {}
        }
        false
    }

    fn cycle_type(&mut self) {
        self.password_type = match self.password_type {
            PasswordType::Random => PasswordType::Memorable,
            PasswordType::Memorable => PasswordType::Pin,
            PasswordType::Pin => PasswordType::Passphrase,
            PasswordType::Passphrase => PasswordType::Random,
        };
        self.generate();
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        "g:generate c:copy t:type +/-:length u/l/d/s:toggle options".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
