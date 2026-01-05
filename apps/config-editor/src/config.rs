//! Configuration for config editor.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            recent_files: Vec::new(),
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

    pub fn add_recent(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "config-editor")
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
    pub tree_view: bool,
    #[serde(default = "default_true")]
    pub auto_validate: bool,
}

fn default_tab_width() -> usize { 2 }
fn default_true() -> bool { true }

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 2,
            line_numbers: true,
            tree_view: true,
            auto_validate: true,
        }
    }
}
