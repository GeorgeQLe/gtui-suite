use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub tabs: TabsConfig,
    #[serde(default)]
    pub splits: SplitsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabsConfig {
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default = "default_true")]
    pub show_index: bool,
    #[serde(default = "default_max_title")]
    pub max_title_length: usize,
    #[serde(default = "default_true")]
    pub show_close: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitsConfig {
    #[serde(default = "default_ratio")]
    pub default_ratio: f32,
    #[serde(default = "default_min_pane")]
    pub min_pane_size: u16,
}

fn default_position() -> String {
    "top".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_title() -> usize {
    20
}

fn default_ratio() -> f32 {
    0.5
}

fn default_min_pane() -> u16 {
    5
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tabs: TabsConfig::default(),
            splits: SplitsConfig::default(),
        }
    }
}

impl Default for TabsConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            show_index: true,
            max_title_length: default_max_title(),
            show_close: true,
        }
    }
}

impl Default for SplitsConfig {
    fn default() -> Self {
        Self {
            default_ratio: default_ratio(),
            min_pane_size: default_min_pane(),
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

    fn config_path() -> Result<PathBuf> {
        ProjectDirs::from("", "", "tui-shell-tabbed")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
