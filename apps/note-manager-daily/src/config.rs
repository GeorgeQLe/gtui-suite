//! Configuration for daily notes manager.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub template: TemplateConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            template: TemplateConfig::default(),
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
        directories::ProjectDirs::from("", "", "note-manager-daily")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "note-manager-daily")
            .map(|d| d.data_dir().join("journal.db"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default = "default_true")]
    pub show_word_count: bool,
    #[serde(default = "default_true")]
    pub show_stats: bool,
    #[serde(default)]
    pub week_starts_monday: bool,
}

fn default_date_format() -> String { "%A, %B %d, %Y".to_string() }
fn default_true() -> bool { true }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            date_format: "%A, %B %d, %Y".to_string(),
            show_word_count: true,
            show_stats: true,
            week_starts_monday: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    #[serde(default = "default_template")]
    pub daily_template: String,
    #[serde(default = "default_true")]
    pub use_template: bool,
}

fn default_template() -> String {
    "# {{date}}\n\n## Today's Goals\n- \n\n## Notes\n\n## Reflections\n".to_string()
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            daily_template: default_template(),
            use_template: true,
        }
    }
}
