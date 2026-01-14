use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Encode,
    Decode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Output,
}

pub struct App {
    pub input: String,
    pub output: String,
    pub mode: Mode,
    pub focus: Focus,
    pub error: Option<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            mode: Mode::Encode,
            focus: Focus::Input,
            error: None,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Tab => {
                self.mode = match self.mode {
                    Mode::Encode => Mode::Decode,
                    Mode::Decode => Mode::Encode,
                };
                self.process();
                self.status_message = Some(format!("Mode: {:?}", self.mode));
            }
            KeyCode::Esc => {
                self.input.clear();
                self.output.clear();
                self.error = None;
                self.status_message = Some("Cleared".to_string());
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.process();
            }
            KeyCode::Enter => {
                self.input.push('\n');
                self.process();
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.status_message = Some("Copied output to clipboard".to_string());
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Swap input and output
                std::mem::swap(&mut self.input, &mut self.output);
                self.mode = match self.mode {
                    Mode::Encode => Mode::Decode,
                    Mode::Decode => Mode::Encode,
                };
                self.process();
                self.status_message = Some("Swapped".to_string());
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.process();
            }
            _ => {}
        }
        false
    }

    fn process(&mut self) {
        self.error = None;

        if self.input.is_empty() {
            self.output.clear();
            return;
        }

        match self.mode {
            Mode::Encode => {
                self.output = base64_encode(&self.input);
            }
            Mode::Decode => {
                match base64_decode(&self.input) {
                    Ok(decoded) => self.output = decoded,
                    Err(e) => {
                        self.error = Some(e);
                        self.output.clear();
                    }
                }
            }
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        format!(
            "{:?} | Tab:toggle Esc:clear Ctrl+Y:copy Ctrl+S:swap Ctrl+Q:quit",
            self.mode
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// Simple base64 implementation
const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }

        let n = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

        result.push(BASE64_CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(BASE64_CHARS[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

fn base64_decode(input: &str) -> Result<String, String> {
    let input = input.trim().replace(['\n', '\r', ' '], "");

    if input.is_empty() {
        return Ok(String::new());
    }

    if input.len() % 4 != 0 {
        return Err("Invalid base64 length".to_string());
    }

    let mut result = Vec::new();

    for chunk in input.as_bytes().chunks(4) {
        let mut buf = [0u8; 4];
        let mut pad_count = 0;

        for (i, &c) in chunk.iter().enumerate() {
            if c == b'=' {
                pad_count += 1;
                buf[i] = 0;
            } else if let Some(pos) = BASE64_CHARS.iter().position(|&x| x == c) {
                buf[i] = pos as u8;
            } else {
                return Err(format!("Invalid character: {}", c as char));
            }
        }

        let n = ((buf[0] as u32) << 18)
            | ((buf[1] as u32) << 12)
            | ((buf[2] as u32) << 6)
            | (buf[3] as u32);

        result.push(((n >> 16) & 0xFF) as u8);
        if pad_count < 2 {
            result.push(((n >> 8) & 0xFF) as u8);
        }
        if pad_count < 1 {
            result.push((n & 0xFF) as u8);
        }
    }

    String::from_utf8(result).map_err(|_| "Invalid UTF-8 in decoded data".to_string())
}
