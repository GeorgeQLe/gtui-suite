use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Backup backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    Rsync,
    Restic,
    Borg,
}

impl BackendType {
    pub fn label(&self) -> &'static str {
        match self {
            BackendType::Rsync => "rsync",
            BackendType::Restic => "restic",
            BackendType::Borg => "borg",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rsync" => Some(BackendType::Rsync),
            "restic" => Some(BackendType::Restic),
            "borg" => Some(BackendType::Borg),
            _ => None,
        }
    }
}

/// Backup profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupProfile {
    pub id: String,
    pub name: String,
    pub backend: BackendType,
    pub source_paths: Vec<PathBuf>,
    pub destination: String,
    pub excludes: Vec<String>,
    pub schedule: Option<String>,
    pub retention: RetentionPolicy,
    pub pre_hooks: Vec<String>,
    pub post_hooks: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl BackupProfile {
    pub fn new(name: String, backend: BackendType, destination: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            backend,
            source_paths: Vec::new(),
            destination,
            excludes: Vec::new(),
            schedule: None,
            retention: RetentionPolicy::default(),
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }

    /// Get source paths as comma-separated string
    pub fn sources_display(&self) -> String {
        self.source_paths.iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetentionPolicy {
    pub keep_last: Option<u32>,
    pub keep_hourly: Option<u32>,
    pub keep_daily: Option<u32>,
    pub keep_weekly: Option<u32>,
    pub keep_monthly: Option<u32>,
    pub keep_yearly: Option<u32>,
}

impl RetentionPolicy {
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if let Some(n) = self.keep_last { parts.push(format!("last:{}", n)); }
        if let Some(n) = self.keep_daily { parts.push(format!("daily:{}", n)); }
        if let Some(n) = self.keep_weekly { parts.push(format!("weekly:{}", n)); }
        if let Some(n) = self.keep_monthly { parts.push(format!("monthly:{}", n)); }
        if parts.is_empty() {
            "None".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Backup run record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRun {
    pub id: String,
    pub profile_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: RunStatus,
    pub bytes_transferred: Option<u64>,
    pub files_transferred: Option<u64>,
    pub error_message: Option<String>,
}

impl BackupRun {
    pub fn new(profile_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            profile_id,
            started_at: Utc::now(),
            finished_at: None,
            status: RunStatus::Running,
            bytes_transferred: None,
            files_transferred: None,
            error_message: None,
        }
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.finished_at.map(|end| end - self.started_at)
    }

    pub fn duration_display(&self) -> String {
        match self.duration() {
            Some(d) => {
                let secs = d.num_seconds();
                if secs >= 3600 {
                    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                } else if secs >= 60 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    format!("{}s", secs)
                }
            }
            None => "Running...".to_string(),
        }
    }
}

/// Backup run status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Running,
    Success,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn label(&self) -> &'static str {
        match self {
            RunStatus::Running => "Running",
            RunStatus::Success => "Success",
            RunStatus::Failed => "Failed",
            RunStatus::Cancelled => "Cancelled",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            RunStatus::Running => "⏳",
            RunStatus::Success => "✓",
            RunStatus::Failed => "✗",
            RunStatus::Cancelled => "○",
        }
    }
}

/// Snapshot info (from backup backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub profile_id: String,
    pub created_at: DateTime<Utc>,
    pub size: Option<u64>,
    pub paths: Vec<String>,
}

impl Snapshot {
    pub fn format_size(&self) -> String {
        match self.size {
            Some(bytes) => format_bytes(bytes),
            None => "Unknown".to_string(),
        }
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
