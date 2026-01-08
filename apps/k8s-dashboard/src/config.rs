use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub kubernetes: KubernetesConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub logs: LogsConfig,
    #[serde(default)]
    pub resources: ResourcesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    #[serde(default = "default_kubeconfig")]
    pub kubeconfig: String,
    #[serde(default = "default_namespace")]
    pub default_namespace: String,
    #[serde(default)]
    pub context: String,
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            kubeconfig: default_kubeconfig(),
            default_namespace: default_namespace(),
            context: String::new(),
        }
    }
}

fn default_kubeconfig() -> String {
    "~/.kube/config".to_string()
}

fn default_namespace() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh")]
    pub refresh_secs: u64,
    #[serde(default = "default_true")]
    pub show_metrics: bool,
    #[serde(default = "default_true")]
    pub watch_mode: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_secs: default_refresh(),
            show_metrics: true,
            watch_mode: true,
        }
    }
}

fn default_refresh() -> u64 {
    5
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsConfig {
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    #[serde(default = "default_true")]
    pub follow: bool,
    #[serde(default = "default_true")]
    pub timestamps: bool,
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            max_lines: default_max_lines(),
            follow: true,
            timestamps: true,
        }
    }
}

fn default_max_lines() -> usize {
    1000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesConfig {
    #[serde(default = "default_view")]
    pub default_view: String,
}

impl Default for ResourcesConfig {
    fn default() -> Self {
        Self {
            default_view: default_view(),
        }
    }
}

fn default_view() -> String {
    "pods".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kubernetes: KubernetesConfig::default(),
            display: DisplayConfig::default(),
            logs: LogsConfig::default(),
            resources: ResourcesConfig::default(),
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
        ProjectDirs::from("", "", "k8s-dashboard")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
