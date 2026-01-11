use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub gitlab: GitLabConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub repos: Vec<String>,
    #[serde(default)]
    pub label_mapping: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    #[serde(default = "default_gitlab_url")]
    pub url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub projects: Vec<String>,
}

impl Default for GitLabConfig {
    fn default() -> Self {
        Self {
            url: default_gitlab_url(),
            token: String::new(),
            projects: Vec::new(),
        }
    }
}

fn default_gitlab_url() -> String {
    "https://gitlab.com".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    #[serde(default = "default_true")]
    pub auto_sync: bool,
    #[serde(default = "default_conflict_strategy")]
    pub conflict_strategy: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_interval(),
            auto_sync: true,
            conflict_strategy: default_conflict_strategy(),
        }
    }
}

fn default_interval() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

fn default_conflict_strategy() -> String {
    "prompt".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_column_width")]
    pub column_width: u16,
    #[serde(default = "default_true")]
    pub show_labels: bool,
    #[serde(default = "default_true")]
    pub show_assignees: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            column_width: default_column_width(),
            show_labels: true,
            show_assignees: true,
        }
    }
}

fn default_column_width() -> u16 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: GitHubConfig::default(),
            gitlab: GitLabConfig::default(),
            sync: SyncConfig::default(),
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
        ProjectDirs::from("", "", "kanban-integrated")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
