use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub collection: CollectionConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    #[serde(default = "default_concurrent")]
    pub concurrent_connections: usize,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_retention")]
    pub retention_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_graphs: bool,
    #[serde(default = "default_graph_width")]
    pub graph_width: u16,
}

fn default_interval() -> u64 { 30 }
fn default_concurrent() -> usize { 10 }
fn default_timeout() -> u64 { 10 }
fn default_retention() -> i64 { 30 }
fn default_true() -> bool { true }
fn default_graph_width() -> u16 { 20 }

impl Default for Config {
    fn default() -> Self {
        Self {
            collection: CollectionConfig::default(),
            storage: StorageConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_interval(),
            concurrent_connections: default_concurrent(),
            timeout_secs: default_timeout(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            retention_days: default_retention(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_graphs: true,
            graph_width: default_graph_width(),
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
        directories::ProjectDirs::from("", "", "server-dashboard-ssh")
            .map(|p| p.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
