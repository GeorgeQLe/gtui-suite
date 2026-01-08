use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub name: String,
    pub cluster: String,
    pub user: String,
    pub namespace: Option<String>,
}

impl Context {
    pub fn new(name: &str, cluster: &str, user: &str) -> Self {
        Self {
            name: name.to_string(),
            cluster: cluster.to_string(),
            user: user.to_string(),
            namespace: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub name: String,
    pub status: String,
    pub age: Duration,
}

impl Namespace {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: "Active".to_string(),
            age: Duration::from_secs(86400),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PodPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

impl PodPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            PodPhase::Pending => "Pending",
            PodPhase::Running => "Running",
            PodPhase::Succeeded => "Succeeded",
            PodPhase::Failed => "Failed",
            PodPhase::Unknown => "Unknown",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PodPhase::Pending => "○",
            PodPhase::Running => "●",
            PodPhase::Succeeded => "✓",
            PodPhase::Failed => "✗",
            PodPhase::Unknown => "?",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub phase: PodPhase,
    pub ready: String,
    pub restarts: u32,
    pub age: Duration,
    pub node: String,
    pub containers: Vec<Container>,
    pub cpu: Option<f64>,
    pub memory: Option<u64>,
    pub ip: Option<String>,
}

impl Pod {
    pub fn new(name: &str, namespace: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: namespace.to_string(),
            phase: PodPhase::Running,
            ready: "1/1".to_string(),
            restarts: 0,
            age: Duration::from_secs(3600),
            node: String::new(),
            containers: Vec::new(),
            cpu: None,
            memory: None,
            ip: None,
        }
    }

    pub fn age_display(&self) -> String {
        let secs = self.age.as_secs();
        if secs >= 86400 {
            format!("{}d", secs / 86400)
        } else if secs >= 3600 {
            format!("{}h", secs / 3600)
        } else if secs >= 60 {
            format!("{}m", secs / 60)
        } else {
            format!("{}s", secs)
        }
    }

    pub fn memory_display(&self) -> String {
        match self.memory {
            Some(bytes) if bytes >= 1024 * 1024 * 1024 => {
                format!("{:.1}Gi", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
            Some(bytes) if bytes >= 1024 * 1024 => {
                format!("{:.1}Mi", bytes as f64 / (1024.0 * 1024.0))
            }
            Some(bytes) => format!("{}Ki", bytes / 1024),
            None => "-".to_string(),
        }
    }

    pub fn cpu_display(&self) -> String {
        match self.cpu {
            Some(cpu) if cpu >= 1.0 => format!("{:.1}", cpu),
            Some(cpu) => format!("{}m", (cpu * 1000.0) as u64),
            None => "-".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: u32,
    pub state: ContainerState,
}

impl Container {
    pub fn new(name: &str, image: &str) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
            ready: true,
            restart_count: 0,
            state: ContainerState::Running,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Waiting,
    Running,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub name: String,
    pub namespace: String,
    pub ready: String,
    pub up_to_date: u32,
    pub available: u32,
    pub age: Duration,
    pub images: Vec<String>,
}

impl Deployment {
    pub fn new(name: &str, namespace: &str, replicas: u32) -> Self {
        Self {
            name: name.to_string(),
            namespace: namespace.to_string(),
            ready: format!("{}/{}", replicas, replicas),
            up_to_date: replicas,
            available: replicas,
            age: Duration::from_secs(86400),
            images: Vec::new(),
        }
    }

    pub fn age_display(&self) -> String {
        let secs = self.age.as_secs();
        if secs >= 86400 {
            format!("{}d", secs / 86400)
        } else if secs >= 3600 {
            format!("{}h", secs / 3600)
        } else {
            format!("{}m", secs / 60)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub external_ip: Option<String>,
    pub ports: Vec<String>,
    pub age: Duration,
}

impl Service {
    pub fn new(name: &str, namespace: &str, service_type: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: namespace.to_string(),
            service_type: service_type.to_string(),
            cluster_ip: "10.0.0.1".to_string(),
            external_ip: None,
            ports: Vec::new(),
            age: Duration::from_secs(86400),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub status: NodeStatus,
    pub roles: Vec<String>,
    pub age: Duration,
    pub version: String,
    pub internal_ip: String,
    pub cpu_capacity: String,
    pub memory_capacity: String,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<u64>,
}

impl Node {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: NodeStatus::Ready,
            roles: vec!["worker".to_string()],
            age: Duration::from_secs(86400 * 30),
            version: "v1.28.0".to_string(),
            internal_ip: "192.168.1.1".to_string(),
            cpu_capacity: "4".to_string(),
            memory_capacity: "16Gi".to_string(),
            cpu_usage: None,
            memory_usage: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Ready,
    NotReady,
    SchedulingDisabled,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::Ready => "Ready",
            NodeStatus::NotReady => "NotReady",
            NodeStatus::SchedulingDisabled => "SchedulingDisabled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub type_: String,
    pub reason: String,
    pub object: String,
    pub message: String,
    pub count: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl Event {
    pub fn new(type_: &str, reason: &str, object: &str, message: &str) -> Self {
        Self {
            type_: type_.to_string(),
            reason: reason.to_string(),
            object: object.to_string(),
            message: message.to_string(),
            count: 1,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMap {
    pub name: String,
    pub namespace: String,
    pub data_keys: Vec<String>,
    pub age: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub namespace: String,
    pub secret_type: String,
    pub data_keys: Vec<String>,
    pub age: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Pods,
    Deployments,
    Services,
    ConfigMaps,
    Secrets,
    Nodes,
    Events,
    Namespaces,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Pods => "Pods",
            ResourceType::Deployments => "Deployments",
            ResourceType::Services => "Services",
            ResourceType::ConfigMaps => "ConfigMaps",
            ResourceType::Secrets => "Secrets",
            ResourceType::Nodes => "Nodes",
            ResourceType::Events => "Events",
            ResourceType::Namespaces => "Namespaces",
        }
    }

    pub fn all() -> Vec<ResourceType> {
        vec![
            ResourceType::Pods,
            ResourceType::Deployments,
            ResourceType::Services,
            ResourceType::ConfigMaps,
            ResourceType::Secrets,
            ResourceType::Nodes,
            ResourceType::Events,
            ResourceType::Namespaces,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_age_display() {
        let mut pod = Pod::new("test", "default");
        pod.age = Duration::from_secs(90000);
        assert_eq!(pod.age_display(), "1d");

        pod.age = Duration::from_secs(7200);
        assert_eq!(pod.age_display(), "2h");
    }

    #[test]
    fn test_pod_memory_display() {
        let mut pod = Pod::new("test", "default");
        pod.memory = Some(256 * 1024 * 1024);
        assert_eq!(pod.memory_display(), "256.0Mi");
    }

    #[test]
    fn test_pod_phase() {
        assert_eq!(PodPhase::Running.as_str(), "Running");
        assert_eq!(PodPhase::Failed.icon(), "✗");
    }
}
