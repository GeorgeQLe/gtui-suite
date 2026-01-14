use chrono::{FixedOffset, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct TimeZone {
    pub name: String,
    pub city: String,
    pub offset_hours: i32,
    pub enabled: bool,
}

impl TimeZone {
    pub fn current_time(&self) -> String {
        let offset = FixedOffset::east_opt(self.offset_hours * 3600).unwrap();
        let now = Utc::now().with_timezone(&offset);
        now.format("%H:%M:%S").to_string()
    }

    pub fn current_date(&self) -> String {
        let offset = FixedOffset::east_opt(self.offset_hours * 3600).unwrap();
        let now = Utc::now().with_timezone(&offset);
        now.format("%Y-%m-%d").to_string()
    }

    pub fn offset_str(&self) -> String {
        if self.offset_hours >= 0 {
            format!("UTC+{}", self.offset_hours)
        } else {
            format!("UTC{}", self.offset_hours)
        }
    }
}

pub struct App {
    pub timezones: Vec<TimeZone>,
    pub selected: usize,
    pub show_24h: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            timezones: create_default_timezones(),
            selected: 0,
            show_24h: true,
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
                if self.selected < self.timezones.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(tz) = self.timezones.get_mut(self.selected) {
                    tz.enabled = !tz.enabled;
                }
            }
            KeyCode::Char('f') => {
                self.show_24h = !self.show_24h;
                self.status_message = Some(format!(
                    "Format: {}",
                    if self.show_24h { "24-hour" } else { "12-hour" }
                ));
            }
            KeyCode::Char('a') => {
                for tz in &mut self.timezones {
                    tz.enabled = true;
                }
                self.status_message = Some("All timezones enabled".to_string());
            }
            KeyCode::Char('n') => {
                for tz in &mut self.timezones {
                    tz.enabled = false;
                }
                self.status_message = Some("All timezones disabled".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn tick(&mut self) {
        // Time updates automatically on each render
    }

    pub fn enabled_count(&self) -> usize {
        self.timezones.iter().filter(|tz| tz.enabled).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav Space:toggle f:format a:all n:none q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_default_timezones() -> Vec<TimeZone> {
    vec![
        TimeZone {
            name: "UTC".to_string(),
            city: "Coordinated Universal Time".to_string(),
            offset_hours: 0,
            enabled: true,
        },
        TimeZone {
            name: "EST".to_string(),
            city: "New York".to_string(),
            offset_hours: -5,
            enabled: true,
        },
        TimeZone {
            name: "PST".to_string(),
            city: "Los Angeles".to_string(),
            offset_hours: -8,
            enabled: true,
        },
        TimeZone {
            name: "GMT".to_string(),
            city: "London".to_string(),
            offset_hours: 0,
            enabled: true,
        },
        TimeZone {
            name: "CET".to_string(),
            city: "Paris".to_string(),
            offset_hours: 1,
            enabled: true,
        },
        TimeZone {
            name: "IST".to_string(),
            city: "Mumbai".to_string(),
            offset_hours: 5,
            enabled: false,
        },
        TimeZone {
            name: "CST".to_string(),
            city: "Beijing".to_string(),
            offset_hours: 8,
            enabled: true,
        },
        TimeZone {
            name: "JST".to_string(),
            city: "Tokyo".to_string(),
            offset_hours: 9,
            enabled: true,
        },
        TimeZone {
            name: "AEST".to_string(),
            city: "Sydney".to_string(),
            offset_hours: 10,
            enabled: false,
        },
        TimeZone {
            name: "NZST".to_string(),
            city: "Auckland".to_string(),
            offset_hours: 12,
            enabled: false,
        },
    ]
}
