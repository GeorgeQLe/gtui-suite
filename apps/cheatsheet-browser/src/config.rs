//! Configuration for cheatsheet browser.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub sources: SourcesConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesConfig {
    #[serde(default = "default_true")]
    pub bundled: bool,
    #[serde(default)]
    pub user_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    #[serde(default = "default_true")]
    pub wrap_lines: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SourcesConfig {
    fn default() -> Self {
        Self {
            bundled: true,
            user_path: None,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            wrap_lines: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sources: SourcesConfig::default(),
            display: DisplayConfig::default(),
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

    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "cheatsheet-browser")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn user_cheatsheets_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "cheatsheet-browser")
            .map(|d| d.config_dir().join("cheatsheets"))
    }
}
