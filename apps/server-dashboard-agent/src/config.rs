use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub collector: CollectorConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub alerts: AlertsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    #[serde(default = "default_collector_url")]
    pub url: String,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default = "default_interval")]
    pub poll_interval_secs: u64,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            url: default_collector_url(),
            auth_token: None,
            poll_interval_secs: default_interval(),
        }
    }
}

fn default_collector_url() -> String {
    "http://localhost:9100".to_string()
}

fn default_interval() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh")]
    pub refresh_secs: u64,
    #[serde(default = "default_true")]
    pub show_graphs: bool,
    #[serde(default = "default_graph_width")]
    pub graph_width: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_secs: default_refresh(),
            show_graphs: true,
            graph_width: default_graph_width(),
        }
    }
}

fn default_refresh() -> u64 {
    5
}

fn default_true() -> bool {
    true
}

fn default_graph_width() -> usize {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    #[serde(default = "default_true")]
    pub notifications: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
    #[serde(default = "default_grace_period")]
    pub grace_period_secs: u64,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            notifications: true,
            sound: false,
            grace_period_secs: default_grace_period(),
        }
    }
}

fn default_grace_period() -> u64 {
    90 // 3x default interval
}

impl Default for Config {
    fn default() -> Self {
        Self {
            collector: CollectorConfig::default(),
            display: DisplayConfig::default(),
            alerts: AlertsConfig::default(),
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
        ProjectDirs::from("", "", "server-dashboard-agent")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
