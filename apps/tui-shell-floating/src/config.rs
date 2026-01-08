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
    pub window: WindowConfig,
    #[serde(default)]
    pub taskbar: TaskbarConfig,
    #[serde(default)]
    pub desktops: DesktopsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_mod_key")]
    pub mod_key: String,
    #[serde(default = "default_true")]
    pub raise_on_focus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_min_width")]
    pub min_width: u16,
    #[serde(default = "default_min_height")]
    pub min_height: u16,
    #[serde(default = "default_width")]
    pub default_width: u16,
    #[serde(default = "default_height")]
    pub default_height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskbarConfig {
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default = "default_true")]
    pub show_desktop_switcher: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopsConfig {
    #[serde(default = "default_desktop_count")]
    pub count: usize,
    #[serde(default = "default_desktop_names")]
    pub names: Vec<String>,
}

fn default_mod_key() -> String {
    "ctrl+space".to_string()
}

fn default_true() -> bool {
    true
}

fn default_min_width() -> u16 {
    20
}

fn default_min_height() -> u16 {
    5
}

fn default_width() -> u16 {
    60
}

fn default_height() -> u16 {
    20
}

fn default_position() -> String {
    "bottom".to_string()
}

fn default_desktop_count() -> usize {
    4
}

fn default_desktop_names() -> Vec<String> {
    (1..=4).map(|i| format!("Desktop {}", i)).collect()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            window: WindowConfig::default(),
            taskbar: TaskbarConfig::default(),
            desktops: DesktopsConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mod_key: default_mod_key(),
            raise_on_focus: true,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            min_width: default_min_width(),
            min_height: default_min_height(),
            default_width: default_width(),
            default_height: default_height(),
        }
    }
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            show_desktop_switcher: true,
        }
    }
}

impl Default for DesktopsConfig {
    fn default() -> Self {
        Self {
            count: default_desktop_count(),
            names: default_desktop_names(),
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
        ProjectDirs::from("", "", "tui-shell-floating")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
