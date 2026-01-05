//! Configuration for Zettelkasten.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            defaults: DefaultsConfig::default(),
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
        directories::ProjectDirs::from("", "", "note-manager-zettelkasten")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "note-manager-zettelkasten")
            .map(|d| d.data_dir().join("zettelkasten.db"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_links: bool,
    #[serde(default = "default_true")]
    pub show_tags: bool,
    #[serde(default = "default_preview_len")]
    pub preview_length: usize,
}

fn default_true() -> bool { true }
fn default_preview_len() -> usize { 80 }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_links: true,
            show_tags: true,
            preview_length: 80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_type")]
    pub default_type: String,
}

fn default_type() -> String { "Fleeting".to_string() }

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            default_type: "Fleeting".to_string(),
        }
    }
}
