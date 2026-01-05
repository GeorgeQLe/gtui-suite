//! Configuration for task scheduler.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub scheduler: SchedulerConfig,
}

impl Config {
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "task-scheduler")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "task-scheduler")
            .map(|d| d.data_dir().join("scheduler.db"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    #[serde(default = "default_timeout")]
    pub default_timeout_secs: u64,
    #[serde(default = "default_history")]
    pub keep_history_days: u32,
}

fn default_timeout() -> u64 { 300 }
fn default_history() -> u32 { 30 }

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 300,
            keep_history_days: 30,
        }
    }
}
