use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub parsing: ParsingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    #[serde(default)]
    pub wrap_lines: bool,
    #[serde(default = "default_timestamp_format")]
    pub timestamp_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    #[serde(default = "default_error_color")]
    pub error: String,
    #[serde(default = "default_warn_color")]
    pub warn: String,
    #[serde(default = "default_info_color")]
    pub info: String,
    #[serde(default = "default_debug_color")]
    pub debug: String,
    #[serde(default = "default_trace_color")]
    pub trace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsingConfig {
    #[serde(default = "default_true")]
    pub detect_format: bool,
    #[serde(default = "default_timestamp_field")]
    pub json_timestamp_field: String,
    #[serde(default = "default_message_field")]
    pub json_message_field: String,
    #[serde(default = "default_level_field")]
    pub json_level_field: String,
}

fn default_true() -> bool { true }
fn default_timestamp_format() -> String { "auto".to_string() }
fn default_error_color() -> String { "red".to_string() }
fn default_warn_color() -> String { "yellow".to_string() }
fn default_info_color() -> String { "default".to_string() }
fn default_debug_color() -> String { "dim".to_string() }
fn default_trace_color() -> String { "dim".to_string() }
fn default_timestamp_field() -> String { "timestamp".to_string() }
fn default_message_field() -> String { "message".to_string() }
fn default_level_field() -> String { "level".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            colors: ColorConfig::default(),
            parsing: ParsingConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            line_numbers: true,
            wrap_lines: false,
            timestamp_format: "auto".to_string(),
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            error: "red".to_string(),
            warn: "yellow".to_string(),
            info: "default".to_string(),
            debug: "dim".to_string(),
            trace: "dim".to_string(),
        }
    }
}

impl Default for ParsingConfig {
    fn default() -> Self {
        Self {
            detect_format: true,
            json_timestamp_field: "timestamp".to_string(),
            json_message_field: "message".to_string(),
            json_level_field: "level".to_string(),
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
        directories::ProjectDirs::from("", "", "log-viewer")
            .map(|p| p.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
