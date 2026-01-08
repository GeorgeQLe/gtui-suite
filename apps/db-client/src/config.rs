use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub recent_connections: Vec<ConnectionInfo>,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub name: String,
    pub db_type: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_true")]
    pub show_row_numbers: bool,
}

fn default_max_rows() -> usize {
    1000
}

fn default_true() -> bool {
    true
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            max_rows: default_max_rows(),
            show_row_numbers: default_true(),
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

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        ProjectDirs::from("", "", "db-client")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }

    pub fn add_recent(&mut self, info: ConnectionInfo) {
        // Remove if already exists
        self.recent_connections.retain(|c| c.path != info.path);
        // Add to front
        self.recent_connections.insert(0, info);
        // Keep only last 10
        self.recent_connections.truncate(10);
    }
}
