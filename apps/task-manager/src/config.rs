//! Configuration for task manager.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub filters: FilterConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            defaults: DefaultsConfig::default(),
            filters: FilterConfig::default(),
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
        directories::ProjectDirs::from("", "", "task-manager")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "task-manager")
            .map(|d| d.data_dir().join("tasks.db"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_completed: bool,
    #[serde(default = "default_true")]
    pub show_description: bool,
    #[serde(default = "default_true")]
    pub show_due_date: bool,
    #[serde(default)]
    pub compact_mode: bool,
    #[serde(default = "default_date_format")]
    pub date_format: String,
}

fn default_true() -> bool { true }
fn default_date_format() -> String { "%Y-%m-%d".to_string() }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_completed: true,
            show_description: true,
            show_due_date: true,
            compact_mode: false,
            date_format: "%Y-%m-%d".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default)]
    pub default_project: Option<String>,
    #[serde(default)]
    pub default_context: Option<String>,
    #[serde(default = "default_priority")]
    pub default_priority: String,
}

fn default_priority() -> String { "Low".to_string() }

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            default_project: None,
            default_context: None,
            default_priority: "Low".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    #[serde(default)]
    pub hide_future_scheduled: bool,
    #[serde(default = "default_upcoming_days")]
    pub upcoming_days: u32,
}

fn default_upcoming_days() -> u32 { 7 }

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            hide_future_scheduled: false,
            upcoming_days: 7,
        }
    }
}
