use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub status_bar: StatusBarConfig,
    #[serde(default)]
    pub switcher: SwitcherConfig,
    #[serde(default)]
    pub quick_slots: QuickSlotsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub show_status_bar: bool,
    #[serde(default = "default_true")]
    pub double_tap_switch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarConfig {
    #[serde(default = "default_true")]
    pub show_app_name: bool,
    #[serde(default = "default_true")]
    pub show_app_count: bool,
    #[serde(default = "default_true")]
    pub show_clock: bool,
    #[serde(default = "default_clock_format")]
    pub clock_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitcherConfig {
    #[serde(default = "default_switcher_width")]
    pub width: u16,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_true")]
    pub recent_first: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickSlotsConfig {
    #[serde(default)]
    pub slots: Vec<Option<String>>,
}

fn default_true() -> bool {
    true
}

fn default_clock_format() -> String {
    "%H:%M".to_string()
}

fn default_switcher_width() -> u16 {
    50
}

fn default_max_results() -> usize {
    10
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            status_bar: StatusBarConfig::default(),
            switcher: SwitcherConfig::default(),
            quick_slots: QuickSlotsConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            show_status_bar: true,
            double_tap_switch: true,
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            show_app_name: true,
            show_app_count: true,
            show_clock: true,
            clock_format: default_clock_format(),
        }
    }
}

impl Default for SwitcherConfig {
    fn default() -> Self {
        Self {
            width: default_switcher_width(),
            max_results: default_max_results(),
            recent_first: true,
        }
    }
}

impl Default for QuickSlotsConfig {
    fn default() -> Self {
        Self {
            slots: vec![
                Some("habit-tracker".to_string()),
                Some("task-manager".to_string()),
                Some("time-tracker".to_string()),
                Some("note-manager".to_string()),
                Some("file-manager".to_string()),
                None,
                None,
                None,
                None,
            ],
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
        ProjectDirs::from("", "", "tui-shell-fullscreen")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
