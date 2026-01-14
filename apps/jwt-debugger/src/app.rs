use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct DecodedJwt {
    pub header: String,
    pub payload: String,
    pub signature: String,
    pub header_json: Option<serde_json::Value>,
    pub payload_json: Option<serde_json::Value>,
    pub expired: Option<bool>,
    pub exp_time: Option<String>,
}

pub struct App {
    pub input: String,
    pub decoded: Option<DecodedJwt>,
    pub error: Option<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            input: String::new(),
            decoded: None,
            error: None,
            status_message: None,
        };
        // Start with a demo JWT
        app.input = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE3MzU2ODk2MDB9.signature".to_string();
        app.decode();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Backspace => {
                self.input.pop();
                self.decode();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return true;
            }
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.clear();
                self.decoded = None;
                self.error = None;
            }
            KeyCode::Char('y') => {
                if self.decoded.is_some() {
                    self.status_message = Some("Copied decoded JWT".to_string());
                }
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.decode();
            }
            KeyCode::Enter => {
                // Just decode again
                self.decode();
            }
            _ => {}
        }
        false
    }

    fn decode(&mut self) {
        self.error = None;
        self.decoded = None;

        if self.input.is_empty() {
            return;
        }

        let parts: Vec<&str> = self.input.split('.').collect();
        if parts.len() != 3 {
            self.error = Some("Invalid JWT format: expected 3 parts separated by dots".to_string());
            return;
        }

        let header = parts[0];
        let payload = parts[1];
        let signature = parts[2];

        let header_json = decode_base64_json(header);
        let payload_json = decode_base64_json(payload);

        let (expired, exp_time) = if let Some(ref json) = payload_json {
            if let Some(exp) = json.get("exp").and_then(|e| e.as_i64()) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let is_expired = exp < now;
                let exp_str = format_timestamp(exp);
                (Some(is_expired), Some(exp_str))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        self.decoded = Some(DecodedJwt {
            header: header.to_string(),
            payload: payload.to_string(),
            signature: signature.to_string(),
            header_json,
            payload_json,
            expired,
            exp_time,
        });
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "Paste JWT token | Ctrl+L:clear y:copy q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn decode_base64_json(input: &str) -> Option<serde_json::Value> {
    // Simple base64url decode
    let input = input.replace('-', "+").replace('_', "/");
    let padding = match input.len() % 4 {
        2 => "==",
        3 => "=",
        _ => "",
    };
    let padded = format!("{}{}", input, padding);

    // Simple base64 decode (demo implementation)
    let decoded = simple_base64_decode(&padded)?;
    let json_str = String::from_utf8(decoded).ok()?;
    serde_json::from_str(&json_str).ok()
}

fn simple_base64_decode(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;

    for c in input.chars() {
        if c == '=' {
            break;
        }
        let idx = ALPHABET.iter().position(|&x| x == c as u8)?;
        buffer = (buffer << 6) | idx as u32;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Some(output)
}

fn format_timestamp(ts: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Invalid timestamp".to_string())
}
