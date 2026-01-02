//! Configuration for habit tracker.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Display settings.
    #[serde(default)]
    pub display: DisplayConfig,
    /// Notification settings.
    #[serde(default)]
    pub notifications: NotificationConfig,
    /// Export settings.
    #[serde(default)]
    pub export: ExportConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            notifications: NotificationConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from default path.
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save configuration to default path.
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

    /// Get configuration file path.
    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "habit-tracker")
            .map(|d| d.config_dir().join("config.toml"))
    }

    /// Get database path.
    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "habit-tracker")
            .map(|d| d.data_dir().join("habits.db"))
    }
}

/// Display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// First day of week.
    #[serde(default = "default_week_start")]
    pub week_start: WeekStart,
    /// Date format string.
    #[serde(default = "default_date_format")]
    pub date_format: String,
    /// Show completed habits at bottom.
    #[serde(default)]
    pub completed_at_bottom: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            week_start: WeekStart::Monday,
            date_format: default_date_format(),
            completed_at_bottom: false,
        }
    }
}

fn default_week_start() -> WeekStart {
    WeekStart::Monday
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

/// First day of week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}

/// Notification settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Time for daily reminder (HH:MM format).
    #[serde(default)]
    pub remind_time: Option<String>,
    /// Warn about incomplete habits.
    #[serde(default = "default_true")]
    pub incomplete_warning: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            remind_time: Some("20:00".to_string()),
            incomplete_warning: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Export settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format.
    #[serde(default)]
    pub format: ExportFormat,
    /// Export directory.
    #[serde(default)]
    pub path: Option<PathBuf>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Csv,
            path: None,
        }
    }
}

/// Export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Csv,
    Json,
}
