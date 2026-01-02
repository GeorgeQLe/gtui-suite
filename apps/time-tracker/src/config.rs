//! Configuration for time tracker.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub timer: TimerConfig,
    #[serde(default)]
    pub pomodoro: PomodoroConfig,
    #[serde(default)]
    pub idle: IdleConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timer: TimerConfig::default(),
            pomodoro: PomodoroConfig::default(),
            idle: IdleConfig::default(),
            display: DisplayConfig::default(),
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
        directories::ProjectDirs::from("", "", "time-tracker")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "time-tracker")
            .map(|d| d.data_dir().join("time.db"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    #[serde(default)]
    pub default_project: Option<String>,
    #[serde(default = "default_true")]
    pub show_seconds: bool,
}

fn default_true() -> bool { true }

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            default_project: None,
            show_seconds: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    #[serde(default = "default_work_mins")]
    pub work_mins: u32,
    #[serde(default = "default_short_break")]
    pub short_break_mins: u32,
    #[serde(default = "default_long_break")]
    pub long_break_mins: u32,
    #[serde(default = "default_pomodoros")]
    pub pomodoros_before_long: u32,
    #[serde(default)]
    pub auto_start_breaks: bool,
    #[serde(default)]
    pub auto_start_work: bool,
}

fn default_work_mins() -> u32 { 25 }
fn default_short_break() -> u32 { 5 }
fn default_long_break() -> u32 { 15 }
fn default_pomodoros() -> u32 { 4 }

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_mins: 25,
            short_break_mins: 5,
            long_break_mins: 15,
            pomodoros_before_long: 4,
            auto_start_breaks: false,
            auto_start_work: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_idle_threshold")]
    pub threshold_mins: u32,
    #[serde(default)]
    pub action: IdleAction,
}

fn default_idle_threshold() -> u32 { 5 }

impl Default for IdleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_mins: 5,
            action: IdleAction::Prompt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IdleAction {
    Pause,
    #[default]
    Prompt,
    DiscardIdle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_time_format")]
    pub time_format: TimeFormat,
    #[serde(default)]
    pub week_start: WeekStart,
}

fn default_time_format() -> TimeFormat { TimeFormat::H24 }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            time_format: TimeFormat::H24,
            week_start: WeekStart::Monday,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeFormat {
    #[default]
    #[serde(rename = "24h")]
    H24,
    #[serde(rename = "12h")]
    H12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}
