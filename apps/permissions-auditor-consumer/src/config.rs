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
    pub checks: ChecksConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_paths")]
    pub paths: Vec<String>,
    #[serde(default)]
    pub follow_symlinks: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksConfig {
    #[serde(default = "default_true")]
    pub world_writable: bool,
    #[serde(default = "default_true")]
    pub suid_sgid: bool,
    #[serde(default = "default_true")]
    pub ssh_permissions: bool,
    #[serde(default = "default_true")]
    pub gpg_permissions: bool,
    #[serde(default = "default_true")]
    pub ownership: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    #[serde(default = "default_ignore_paths")]
    pub paths: Vec<String>,
    #[serde(default = "default_ignore_patterns")]
    pub patterns: Vec<String>,
}

fn default_paths() -> Vec<String> {
    vec![dirs::home_dir().unwrap_or_default().to_string_lossy().to_string()]
}

fn default_max_depth() -> usize {
    10
}

fn default_true() -> bool {
    true
}

fn default_ignore_paths() -> Vec<String> {
    vec![
        "/tmp".to_string(),
        "/var/tmp".to_string(),
    ]
}

fn default_ignore_patterns() -> Vec<String> {
    vec![
        "*/node_modules/*".to_string(),
        "*/.git/*".to_string(),
        "*/.cache/*".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan: ScanConfig::default(),
            checks: ChecksConfig::default(),
            ignore: IgnoreConfig::default(),
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            paths: default_paths(),
            follow_symlinks: false,
            max_depth: default_max_depth(),
        }
    }
}

impl Default for ChecksConfig {
    fn default() -> Self {
        Self {
            world_writable: true,
            suid_sgid: true,
            ssh_permissions: true,
            gpg_permissions: true,
            ownership: true,
        }
    }
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            paths: default_ignore_paths(),
            patterns: default_ignore_patterns(),
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
        ProjectDirs::from("", "", "permissions-auditor-consumer")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}

mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
