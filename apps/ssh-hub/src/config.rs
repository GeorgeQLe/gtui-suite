use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ssh: SshConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    #[serde(default = "default_true")]
    pub parse_config: bool,
    #[serde(default = "default_ssh_config_path")]
    pub config_path: String,
    #[serde(default)]
    pub agent_forwarding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_tags: bool,
    #[serde(default = "default_true")]
    pub show_last_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
    #[serde(default = "default_keepalive")]
    pub keepalive_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_true() -> bool { true }
fn default_ssh_config_path() -> String { "~/.ssh/config".to_string() }
fn default_timeout() -> u32 { 30 }
fn default_keepalive() -> u32 { 60 }
fn default_max_entries() -> usize { 100 }

impl Default for Config {
    fn default() -> Self {
        Self {
            ssh: SshConfig::default(),
            display: DisplayConfig::default(),
            connection: ConnectionConfig::default(),
            history: HistoryConfig::default(),
        }
    }
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            parse_config: true,
            config_path: default_ssh_config_path(),
            agent_forwarding: false,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_tags: true,
            show_last_connected: true,
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            keepalive_secs: default_keepalive(),
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
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
        directories::ProjectDirs::from("", "", "ssh-hub")
            .map(|p| p.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
