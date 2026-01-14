use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Table,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharRange {
    Control,    // 0-31
    Printable,  // 32-126
    Extended,   // 127-255
    All,
}

impl CharRange {
    pub fn name(&self) -> &'static str {
        match self {
            CharRange::Control => "Control (0-31)",
            CharRange::Printable => "Printable (32-126)",
            CharRange::Extended => "Extended (127-255)",
            CharRange::All => "All (0-255)",
        }
    }

    pub fn range(&self) -> std::ops::Range<u16> {
        match self {
            CharRange::Control => 0..32,
            CharRange::Printable => 32..127,
            CharRange::Extended => 127..256,
            CharRange::All => 0..256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CharInfo {
    pub code: u8,
    pub char_display: String,
    pub name: String,
    pub hex: String,
    pub octal: String,
    pub binary: String,
}

impl CharInfo {
    pub fn new(code: u8) -> Self {
        let char_display = match code {
            0 => "NUL".to_string(),
            1 => "SOH".to_string(),
            2 => "STX".to_string(),
            3 => "ETX".to_string(),
            4 => "EOT".to_string(),
            5 => "ENQ".to_string(),
            6 => "ACK".to_string(),
            7 => "BEL".to_string(),
            8 => "BS".to_string(),
            9 => "TAB".to_string(),
            10 => "LF".to_string(),
            11 => "VT".to_string(),
            12 => "FF".to_string(),
            13 => "CR".to_string(),
            14 => "SO".to_string(),
            15 => "SI".to_string(),
            16 => "DLE".to_string(),
            17 => "DC1".to_string(),
            18 => "DC2".to_string(),
            19 => "DC3".to_string(),
            20 => "DC4".to_string(),
            21 => "NAK".to_string(),
            22 => "SYN".to_string(),
            23 => "ETB".to_string(),
            24 => "CAN".to_string(),
            25 => "EM".to_string(),
            26 => "SUB".to_string(),
            27 => "ESC".to_string(),
            28 => "FS".to_string(),
            29 => "GS".to_string(),
            30 => "RS".to_string(),
            31 => "US".to_string(),
            32 => "SPC".to_string(),
            127 => "DEL".to_string(),
            c if c >= 32 && c < 127 => (c as char).to_string(),
            c => format!("x{:02X}", c),
        };

        let name = get_char_name(code);

        Self {
            code,
            char_display,
            name,
            hex: format!("0x{:02X}", code),
            octal: format!("0o{:03o}", code),
            binary: format!("{:08b}", code),
        }
    }
}

fn get_char_name(code: u8) -> String {
    match code {
        0 => "Null".to_string(),
        1 => "Start of Heading".to_string(),
        2 => "Start of Text".to_string(),
        3 => "End of Text".to_string(),
        4 => "End of Transmission".to_string(),
        5 => "Enquiry".to_string(),
        6 => "Acknowledge".to_string(),
        7 => "Bell".to_string(),
        8 => "Backspace".to_string(),
        9 => "Horizontal Tab".to_string(),
        10 => "Line Feed".to_string(),
        11 => "Vertical Tab".to_string(),
        12 => "Form Feed".to_string(),
        13 => "Carriage Return".to_string(),
        14 => "Shift Out".to_string(),
        15 => "Shift In".to_string(),
        16 => "Data Link Escape".to_string(),
        17 => "Device Control 1".to_string(),
        18 => "Device Control 2".to_string(),
        19 => "Device Control 3".to_string(),
        20 => "Device Control 4".to_string(),
        21 => "Negative Acknowledge".to_string(),
        22 => "Synchronous Idle".to_string(),
        23 => "End of Trans. Block".to_string(),
        24 => "Cancel".to_string(),
        25 => "End of Medium".to_string(),
        26 => "Substitute".to_string(),
        27 => "Escape".to_string(),
        28 => "File Separator".to_string(),
        29 => "Group Separator".to_string(),
        30 => "Record Separator".to_string(),
        31 => "Unit Separator".to_string(),
        32 => "Space".to_string(),
        127 => "Delete".to_string(),
        c if c >= 33 && c < 127 => format!("'{}'", c as char),
        _ => "Extended ASCII".to_string(),
    }
}

pub struct App {
    pub chars: Vec<CharInfo>,
    pub selected: usize,
    pub mode: ViewMode,
    pub range: CharRange,
    pub search: String,
    pub searching: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            chars: Vec::new(),
            selected: 0,
            mode: ViewMode::Table,
            range: CharRange::All,
            search: String::new(),
            searching: false,
            status_message: None,
        };
        app.update_chars();
        app
    }

    fn update_chars(&mut self) {
        self.chars = self.range.range()
            .map(|c| CharInfo::new(c as u8))
            .collect();

        if !self.search.is_empty() {
            let search_lower = self.search.to_lowercase();
            self.chars.retain(|c| {
                c.name.to_lowercase().contains(&search_lower)
                    || c.char_display.to_lowercase().contains(&search_lower)
                    || c.hex.to_lowercase().contains(&search_lower)
                    || c.code.to_string().contains(&search_lower)
            });
        }

        self.selected = self.selected.min(self.chars.len().saturating_sub(1));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        if self.searching {
            return self.handle_search_key(key);
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.chars.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if self.selected >= 16 {
                    self.selected -= 16;
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.selected + 16 < self.chars.len() {
                    self.selected += 16;
                }
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.selected = self.chars.len().saturating_sub(1);
            }
            KeyCode::Enter => {
                self.mode = match self.mode {
                    ViewMode::Table => ViewMode::Detail,
                    ViewMode::Detail => ViewMode::Table,
                };
            }
            KeyCode::Char('1') => {
                self.range = CharRange::Control;
                self.update_chars();
            }
            KeyCode::Char('2') => {
                self.range = CharRange::Printable;
                self.update_chars();
            }
            KeyCode::Char('3') => {
                self.range = CharRange::Extended;
                self.update_chars();
            }
            KeyCode::Char('4') => {
                self.range = CharRange::All;
                self.update_chars();
            }
            KeyCode::Char('/') => {
                self.searching = true;
                self.search.clear();
            }
            KeyCode::Char('y') => {
                if let Some(c) = self.chars.get(self.selected) {
                    self.status_message = Some(format!("Copied: {} ({})", c.char_display, c.code));
                }
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.searching = false;
                self.search.clear();
                self.update_chars();
            }
            KeyCode::Enter => {
                self.searching = false;
            }
            KeyCode::Backspace => {
                self.search.pop();
                self.update_chars();
            }
            KeyCode::Char(c) => {
                self.search.push(c);
                self.update_chars();
            }
            _ => {}
        }
        false
    }

    pub fn selected_char(&self) -> Option<&CharInfo> {
        self.chars.get(self.selected)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        if self.searching {
            return format!("Search: {}_ | Esc:cancel Enter:confirm", self.search);
        }
        "j/k:nav 1-4:range /:search Enter:detail y:copy q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
