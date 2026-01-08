use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::models::Protocol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub protocol: Protocol,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub auto_connect: bool,
    #[serde(default)]
    pub auto_join: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_timestamp_format")]
    pub timestamp_format: String,
    #[serde(default = "default_true")]
    pub show_timestamps: bool,
    #[serde(default = "default_true")]
    pub show_user_list: bool,
    #[serde(default = "default_max_history")]
    pub max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub on_mention: bool,
    #[serde(default = "default_true")]
    pub on_private: bool,
    #[serde(default)]
    pub sound: bool,
}

fn default_port() -> u16 {
    6667
}

fn default_timestamp_format() -> String {
    "%H:%M".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_history() -> usize {
    1000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: vec![ServerConfig {
                name: "Libera Chat".to_string(),
                protocol: Protocol::Irc,
                host: "irc.libera.chat".to_string(),
                port: 6697,
                username: "user".to_string(),
                password: None,
                auto_connect: false,
                auto_join: vec!["#rust".to_string()],
            }],
            display: DisplayConfig::default(),
            notifications: NotificationConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            timestamp_format: default_timestamp_format(),
            show_timestamps: true,
            show_user_list: true,
            max_history: default_max_history(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            on_mention: true,
            on_private: true,
            sound: false,
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
        ProjectDirs::from("", "", "chat-client")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
