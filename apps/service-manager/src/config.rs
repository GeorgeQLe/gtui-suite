//! Configuration for service manager.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub refresh_interval_secs: u64,
    #[serde(default)]
    pub show_system_services: bool,
}

impl Config {
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "service-manager")
            .map(|d| d.config_dir().join("config.toml"))
    }
}
