use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    EpochToDate,
    DateToEpoch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampUnit {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

impl TimestampUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimestampUnit::Seconds => "seconds",
            TimestampUnit::Milliseconds => "milliseconds",
            TimestampUnit::Microseconds => "microseconds",
            TimestampUnit::Nanoseconds => "nanoseconds",
        }
    }
}

pub struct App {
    pub input: String,
    pub mode: Mode,
    pub unit: TimestampUnit,
    pub current_epoch: i64,
    pub current_datetime: DateTime<Utc>,
    pub converted_epoch: Option<i64>,
    pub converted_datetime: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            input: String::new(),
            mode: Mode::EpochToDate,
            unit: TimestampUnit::Seconds,
            current_epoch: now.timestamp(),
            current_datetime: now,
            converted_epoch: None,
            converted_datetime: None,
            error: None,
            status_message: None,
        }
    }

    pub fn update_now(&mut self) {
        let now = Utc::now();
        self.current_epoch = now.timestamp();
        self.current_datetime = now;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Tab => {
                self.mode = match self.mode {
                    Mode::EpochToDate => Mode::DateToEpoch,
                    Mode::DateToEpoch => Mode::EpochToDate,
                };
                self.input.clear();
                self.error = None;
                self.converted_epoch = None;
                self.converted_datetime = None;
                self.status_message = Some(format!("Mode: {:?}", self.mode));
            }
            KeyCode::Char('u') => {
                self.unit = match self.unit {
                    TimestampUnit::Seconds => TimestampUnit::Milliseconds,
                    TimestampUnit::Milliseconds => TimestampUnit::Microseconds,
                    TimestampUnit::Microseconds => TimestampUnit::Nanoseconds,
                    TimestampUnit::Nanoseconds => TimestampUnit::Seconds,
                };
                self.process();
                self.status_message = Some(format!("Unit: {}", self.unit.as_str()));
            }
            KeyCode::Esc => {
                self.input.clear();
                self.error = None;
                self.converted_epoch = None;
                self.converted_datetime = None;
                self.status_message = Some("Cleared".to_string());
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.process();
            }
            KeyCode::Char('n') => {
                // Use current timestamp
                self.input = self.current_epoch.to_string();
                self.process();
                self.status_message = Some("Using current timestamp".to_string());
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.status_message = Some("Copied to clipboard".to_string());
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '-' || c == ':' || c == ' ' || c == 'T' => {
                self.input.push(c);
                self.process();
            }
            _ => {}
        }
        false
    }

    fn process(&mut self) {
        self.error = None;
        self.converted_epoch = None;
        self.converted_datetime = None;

        if self.input.is_empty() {
            return;
        }

        match self.mode {
            Mode::EpochToDate => self.convert_epoch_to_date(),
            Mode::DateToEpoch => self.convert_date_to_epoch(),
        }
    }

    fn convert_epoch_to_date(&mut self) {
        let Ok(timestamp) = self.input.trim().parse::<i64>() else {
            self.error = Some("Invalid timestamp".to_string());
            return;
        };

        let seconds = match self.unit {
            TimestampUnit::Seconds => timestamp,
            TimestampUnit::Milliseconds => timestamp / 1000,
            TimestampUnit::Microseconds => timestamp / 1_000_000,
            TimestampUnit::Nanoseconds => timestamp / 1_000_000_000,
        };

        match Utc.timestamp_opt(seconds, 0) {
            chrono::LocalResult::Single(dt) => {
                self.converted_datetime = Some(dt);
            }
            _ => {
                self.error = Some("Invalid timestamp value".to_string());
            }
        }
    }

    fn convert_date_to_epoch(&mut self) {
        let input = self.input.trim();

        // Try various date formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d",
        ];

        for format in formats {
            if let Ok(naive) = NaiveDateTime::parse_from_str(input, format) {
                let dt = Utc.from_utc_datetime(&naive);
                self.converted_epoch = Some(match self.unit {
                    TimestampUnit::Seconds => dt.timestamp(),
                    TimestampUnit::Milliseconds => dt.timestamp_millis(),
                    TimestampUnit::Microseconds => dt.timestamp_micros(),
                    TimestampUnit::Nanoseconds => dt.timestamp_nanos_opt().unwrap_or(0),
                });
                return;
            }
        }

        // Try date-only format
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
            let naive = naive_date.and_hms_opt(0, 0, 0).unwrap();
            let dt = Utc.from_utc_datetime(&naive);
            self.converted_epoch = Some(match self.unit {
                TimestampUnit::Seconds => dt.timestamp(),
                TimestampUnit::Milliseconds => dt.timestamp_millis(),
                TimestampUnit::Microseconds => dt.timestamp_micros(),
                TimestampUnit::Nanoseconds => dt.timestamp_nanos_opt().unwrap_or(0),
            });
            return;
        }

        self.error = Some("Invalid date format. Use: YYYY-MM-DD HH:MM:SS".to_string());
    }

    pub fn local_time(&self) -> DateTime<Local> {
        self.current_datetime.with_timezone(&Local)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        format!(
            "{:?} ({}) | Tab:mode u:unit n:now Esc:clear",
            self.mode,
            self.unit.as_str()
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
