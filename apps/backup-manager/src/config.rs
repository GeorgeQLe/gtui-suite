use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub rsync: RsyncConfig,
    #[serde(default)]
    pub restic: ResticConfig,
    #[serde(default)]
    pub borg: BorgConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_log_path")]
    pub log_path: String,
    #[serde(default)]
    pub notification_cmd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsyncConfig {
    #[serde(default = "default_rsync_options")]
    pub default_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResticConfig {
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorgConfig {
    #[serde(default)]
    pub compression: Option<String>,
}

fn default_log_path() -> String {
    "~/.local/share/backup-manager/logs".to_string()
}

fn default_rsync_options() -> Vec<String> {
    vec!["-avz".to_string(), "--delete".to_string()]
}

fn default_cache_dir() -> String {
    "~/.cache/restic".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            rsync: RsyncConfig::default(),
            restic: ResticConfig::default(),
            borg: BorgConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_path: default_log_path(),
            notification_cmd: Some("notify-send".to_string()),
        }
    }
}

impl Default for RsyncConfig {
    fn default() -> Self {
        Self {
            default_options: default_rsync_options(),
        }
    }
}

impl Default for ResticConfig {
    fn default() -> Self {
        Self {
            cache_dir: default_cache_dir(),
        }
    }
}

impl Default for BorgConfig {
    fn default() -> Self {
        Self {
            compression: Some("lz4".to_string()),
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
        directories::ProjectDirs::from("", "", "backup-manager")
            .map(|p| p.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
