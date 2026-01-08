use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub scan: ScanConfig,
    #[serde(default)]
    pub ports: PortsConfig,
    #[serde(default)]
    pub disclaimer_accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_concurrent")]
    pub max_concurrent: usize,
    #[serde(default = "default_rate")]
    pub rate_limit_per_sec: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortsConfig {
    #[serde(default = "default_preset")]
    pub default_preset: String,
    #[serde(default = "default_common_ports")]
    pub common: Vec<u16>,
}

fn default_timeout() -> u64 {
    1000
}

fn default_concurrent() -> usize {
    100
}

fn default_rate() -> u32 {
    500
}

fn default_preset() -> String {
    "common".to_string()
}

fn default_common_ports() -> Vec<u16> {
    vec![21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 993, 995, 3306, 3389, 5432, 8080, 8443]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan: ScanConfig::default(),
            ports: PortsConfig::default(),
            disclaimer_accepted: false,
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_timeout(),
            max_concurrent: default_concurrent(),
            rate_limit_per_sec: default_rate(),
        }
    }
}

impl Default for PortsConfig {
    fn default() -> Self {
        Self {
            default_preset: default_preset(),
            common: default_common_ports(),
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

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        ProjectDirs::from("", "", "port-scanner-local")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }

    pub fn data_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("", "", "port-scanner-local")
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
        let path = dirs.data_dir().to_path_buf();
        fs::create_dir_all(&path)?;
        Ok(path)
    }
}
