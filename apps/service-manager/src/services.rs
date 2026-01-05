//! Service discovery and management.

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub description: String,
    pub status: ServiceStatus,
    pub enabled: bool,
    pub pid: Option<u32>,
    pub memory_mb: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

impl ServiceStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ServiceStatus::Running => "running",
            ServiceStatus::Stopped => "stopped",
            ServiceStatus::Failed => "failed",
            ServiceStatus::Unknown => "unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ServiceStatus::Running => "●",
            ServiceStatus::Stopped => "○",
            ServiceStatus::Failed => "✗",
            ServiceStatus::Unknown => "?",
        }
    }
}

pub fn list_services() -> Vec<Service> {
    // Try systemctl first (Linux)
    if let Ok(output) = Command::new("systemctl")
        .args(["list-units", "--type=service", "--no-pager", "--plain"])
        .output()
    {
        if output.status.success() {
            return parse_systemctl_output(&String::from_utf8_lossy(&output.stdout));
        }
    }

    // Fallback to mock data
    vec![
        Service { name: "ssh".into(), description: "OpenSSH Server".into(), status: ServiceStatus::Running, enabled: true, pid: Some(1234), memory_mb: Some(12.5) },
        Service { name: "nginx".into(), description: "Web Server".into(), status: ServiceStatus::Running, enabled: true, pid: Some(2345), memory_mb: Some(45.2) },
        Service { name: "postgresql".into(), description: "Database Server".into(), status: ServiceStatus::Stopped, enabled: false, pid: None, memory_mb: None },
        Service { name: "docker".into(), description: "Container Runtime".into(), status: ServiceStatus::Running, enabled: true, pid: Some(3456), memory_mb: Some(128.0) },
        Service { name: "redis".into(), description: "Cache Server".into(), status: ServiceStatus::Stopped, enabled: true, pid: None, memory_mb: None },
    ]
}

fn parse_systemctl_output(output: &str) -> Vec<Service> {
    output.lines()
        .filter(|line| line.contains(".service"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].trim_end_matches(".service").to_string();
                let status = match parts[3] {
                    "running" => ServiceStatus::Running,
                    "exited" | "dead" => ServiceStatus::Stopped,
                    "failed" => ServiceStatus::Failed,
                    _ => ServiceStatus::Unknown,
                };
                Some(Service {
                    name,
                    description: parts.get(4..).map(|p| p.join(" ")).unwrap_or_default(),
                    status,
                    enabled: true,
                    pid: None,
                    memory_mb: None,
                })
            } else {
                None
            }
        })
        .take(50)
        .collect()
}

pub fn start_service(name: &str) -> Result<(), String> {
    Command::new("systemctl")
        .args(["start", name])
        .output()
        .map_err(|e| e.to_string())
        .and_then(|o| if o.status.success() { Ok(()) } else { Err("Failed to start".into()) })
}

pub fn stop_service(name: &str) -> Result<(), String> {
    Command::new("systemctl")
        .args(["stop", name])
        .output()
        .map_err(|e| e.to_string())
        .and_then(|o| if o.status.success() { Ok(()) } else { Err("Failed to stop".into()) })
}

pub fn restart_service(name: &str) -> Result<(), String> {
    Command::new("systemctl")
        .args(["restart", name])
        .output()
        .map_err(|e| e.to_string())
        .and_then(|o| if o.status.success() { Ok(()) } else { Err("Failed to restart".into()) })
}
