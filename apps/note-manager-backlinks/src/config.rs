//! Configuration for backlinks note manager.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub editor: EditorConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            editor: EditorConfig::default(),
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
        directories::ProjectDirs::from("", "", "note-manager-backlinks")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "note-manager-backlinks")
            .map(|d| d.data_dir().join("notes.db"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_backlinks: bool,
    #[serde(default = "default_true")]
    pub show_forward_links: bool,
    #[serde(default = "default_preview_len")]
    pub preview_length: usize,
}

fn default_true() -> bool { true }
fn default_preview_len() -> usize { 100 }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_backlinks: true,
            show_forward_links: true,
            preview_length: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default = "default_true")]
    pub auto_link_titles: bool,
    #[serde(default = "default_true")]
    pub show_link_preview: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            auto_link_titles: true,
            show_link_preview: true,
        }
    }
}
