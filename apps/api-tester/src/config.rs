#![allow(dead_code)]

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    #[serde(default = "default_max_redirects")]
    pub max_redirects: usize,
    #[serde(default = "default_true")]
    pub verify_ssl: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

fn default_max_redirects() -> usize {
    10
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            follow_redirects: true,
            max_redirects: 10,
            verify_ssl: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_timing: bool,
    #[serde(default = "default_true")]
    pub show_headers: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_timing: true,
            show_headers: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http: HttpConfig::default(),
            display: DisplayConfig::default(),
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
        let dirs = ProjectDirs::from("com", "tui-suite", "api-tester")
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let config_dir = dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn data_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "tui-suite", "api-tester")
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
        let data_dir = dirs.data_dir();
        fs::create_dir_all(data_dir)?;
        Ok(data_dir.join("api-tester.db"))
    }
}
