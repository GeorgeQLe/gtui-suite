use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub input: InputConfig,
    #[serde(default)]
    pub baseline: BaselineConfig,
    #[serde(default)]
    pub rules: RulesConfig,
    #[serde(default)]
    pub alerts: AlertsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    #[serde(default = "default_files")]
    pub files: Vec<String>,
    #[serde(default = "default_true")]
    pub watch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_training_days")]
    pub training_days: u32,
    #[serde(default = "default_sensitivity")]
    pub sensitivity: String,
    #[serde(default = "default_scan_interval")]
    pub scan_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    #[serde(default = "default_true")]
    pub builtin: bool,
    #[serde(default)]
    pub custom_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    #[serde(default)]
    pub notification_cmd: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default = "default_min_severity")]
    pub min_severity: String,
}

fn default_files() -> Vec<String> {
    vec![
        "/var/log/syslog".to_string(),
        "/var/log/auth.log".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

fn default_training_days() -> u32 {
    7
}

fn default_sensitivity() -> String {
    "medium".to_string()
}

fn default_scan_interval() -> u64 {
    30
}

fn default_min_severity() -> String {
    "warning".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input: InputConfig::default(),
            baseline: BaselineConfig::default(),
            rules: RulesConfig::default(),
            alerts: AlertsConfig::default(),
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            files: default_files(),
            watch: true,
        }
    }
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            training_days: default_training_days(),
            sensitivity: default_sensitivity(),
            scan_interval_secs: default_scan_interval(),
        }
    }
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            builtin: true,
            custom_path: None,
        }
    }
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            notification_cmd: None,
            email: None,
            min_severity: default_min_severity(),
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
        ProjectDirs::from("", "", "log-anomaly-detector")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
