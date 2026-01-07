use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

use crate::log_entry::{LogEntry, LogFormat, LogLevel};

/// Log line parser with format detection
pub struct LogParser {
    // Common timestamp patterns
    timestamp_patterns: Vec<Regex>,
    // Level pattern
    level_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            timestamp_patterns: vec![
                // ISO 8601
                Regex::new(r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)").unwrap(),
                // Common log format: 2024-01-15 10:30:00
                Regex::new(r"(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}(?:\.\d+)?)").unwrap(),
                // Syslog format: Jan 15 10:30:00
                Regex::new(r"([A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})").unwrap(),
            ],
            level_pattern: Regex::new(r"\b(TRACE|DEBUG|INFO|WARN(?:ING)?|ERROR|FATAL|CRITICAL|TRC|DBG|INF|WRN|ERR)\b").unwrap(),
        }
    }

    pub fn parse(&self, line: &str, line_number: usize) -> LogEntry {
        let trimmed = line.trim();

        // Try JSON first
        if trimmed.starts_with('{') {
            if let Some(entry) = self.parse_json(trimmed, line_number) {
                return entry;
            }
        }

        // Try logfmt
        if trimmed.contains('=') && !trimmed.contains('{') {
            if let Some(entry) = self.parse_logfmt(trimmed, line_number) {
                return entry;
            }
        }

        // Fall back to plain text parsing
        self.parse_plain(line, line_number)
    }

    fn parse_json(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let json: serde_json::Value = serde_json::from_str(line).ok()?;
        let obj = json.as_object()?;

        let mut entry = LogEntry::new(line_number, line.to_string());
        entry.format = LogFormat::Json;

        // Extract timestamp
        for key in &["timestamp", "time", "ts", "@timestamp", "datetime"] {
            if let Some(val) = obj.get(*key) {
                if let Some(ts) = self.parse_timestamp_value(val) {
                    entry.timestamp = Some(ts);
                    break;
                }
            }
        }

        // Extract level
        for key in &["level", "severity", "loglevel", "lvl"] {
            if let Some(val) = obj.get(*key) {
                if let Some(s) = val.as_str() {
                    if let Some(level) = LogLevel::from_str(s) {
                        entry.level = level;
                        break;
                    }
                }
            }
        }

        // Extract message
        for key in &["message", "msg", "text", "log"] {
            if let Some(val) = obj.get(*key) {
                if let Some(s) = val.as_str() {
                    entry.message = s.to_string();
                    break;
                }
            }
        }

        // Extract other fields
        for (key, val) in obj {
            if !["timestamp", "time", "ts", "@timestamp", "datetime",
                 "level", "severity", "loglevel", "lvl",
                 "message", "msg", "text", "log"].contains(&key.as_str()) {
                entry.fields.insert(key.clone(), format_json_value(val));
            }
        }

        Some(entry)
    }

    fn parse_logfmt(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let mut fields = HashMap::new();
        let mut message = String::new();
        let mut level = LogLevel::Info;
        let mut timestamp = None;

        // Simple logfmt parser
        let mut chars = line.chars().peekable();
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut in_value = false;
        let mut in_quoted = false;

        while let Some(c) = chars.next() {
            if in_quoted {
                if c == '"' {
                    in_quoted = false;
                } else {
                    current_value.push(c);
                }
            } else if c == '=' && !in_value {
                in_value = true;
            } else if c == '"' && in_value && current_value.is_empty() {
                in_quoted = true;
            } else if c == ' ' && in_value {
                // End of key=value pair
                if !current_key.is_empty() {
                    let key = current_key.trim().to_lowercase();
                    let val = current_value.trim().to_string();

                    match key.as_str() {
                        "msg" | "message" => message = val,
                        "level" | "lvl" => {
                            if let Some(l) = LogLevel::from_str(&val) {
                                level = l;
                            }
                        }
                        "time" | "timestamp" | "ts" => {
                            timestamp = self.parse_timestamp_str(&val);
                        }
                        _ => {
                            fields.insert(current_key.clone(), val);
                        }
                    }
                }
                current_key.clear();
                current_value.clear();
                in_value = false;
            } else if in_value {
                current_value.push(c);
            } else {
                current_key.push(c);
            }
        }

        // Handle last pair
        if !current_key.is_empty() && in_value {
            let key = current_key.trim().to_lowercase();
            let val = current_value.trim().to_string();
            match key.as_str() {
                "msg" | "message" => message = val,
                "level" | "lvl" => {
                    if let Some(l) = LogLevel::from_str(&val) {
                        level = l;
                    }
                }
                _ => {
                    fields.insert(current_key, val);
                }
            }
        }

        if fields.is_empty() && message.is_empty() {
            return None;
        }

        let mut entry = LogEntry::new(line_number, line.to_string());
        entry.format = LogFormat::Logfmt;
        entry.level = level;
        entry.message = if message.is_empty() { line.to_string() } else { message };
        entry.fields = fields;
        entry.timestamp = timestamp;

        Some(entry)
    }

    fn parse_plain(&self, line: &str, line_number: usize) -> LogEntry {
        let mut entry = LogEntry::new(line_number, line.to_string());
        entry.format = LogFormat::Plain;

        // Extract timestamp
        for pattern in &self.timestamp_patterns {
            if let Some(caps) = pattern.captures(line) {
                if let Some(ts_str) = caps.get(1) {
                    if let Some(ts) = self.parse_timestamp_str(ts_str.as_str()) {
                        entry.timestamp = Some(ts);
                        break;
                    }
                }
            }
        }

        // Extract level
        if let Some(caps) = self.level_pattern.captures(line) {
            if let Some(level_match) = caps.get(1) {
                if let Some(level) = LogLevel::from_str(level_match.as_str()) {
                    entry.level = level;
                }
            }
        }

        // Message is the full line for plain format
        entry.message = line.to_string();

        entry
    }

    fn parse_timestamp_value(&self, val: &serde_json::Value) -> Option<DateTime<Utc>> {
        match val {
            serde_json::Value::String(s) => self.parse_timestamp_str(s),
            serde_json::Value::Number(n) => {
                // Unix timestamp
                let secs = n.as_i64()?;
                DateTime::from_timestamp(secs, 0)
            }
            _ => None,
        }
    }

    fn parse_timestamp_str(&self, s: &str) -> Option<DateTime<Utc>> {
        // Try ISO 8601
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.with_timezone(&Utc));
        }

        // Try common formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.f",
        ];

        for fmt in &formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
                return Some(dt.and_utc());
            }
        }

        None
    }
}

fn format_json_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => val.to_string(),
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}
