use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SSH host profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<PathBuf>,
    pub proxy_jump: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub last_connected: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl HostProfile {
    pub fn new(name: String, host: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            host,
            user: None,
            port: None,
            identity_file: None,
            proxy_jump: None,
            tags: Vec::new(),
            notes: None,
            last_connected: None,
            created_at: Utc::now(),
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Get connection string (user@host:port)
    pub fn connection_string(&self) -> String {
        let mut s = String::new();
        if let Some(user) = &self.user {
            s.push_str(user);
            s.push('@');
        }
        s.push_str(&self.host);
        if let Some(port) = self.port {
            if port != 22 {
                s.push(':');
                s.push_str(&port.to_string());
            }
        }
        s
    }

    /// Build SSH command arguments
    pub fn ssh_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(port) = self.port {
            args.push("-p".to_string());
            args.push(port.to_string());
        }

        if let Some(identity) = &self.identity_file {
            args.push("-i".to_string());
            args.push(identity.display().to_string());
        }

        if let Some(proxy) = &self.proxy_jump {
            args.push("-J".to_string());
            args.push(proxy.clone());
        }

        // Build destination
        let dest = if let Some(user) = &self.user {
            format!("{}@{}", user, self.host)
        } else {
            self.host.clone()
        };
        args.push(dest);

        args
    }

    /// Format tags for display
    pub fn tags_display(&self) -> String {
        if self.tags.is_empty() {
            String::new()
        } else {
            self.tags.join(", ")
        }
    }

    /// Format last connected for display
    pub fn last_connected_display(&self) -> String {
        match &self.last_connected {
            Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
            None => "Never".to_string(),
        }
    }
}

/// Command snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub command: String,
    pub host_id: Option<String>,
    pub description: Option<String>,
}

impl Snippet {
    pub fn new(name: String, command: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            command,
            host_id: None,
            description: None,
        }
    }

    /// Substitute variables in command
    pub fn substitute(&self, host: &HostProfile) -> String {
        self.command
            .replace("{{host}}", &host.host)
            .replace("{{user}}", host.user.as_deref().unwrap_or(""))
            .replace("{{port}}", &host.port.unwrap_or(22).to_string())
    }
}

/// Port forwarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub id: String,
    pub host_id: String,
    pub forward_type: ForwardType,
    pub local_port: u16,
    pub remote_host: Option<String>,
    pub remote_port: u16,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForwardType {
    Local,
    Remote,
    Dynamic,
}

impl ForwardType {
    pub fn label(&self) -> &'static str {
        match self {
            ForwardType::Local => "Local (-L)",
            ForwardType::Remote => "Remote (-R)",
            ForwardType::Dynamic => "Dynamic (-D)",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            ForwardType::Local => "-L",
            ForwardType::Remote => "-R",
            ForwardType::Dynamic => "-D",
        }
    }
}

impl PortForward {
    /// Build SSH forward argument
    pub fn ssh_arg(&self) -> String {
        match self.forward_type {
            ForwardType::Dynamic => {
                format!("{}:{}", self.forward_type.flag(), self.local_port)
            }
            _ => {
                let remote = self.remote_host.as_deref().unwrap_or("localhost");
                format!("{}:{}:{}:{}",
                    self.forward_type.flag(),
                    self.local_port,
                    remote,
                    self.remote_port
                )
            }
        }
    }
}

/// Connection history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHistory {
    pub id: String,
    pub host_id: String,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<i64>,
}
