//! Configuration for personal wiki.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub editing: EditingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            editing: EditingConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(self)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }

    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "personal-wiki")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "personal-wiki")
            .map(|d| d.data_dir().join("wiki.db"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_backlinks: bool,
    #[serde(default = "default_true")]
    pub show_categories: bool,
    #[serde(default = "default_recent_limit")]
    pub recent_limit: usize,
}

fn default_true() -> bool { true }
fn default_recent_limit() -> usize { 20 }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_backlinks: true,
            show_categories: true,
            recent_limit: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditingConfig {
    #[serde(default = "default_true")]
    pub auto_link: bool,
    #[serde(default = "default_true")]
    pub save_revisions: bool,
}

impl Default for EditingConfig {
    fn default() -> Self {
        Self {
            auto_link: true,
            save_revisions: true,
        }
    }
}
