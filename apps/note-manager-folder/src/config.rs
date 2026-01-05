//! Configuration for note manager.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_notes_dir")]
    pub notes_dir: PathBuf,
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

fn default_notes_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "note-manager-folder")
        .map(|d| d.data_dir().join("notes"))
        .unwrap_or_else(|| PathBuf::from("./notes"))
}

impl Default for Config {
    fn default() -> Self {
        Self {
            notes_dir: default_notes_dir(),
            editor: EditorConfig::default(),
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
        directories::ProjectDirs::from("", "", "note-manager-folder")
            .map(|d| d.config_dir().join("config.toml"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    #[serde(default = "default_true")]
    pub word_wrap: bool,
}

fn default_tab_width() -> usize { 4 }
fn default_true() -> bool { true }

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            line_numbers: true,
            word_wrap: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_hidden: bool,
    #[serde(default = "default_true")]
    pub show_preview: bool,
    #[serde(default = "default_preview_lines")]
    pub preview_lines: usize,
}

fn default_preview_lines() -> usize { 20 }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            show_preview: true,
            preview_lines: 20,
        }
    }
}
