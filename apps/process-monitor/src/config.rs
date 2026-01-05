//! Configuration for process monitor.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub columns: ColumnsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh_ms")]
    pub refresh_ms: u64,
    #[serde(default = "default_view")]
    pub default_view: String,
    #[serde(default = "default_sort")]
    pub default_sort: String,
    #[serde(default)]
    pub show_threads: bool,
    #[serde(default)]
    pub show_kernel_threads: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnsConfig {
    #[serde(default = "default_columns")]
    pub visible: Vec<String>,
}

fn default_refresh_ms() -> u64 {
    1000
}

fn default_view() -> String {
    "list".into()
}

fn default_sort() -> String {
    "cpu".into()
}

fn default_columns() -> Vec<String> {
    vec![
        "pid".into(),
        "user".into(),
        "cpu".into(),
        "mem".into(),
        "state".into(),
        "command".into(),
    ]
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_ms: default_refresh_ms(),
            default_view: default_view(),
            default_sort: default_sort(),
            show_threads: false,
            show_kernel_threads: false,
        }
    }
}

impl Default for ColumnsConfig {
    fn default() -> Self {
        Self {
            visible: default_columns(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            columns: ColumnsConfig::default(),
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

    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "process-monitor")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn refresh_ms(&self) -> u64 {
        self.display.refresh_ms
    }
}
