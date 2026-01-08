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
    pub gaps: GapsConfig,
    #[serde(default)]
    pub borders: BordersConfig,
    #[serde(default)]
    pub workspaces: WorkspacesConfig,
    #[serde(default)]
    pub status_bar: StatusBarConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_mod_key")]
    pub mod_key: String,
    #[serde(default = "default_layout")]
    pub default_layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapsConfig {
    #[serde(default = "default_gap")]
    pub inner: u16,
    #[serde(default)]
    pub outer: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BordersConfig {
    #[serde(default = "default_border")]
    pub width: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacesConfig {
    #[serde(default = "default_workspace_names")]
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarConfig {
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default = "default_true")]
    pub show_workspaces: bool,
    #[serde(default = "default_true")]
    pub show_title: bool,
}

fn default_mod_key() -> String {
    "ctrl+space".to_string()
}

fn default_layout() -> String {
    "splith".to_string()
}

fn default_gap() -> u16 {
    1
}

fn default_border() -> u16 {
    1
}

fn default_workspace_names() -> Vec<String> {
    (1..=9).map(|i| i.to_string()).collect()
}

fn default_position() -> String {
    "bottom".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            gaps: GapsConfig::default(),
            borders: BordersConfig::default(),
            workspaces: WorkspacesConfig::default(),
            status_bar: StatusBarConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mod_key: default_mod_key(),
            default_layout: default_layout(),
        }
    }
}

impl Default for GapsConfig {
    fn default() -> Self {
        Self {
            inner: default_gap(),
            outer: 0,
        }
    }
}

impl Default for BordersConfig {
    fn default() -> Self {
        Self {
            width: default_border(),
        }
    }
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            names: default_workspace_names(),
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            show_workspaces: true,
            show_title: true,
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
        ProjectDirs::from("", "", "tui-shell-tiled")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
