use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub log_entries: Vec<LogEntry>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub count: u32,
    pub acknowledged: bool,
}

impl Alert {
    pub fn new(rule_name: &str, severity: Severity, message: &str, entry: LogEntry) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            rule_name: rule_name.to_string(),
            severity,
            message: message.to_string(),
            log_entries: vec![entry],
            first_seen: now,
            last_seen: now,
            count: 1,
            acknowledged: false,
        }
    }

    pub fn add_entry(&mut self, entry: LogEntry) {
        self.log_entries.push(entry);
        self.last_seen = Utc::now();
        self.count += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "INFO",
            Severity::Warning => "WARN",
            Severity::Error => "ERROR",
            Severity::Critical => "CRIT",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "INFO" | "INFORMATION" => Severity::Info,
            "WARN" | "WARNING" => Severity::Warning,
            "ERR" | "ERROR" => Severity::Error,
            "CRIT" | "CRITICAL" | "FATAL" => Severity::Critical,
            _ => Severity::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: Option<DateTime<Utc>>,
    pub source: String,
    pub line_number: usize,
    pub content: String,
    pub severity: Option<Severity>,
}

impl LogEntry {
    pub fn new(source: &str, line_number: usize, content: &str) -> Self {
        Self {
            timestamp: None,
            source: source.to_string(),
            line_number,
            content: content.to_string(),
            severity: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRule {
    pub id: Uuid,
    pub name: String,
    pub pattern: String,
    pub severity: Severity,
    pub description: String,
    pub enabled: bool,
    pub false_positive_count: u32,
}

impl PatternRule {
    pub fn new(name: &str, pattern: &str, severity: Severity, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            pattern: pattern.to_string(),
            severity,
            description: description.to_string(),
            enabled: true,
            false_positive_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStats {
    pub total_lines: u64,
    pub error_count: u64,
    pub warning_count: u64,
    pub avg_lines_per_minute: f64,
    pub unique_sources: usize,
    pub training_complete: bool,
}

impl Default for BaselineStats {
    fn default() -> Self {
        Self {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            avg_lines_per_minute: 0.0,
            unique_sources: 0,
            training_complete: false,
        }
    }
}

impl BaselineStats {
    pub fn error_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            self.error_count as f64 / self.total_lines as f64 * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from_str("ERROR"), Severity::Error);
        assert_eq!(Severity::from_str("warn"), Severity::Warning);
        assert_eq!(Severity::from_str("CRITICAL"), Severity::Critical);
    }

    #[test]
    fn test_alert_add_entry() {
        let entry = LogEntry::new("/var/log/test", 1, "Error message");
        let mut alert = Alert::new("test-rule", Severity::Error, "Test", entry);
        assert_eq!(alert.count, 1);

        alert.add_entry(LogEntry::new("/var/log/test", 2, "Another error"));
        assert_eq!(alert.count, 2);
        assert_eq!(alert.log_entries.len(), 2);
    }

    #[test]
    fn test_baseline_error_rate() {
        let stats = BaselineStats {
            total_lines: 1000,
            error_count: 50,
            ..Default::default()
        };
        assert!((stats.error_rate() - 5.0).abs() < 0.01);
    }
}
