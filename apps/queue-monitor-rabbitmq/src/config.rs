use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rabbitmq: RabbitConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbitConfig {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default = "default_password")]
    pub password: String,
    #[serde(default = "default_vhost")]
    pub default_vhost: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh_secs")]
    pub refresh_secs: u64,
    #[serde(default = "default_max_messages")]
    pub max_messages_preview: u32,
    #[serde(default = "default_message_truncate")]
    pub message_truncate_bytes: usize,
}

fn default_url() -> String {
    "http://localhost:15672".to_string()
}

fn default_user() -> String {
    "guest".to_string()
}

fn default_password() -> String {
    "guest".to_string()
}

fn default_vhost() -> String {
    "/".to_string()
}

fn default_refresh_secs() -> u64 {
    5
}

fn default_max_messages() -> u32 {
    10
}

fn default_message_truncate() -> usize {
    500
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rabbitmq: RabbitConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl Default for RabbitConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            user: default_user(),
            password: default_password(),
            default_vhost: default_vhost(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_secs: default_refresh_secs(),
            max_messages_preview: default_max_messages(),
            message_truncate_bytes: default_message_truncate(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        ProjectDirs::from("", "", "queue-monitor-rabbitmq")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
