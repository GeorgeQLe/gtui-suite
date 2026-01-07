#![allow(dead_code)]

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub board: BoardConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardConfig {
    #[serde(default = "default_board_name")]
    pub default: String,
}

fn default_board_name() -> String {
    "Personal".to_string()
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            default: default_board_name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_due_dates: bool,
    #[serde(default = "default_true")]
    pub show_labels: bool,
    #[serde(default = "default_cards_visible")]
    pub cards_visible: usize,
}

fn default_true() -> bool {
    true
}

fn default_cards_visible() -> usize {
    10
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_due_dates: true,
            show_labels: true,
            cards_visible: 10,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            board: BoardConfig::default(),
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
        let dirs = ProjectDirs::from("com", "tui-suite", "kanban-standalone")
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let config_dir = dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn data_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "tui-suite", "kanban-standalone")
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
        let data_dir = dirs.data_dir();
        fs::create_dir_all(data_dir)?;
        Ok(data_dir.join("kanban.db"))
    }
}
