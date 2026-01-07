use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Log severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn label(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn severity(&self) -> u8 {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" | "TRC" => Some(LogLevel::Trace),
            "DEBUG" | "DBG" => Some(LogLevel::Debug),
            "INFO" | "INF" => Some(LogLevel::Info),
            "WARN" | "WARNING" | "WRN" => Some(LogLevel::Warn),
            "ERROR" | "ERR" | "FATAL" | "CRITICAL" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// Format of a log entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Plain,
    Json,
    Logfmt,
}

/// A parsed log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Original line number in file
    pub line_number: usize,
    /// Raw line content
    pub raw: String,
    /// Parsed timestamp
    pub timestamp: Option<DateTime<Utc>>,
    /// Log level
    pub level: LogLevel,
    /// Main message
    pub message: String,
    /// Structured fields (for JSON/logfmt)
    pub fields: HashMap<String, String>,
    /// Detected format
    pub format: LogFormat,
}

impl LogEntry {
    pub fn new(line_number: usize, raw: String) -> Self {
        Self {
            line_number,
            message: raw.clone(),
            raw,
            timestamp: None,
            level: LogLevel::Info,
            fields: HashMap::new(),
            format: LogFormat::Plain,
        }
    }

    /// Get display text for the entry
    pub fn display(&self) -> &str {
        if self.message.is_empty() {
            &self.raw
        } else {
            &self.message
        }
    }

    /// Check if entry matches a search pattern
    pub fn matches(&self, pattern: &str) -> bool {
        self.raw.to_lowercase().contains(&pattern.to_lowercase())
    }
}
