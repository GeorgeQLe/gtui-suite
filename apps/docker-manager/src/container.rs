use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Container state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}

impl ContainerState {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "created" => ContainerState::Created,
            "running" => ContainerState::Running,
            "paused" => ContainerState::Paused,
            "restarting" => ContainerState::Restarting,
            "removing" => ContainerState::Removing,
            "exited" => ContainerState::Exited,
            "dead" => ContainerState::Dead,
            _ => ContainerState::Created,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ContainerState::Created => "◯",
            ContainerState::Running => "●",
            ContainerState::Paused => "⏸",
            ContainerState::Restarting => "↻",
            ContainerState::Removing => "✕",
            ContainerState::Exited => "○",
            ContainerState::Dead => "✖",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ContainerState::Created => "Created",
            ContainerState::Running => "Running",
            ContainerState::Paused => "Paused",
            ContainerState::Restarting => "Restarting",
            ContainerState::Removing => "Removing",
            ContainerState::Exited => "Exited",
            ContainerState::Dead => "Dead",
        }
    }
}

/// Port binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortBinding {
    pub host_ip: Option<String>,
    pub host_port: Option<u16>,
    pub container_port: u16,
    pub protocol: String,
}

impl PortBinding {
    pub fn display(&self) -> String {
        if let (Some(ip), Some(port)) = (&self.host_ip, self.host_port) {
            format!("{}:{}->{}/{}", ip, port, self.container_port, self.protocol)
        } else if let Some(port) = self.host_port {
            format!("{}->{}/{}", port, self.container_port, self.protocol)
        } else {
            format!("{}/{}", self.container_port, self.protocol)
        }
    }
}

/// Mount binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub mount_type: String,
}

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub short_id: String,
    pub names: Vec<String>,
    pub image: String,
    pub command: String,
    pub created: Option<DateTime<Utc>>,
    pub state: ContainerState,
    pub status: String,
    pub ports: Vec<PortBinding>,
    pub mounts: Vec<Mount>,
}

impl Container {
    pub fn primary_name(&self) -> &str {
        self.names.first()
            .map(|s| s.trim_start_matches('/'))
            .unwrap_or(&self.short_id)
    }

    pub fn ports_display(&self) -> String {
        if self.ports.is_empty() {
            return String::new();
        }
        self.ports.iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Docker image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: String,
    pub short_id: String,
    pub repo_tags: Vec<String>,
    pub repo_digests: Vec<String>,
    pub created: Option<DateTime<Utc>>,
    pub size: u64,
    pub containers: i64,
}

impl Image {
    pub fn primary_tag(&self) -> &str {
        self.repo_tags.first()
            .map(|s| s.as_str())
            .unwrap_or("<none>:<none>")
    }

    pub fn format_size(&self) -> String {
        format_bytes(self.size)
    }
}

/// Volume information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: Option<DateTime<Utc>>,
    pub labels: Vec<(String, String)>,
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub containers: Vec<String>,
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
