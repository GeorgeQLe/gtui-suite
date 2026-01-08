use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub clusters: Vec<ClusterConfig>,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub name: String,
    pub bootstrap_servers: String,
    #[serde(default)]
    pub sasl_mechanism: Option<String>,
    #[serde(default)]
    pub sasl_username: Option<String>,
    #[serde(default)]
    pub sasl_password: Option<String>,
    #[serde(default)]
    pub ssl_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh_secs")]
    pub refresh_secs: u64,
    #[serde(default)]
    pub default_cluster: Option<String>,
    #[serde(default = "default_lag_threshold")]
    pub lag_alert_threshold: i64,
}

fn default_refresh_secs() -> u64 {
    5
}

fn default_lag_threshold() -> i64 {
    10000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clusters: vec![ClusterConfig::default()],
            display: DisplayConfig::default(),
        }
    }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            name: "local".to_string(),
            bootstrap_servers: "localhost:9092".to_string(),
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
            ssl_enabled: false,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_secs: default_refresh_secs(),
            default_cluster: Some("local".to_string()),
            lag_alert_threshold: default_lag_threshold(),
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
        ProjectDirs::from("", "", "queue-monitor-kafka")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }

    pub fn get_cluster(&self, name: &str) -> Option<&ClusterConfig> {
        self.clusters.iter().find(|c| c.name == name)
    }
}
