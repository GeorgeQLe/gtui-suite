use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub database: u8,
    #[serde(default)]
    pub cluster: bool,
    #[serde(default)]
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh_secs")]
    pub refresh_secs: u64,
    #[serde(default = "default_max_keys")]
    pub max_keys_display: usize,
    #[serde(default = "default_key_pattern")]
    pub default_key_pattern: String,
}

fn default_url() -> String {
    "redis://localhost:6379".to_string()
}

fn default_refresh_secs() -> u64 {
    5
}

fn default_max_keys() -> usize {
    1000
}

fn default_key_pattern() -> String {
    "*".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            redis: RedisConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            password: None,
            database: 0,
            cluster: false,
            nodes: Vec::new(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_secs: default_refresh_secs(),
            max_keys_display: default_max_keys(),
            default_key_pattern: default_key_pattern(),
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
        ProjectDirs::from("", "", "queue-monitor-redis")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
