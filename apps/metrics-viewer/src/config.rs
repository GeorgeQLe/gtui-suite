use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub prometheus: PrometheusConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub dashboards: DashboardsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    #[serde(default = "default_prometheus_url")]
    pub url: String,
    #[serde(default = "default_auth")]
    pub auth: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            url: default_prometheus_url(),
            auth: default_auth(),
            user: None,
            password: None,
            token: None,
        }
    }
}

fn default_prometheus_url() -> String {
    "http://localhost:9090".to_string()
}

fn default_auth() -> String {
    "none".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh")]
    pub refresh_secs: u64,
    #[serde(default = "default_time_range")]
    pub time_range: String,
    #[serde(default = "default_graph_height")]
    pub graph_height: u16,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_secs: default_refresh(),
            time_range: default_time_range(),
            graph_height: default_graph_height(),
        }
    }
}

fn default_refresh() -> u64 {
    30
}

fn default_time_range() -> String {
    "1h".to_string()
}

fn default_graph_height() -> u16 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardsConfig {
    #[serde(default = "default_dashboards_path")]
    pub path: String,
}

impl Default for DashboardsConfig {
    fn default() -> Self {
        Self {
            path: default_dashboards_path(),
        }
    }
}

fn default_dashboards_path() -> String {
    "~/.config/metrics-viewer/dashboards".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prometheus: PrometheusConfig::default(),
            display: DisplayConfig::default(),
            dashboards: DashboardsConfig::default(),
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
        ProjectDirs::from("", "", "metrics-viewer")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
