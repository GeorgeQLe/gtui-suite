use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UuidVersion {
    V4Random,
    V7Timestamp,
    Ulid,
    NanoId,
}

impl UuidVersion {
    pub fn name(&self) -> &'static str {
        match self {
            UuidVersion::V4Random => "UUID v4 (Random)",
            UuidVersion::V7Timestamp => "UUID v7 (Timestamp)",
            UuidVersion::Ulid => "ULID",
            UuidVersion::NanoId => "NanoID",
        }
    }

    pub fn all() -> Vec<UuidVersion> {
        vec![
            UuidVersion::V4Random,
            UuidVersion::V7Timestamp,
            UuidVersion::Ulid,
            UuidVersion::NanoId,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedId {
    pub version: UuidVersion,
    pub value: String,
    pub timestamp: Option<u64>,
}

pub struct App {
    pub version: UuidVersion,
    pub generated: Vec<GeneratedId>,
    pub selected: usize,
    pub count: usize,
    pub uppercase: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            version: UuidVersion::V4Random,
            generated: Vec::new(),
            selected: 0,
            count: 1,
            uppercase: false,
            status_message: None,
        };
        app.generate();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('g') | KeyCode::Enter => {
                self.generate();
            }
            KeyCode::Char('1') => self.version = UuidVersion::V4Random,
            KeyCode::Char('2') => self.version = UuidVersion::V7Timestamp,
            KeyCode::Char('3') => self.version = UuidVersion::Ulid,
            KeyCode::Char('4') => self.version = UuidVersion::NanoId,
            KeyCode::Char('u') => {
                self.uppercase = !self.uppercase;
                self.update_case();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self.count < 100 {
                    self.count += 1;
                }
            }
            KeyCode::Char('-') => {
                if self.count > 1 {
                    self.count -= 1;
                }
            }
            KeyCode::Char('c') => {
                self.generated.clear();
                self.status_message = Some("Cleared all".to_string());
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.generated.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('y') => {
                if let Some(id) = self.generated.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", id.value));
                }
            }
            _ => {}
        }
        false
    }

    fn generate(&mut self) {
        for _ in 0..self.count {
            let id = self.generate_one();
            self.generated.push(id);
        }
        self.status_message = Some(format!("Generated {} {}", self.count, self.version.name()));

        // Keep last 1000
        if self.generated.len() > 1000 {
            self.generated.drain(0..self.generated.len() - 1000);
        }
    }

    fn generate_one(&self) -> GeneratedId {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let (value, timestamp) = match self.version {
            UuidVersion::V4Random => (generate_uuid_v4(), None),
            UuidVersion::V7Timestamp => (generate_uuid_v7(now), Some(now)),
            UuidVersion::Ulid => (generate_ulid(now), Some(now)),
            UuidVersion::NanoId => (generate_nanoid(), None),
        };

        let value = if self.uppercase {
            value.to_uppercase()
        } else {
            value
        };

        GeneratedId {
            version: self.version,
            value,
            timestamp,
        }
    }

    fn update_case(&mut self) {
        for id in &mut self.generated {
            if self.uppercase {
                id.value = id.value.to_uppercase();
            } else {
                id.value = id.value.to_lowercase();
            }
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        format!(
            "1-4:type g:generate +/-:count({}) u:case y:copy c:clear q:quit",
            self.count
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// Simple pseudo-random number generator
fn simple_random() -> u64 {
    static mut SEED: u64 = 0;
    unsafe {
        if SEED == 0 {
            SEED = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
        SEED ^= SEED << 13;
        SEED ^= SEED >> 7;
        SEED ^= SEED << 17;
        SEED
    }
}

fn random_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| (simple_random() & 0xFF) as u8).collect()
}

fn generate_uuid_v4() -> String {
    let bytes = random_bytes(16);
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-4{:01x}{:02x}-{:01x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6] & 0x0F, bytes[7],
        (bytes[8] & 0x3F) | 0x80 >> 4, bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

fn generate_uuid_v7(timestamp_ms: u64) -> String {
    let bytes = random_bytes(10);
    let ts_bytes = timestamp_ms.to_be_bytes();

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-7{:01x}{:02x}-{:01x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        ts_bytes[2], ts_bytes[3], ts_bytes[4], ts_bytes[5],
        ts_bytes[6], ts_bytes[7],
        bytes[0] & 0x0F, bytes[1],
        (bytes[2] & 0x3F) | 0x80 >> 4, bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9]
    )
}

fn generate_ulid(timestamp_ms: u64) -> String {
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

    let mut result = String::with_capacity(26);

    // Encode timestamp (first 10 characters)
    let mut ts = timestamp_ms;
    let mut ts_chars = vec![0u8; 10];
    for i in (0..10).rev() {
        ts_chars[i] = ALPHABET[(ts & 0x1F) as usize];
        ts >>= 5;
    }
    for c in ts_chars {
        result.push(c as char);
    }

    // Encode random (last 16 characters)
    let random = random_bytes(10);
    for byte in random {
        result.push(ALPHABET[(byte & 0x1F) as usize] as char);
    }

    result
}

fn generate_nanoid() -> String {
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_-";

    let bytes = random_bytes(21);
    bytes
        .iter()
        .map(|b| ALPHABET[(b & 0x3F) as usize] as char)
        .collect()
}
