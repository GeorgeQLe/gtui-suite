use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub hostname: String,
    pub ip_address: String,
    pub status: ServerStatus,
    pub last_seen: DateTime<Utc>,
    pub uptime_secs: u64,
    pub metrics: ServerMetrics,
}

impl Server {
    pub fn new(hostname: &str) -> Self {
        Self {
            hostname: hostname.to_string(),
            ip_address: String::new(),
            status: ServerStatus::Unknown,
            last_seen: Utc::now(),
            uptime_secs: 0,
            metrics: ServerMetrics::default(),
        }
    }

    pub fn uptime_display(&self) -> String {
        let secs = self.uptime_secs;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, mins)
        } else if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }

    pub fn last_seen_display(&self) -> String {
        let elapsed = Utc::now().signed_duration_since(self.last_seen);
        let secs = elapsed.num_seconds();

        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else {
            format!("{}h ago", secs / 3600)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerStatus {
    Online,
    Warning,
    Critical,
    Offline,
    Unknown,
}

impl ServerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerStatus::Online => "Online",
            ServerStatus::Warning => "Warning",
            ServerStatus::Critical => "Critical",
            ServerStatus::Offline => "Offline",
            ServerStatus::Unknown => "Unknown",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ServerStatus::Online => "●",
            ServerStatus::Warning => "▲",
            ServerStatus::Critical => "✗",
            ServerStatus::Offline => "○",
            ServerStatus::Unknown => "?",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub cpu_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub process_count: u32,
}

impl ServerMetrics {
    pub fn memory_percent(&self) -> f64 {
        if self.memory_total > 0 {
            (self.memory_used as f64 / self.memory_total as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn disk_percent(&self) -> f64 {
        if self.disk_total > 0 {
            (self.disk_used as f64 / self.disk_total as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn memory_display(&self) -> String {
        format!(
            "{:.1} / {:.1} GB",
            self.memory_used as f64 / (1024.0 * 1024.0 * 1024.0),
            self.memory_total as f64 / (1024.0 * 1024.0 * 1024.0)
        )
    }

    pub fn disk_display(&self) -> String {
        format!(
            "{:.1} / {:.1} GB",
            self.disk_used as f64 / (1024.0 * 1024.0 * 1024.0),
            self.disk_total as f64 / (1024.0 * 1024.0 * 1024.0)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

impl Metric {
    pub fn new(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
            labels: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPayload {
    pub hostname: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub server: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub message: String,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Alert {
    pub fn new(server: &str, metric: &str, value: f64, threshold: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(),
            server: server.to_string(),
            metric: metric.to_string(),
            value,
            threshold,
            severity: AlertSeverity::Warning,
            message: format!("{} is {} (threshold: {})", metric, value, threshold),
            started_at: Utc::now(),
            resolved_at: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.resolved_at.is_none()
    }

    pub fn duration_display(&self) -> String {
        let end = self.resolved_at.unwrap_or_else(Utc::now);
        let duration = end.signed_duration_since(self.started_at);
        let secs = duration.num_seconds();

        if secs >= 3600 {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        } else if secs >= 60 {
            format!("{}m", secs / 60)
        } else {
            format!("{}s", secs)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "Info",
            AlertSeverity::Warning => "Warning",
            AlertSeverity::Critical => "Critical",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "ℹ",
            AlertSeverity::Warning => "⚠",
            AlertSeverity::Critical => "🔴",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub metric: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

impl AlertRule {
    pub fn new(name: &str, metric: &str, condition: AlertCondition, threshold: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            metric: metric.to_string(),
            condition,
            threshold,
            severity: AlertSeverity::Warning,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
}

impl AlertCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertCondition::GreaterThan => ">",
            AlertCondition::LessThan => "<",
            AlertCondition::Equal => "=",
        }
    }

    pub fn check(&self, value: f64, threshold: f64) -> bool {
        match self {
            AlertCondition::GreaterThan => value > threshold,
            AlertCondition::LessThan => value < threshold,
            AlertCondition::Equal => (value - threshold).abs() < f64::EPSILON,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricHistory {
    pub metric: String,
    pub values: Vec<(DateTime<Utc>, f64)>,
}

impl MetricHistory {
    pub fn new(metric: &str) -> Self {
        Self {
            metric: metric.to_string(),
            values: Vec::new(),
        }
    }

    pub fn add(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.values.push((timestamp, value));
        // Keep last 100 values
        if self.values.len() > 100 {
            self.values.remove(0);
        }
    }

    pub fn sparkline(&self, width: usize) -> String {
        if self.values.is_empty() || width == 0 {
            return String::new();
        }

        let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let values: Vec<f64> = self.values.iter().map(|(_, v)| *v).collect();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        let step = values.len() as f64 / width as f64;
        let mut result = String::with_capacity(width);

        for i in 0..width {
            let idx = (i as f64 * step) as usize;
            let value = values.get(idx).copied().unwrap_or(0.0);

            let normalized = if range > 0.0 {
                ((value - min) / range).clamp(0.0, 1.0)
            } else {
                0.5
            };

            let char_idx = (normalized * 7.0) as usize;
            result.push(chars[char_idx.min(7)]);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_uptime_display() {
        let mut server = Server::new("test");
        server.uptime_secs = 90061; // 1d 1h 1m 1s
        assert_eq!(server.uptime_display(), "1d 1h 1m");
    }

    #[test]
    fn test_server_metrics_percent() {
        let metrics = ServerMetrics {
            memory_total: 16 * 1024 * 1024 * 1024,
            memory_used: 8 * 1024 * 1024 * 1024,
            disk_total: 500 * 1024 * 1024 * 1024,
            disk_used: 250 * 1024 * 1024 * 1024,
            ..Default::default()
        };

        assert!((metrics.memory_percent() - 50.0).abs() < 0.01);
        assert!((metrics.disk_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_alert_condition() {
        assert!(AlertCondition::GreaterThan.check(90.0, 80.0));
        assert!(!AlertCondition::GreaterThan.check(70.0, 80.0));
        assert!(AlertCondition::LessThan.check(10.0, 20.0));
    }
}
