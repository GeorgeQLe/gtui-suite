use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct App {
    pub input: String,
    pub hashes: Vec<(String, String)>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            input: String::new(),
            hashes: Vec::new(),
            selected: 0,
            status_message: None,
        };
        app.calculate_hashes();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Esc => return true,
            KeyCode::Backspace => {
                self.input.pop();
                self.calculate_hashes();
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some((name, hash)) = self.hashes.get(self.selected) {
                    self.status_message = Some(format!("Copied {} hash", name));
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.hashes.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.calculate_hashes();
            }
            KeyCode::Enter => {
                self.input.push('\n');
                self.calculate_hashes();
            }
            _ => {}
        }
        false
    }

    fn calculate_hashes(&mut self) {
        self.hashes.clear();

        if self.input.is_empty() {
            return;
        }

        // Simple hash calculations (in real app, use proper crypto libraries)
        self.hashes.push(("MD5".to_string(), simple_hash(&self.input, 32)));
        self.hashes.push(("SHA-1".to_string(), simple_hash(&self.input, 40)));
        self.hashes.push(("SHA-256".to_string(), simple_hash(&self.input, 64)));
        self.hashes.push(("SHA-512".to_string(), simple_hash(&self.input, 128)));
        self.hashes.push(("CRC32".to_string(), simple_hash(&self.input, 8)));
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "Type text to hash | j/k:select Ctrl+Y:copy Esc:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// Simple demo hash function (not cryptographically secure!)
fn simple_hash(input: &str, len: usize) -> String {
    let bytes: Vec<u8> = input.bytes().collect();
    let mut hash = vec![0u8; len / 2];
    let hash_len = hash.len();

    for (i, &b) in bytes.iter().enumerate() {
        hash[i % hash_len] ^= b;
        let next_idx = (i + 1) % hash_len;
        hash[next_idx] = hash[next_idx].wrapping_add(b);
    }

    hash.iter().map(|b| format!("{:02x}", b)).collect()
}
