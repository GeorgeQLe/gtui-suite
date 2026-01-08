use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub logs: LogsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_prefer")]
    pub prefer: String,
    #[serde(default = "default_docker_socket")]
    pub docker_socket: String,
    #[serde(default = "default_podman_socket")]
    pub podman_socket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default)]
    pub show_all_containers: bool,
    #[serde(default = "default_true")]
    pub show_sizes: bool,
    #[serde(default = "default_true")]
    pub show_ports: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsConfig {
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    #[serde(default = "default_true")]
    pub timestamps: bool,
}

fn default_prefer() -> String { "auto".to_string() }
fn default_docker_socket() -> String { "/var/run/docker.sock".to_string() }
fn default_podman_socket() -> String { "$XDG_RUNTIME_DIR/podman/podman.sock".to_string() }
fn default_max_lines() -> usize { 1000 }
fn default_true() -> bool { true }

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            prefer: default_prefer(),
            docker_socket: default_docker_socket(),
            podman_socket: default_podman_socket(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_all_containers: false,
            show_sizes: true,
            show_ports: true,
        }
    }
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            max_lines: default_max_lines(),
            timestamps: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "docker-manager")
            .map(|p| p.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
