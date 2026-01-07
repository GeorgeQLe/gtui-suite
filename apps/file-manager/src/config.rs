use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub sort: SortConfig,
    #[serde(default)]
    pub bookmarks: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "default_true")]
    pub show_icons: bool,
    #[serde(default = "default_true")]
    pub confirm_delete: bool,
    #[serde(default = "default_true")]
    pub preview_pane: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortConfig {
    #[serde(default = "default_sort")]
    pub default: String,
    #[serde(default = "default_true")]
    pub directories_first: bool,
    #[serde(default)]
    pub case_sensitive: bool,
}

fn default_true() -> bool { true }
fn default_sort() -> String { "name".to_string() }

impl Default for Config {
    fn default() -> Self {
        let mut bookmarks = HashMap::new();
        if let Some(home) = dirs::home_dir() {
            bookmarks.insert("home".to_string(), home.display().to_string());
            let downloads = home.join("Downloads");
            if downloads.exists() {
                bookmarks.insert("downloads".to_string(), downloads.display().to_string());
            }
        }

        Self {
            display: DisplayConfig::default(),
            sort: SortConfig::default(),
            bookmarks,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            show_icons: true,
            confirm_delete: true,
            preview_pane: true,
        }
    }
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            default: "name".to_string(),
            directories_first: true,
            case_sensitive: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "file-manager")
            .map(|p| p.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
