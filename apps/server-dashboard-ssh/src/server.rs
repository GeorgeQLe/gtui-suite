use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    pub user: Option<String>,
    pub port: u16,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl Server {
    pub fn new(name: String, host: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            host,
            user: None,
            port: 22,
            tags: Vec::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn connection_string(&self) -> String {
        let mut s = String::new();
        if let Some(user) = &self.user {
            s.push_str(user);
            s.push('@');
        }
        s.push_str(&self.host);
        if self.port != 22 {
            s.push(':');
            s.push_str(&self.port.to_string());
        }
        s
    }

    pub fn tags_display(&self) -> String {
        if self.tags.is_empty() {
            String::new()
        } else {
            self.tags.join(", ")
        }
    }
}

/// Server metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub server_id: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub load_1: f32,
    pub load_5: f32,
    pub load_15: f32,
    pub status: ServerStatus,
}

impl ServerMetrics {
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            timestamp: Utc::now(),
            cpu_percent: 0.0,
            memory_used: 0,
            memory_total: 0,
            disk_used: 0,
            disk_total: 0,
            load_1: 0.0,
            load_5: 0.0,
            load_15: 0.0,
            status: ServerStatus::Unknown,
        }
    }

    pub fn memory_percent(&self) -> f32 {
        if self.memory_total == 0 {
            0.0
        } else {
            (self.memory_used as f32 / self.memory_total as f32) * 100.0
        }
    }

    pub fn disk_percent(&self) -> f32 {
        if self.disk_total == 0 {
            0.0
        } else {
            (self.disk_used as f32 / self.disk_total as f32) * 100.0
        }
    }

    pub fn format_memory(&self) -> String {
        format!("{:.1}G", self.memory_used as f64 / 1_073_741_824.0)
    }
}

/// Server status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerStatus {
    Ok,
    Warning,
    Critical,
    Unreachable,
    Unknown,
}

impl ServerStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            ServerStatus::Ok => "●",
            ServerStatus::Warning => "⚠",
            ServerStatus::Critical => "✗",
            ServerStatus::Unreachable => "○",
            ServerStatus::Unknown => "?",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ServerStatus::Ok => "OK",
            ServerStatus::Warning => "WARNING",
            ServerStatus::Critical => "CRITICAL",
            ServerStatus::Unreachable => "UNREACHABLE",
            ServerStatus::Unknown => "UNKNOWN",
        }
    }
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric: MetricType,
    pub condition: Condition,
    pub threshold: f64,
    pub severity: Severity,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Cpu,
    Memory,
    Disk,
    Load,
}

impl MetricType {
    pub fn label(&self) -> &'static str {
        match self {
            MetricType::Cpu => "CPU",
            MetricType::Memory => "Memory",
            MetricType::Disk => "Disk",
            MetricType::Load => "Load",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Warning,
    Critical,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Warning => "Warning",
            Severity::Critical => "Critical",
        }
    }
}
