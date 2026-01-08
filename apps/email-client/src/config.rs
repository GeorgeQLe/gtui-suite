use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub accounts: Vec<AccountConfig>,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub compose: ComposeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub name: String,
    pub email: String,
    pub imap_host: String,
    #[serde(default = "default_imap_port")]
    pub imap_port: u16,
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    pub username: String,
    #[serde(default)]
    pub password_cmd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default = "default_preview_lines")]
    pub preview_lines: usize,
    #[serde(default = "default_true")]
    pub show_sidebar: bool,
    #[serde(default = "default_true")]
    pub thread_view: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeConfig {
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub default_cc: Option<String>,
    #[serde(default = "default_true")]
    pub quote_reply: bool,
}

fn default_imap_port() -> u16 {
    993
}

fn default_smtp_port() -> u16 {
    587
}

fn default_date_format() -> String {
    "%Y-%m-%d %H:%M".to_string()
}

fn default_page_size() -> usize {
    50
}

fn default_preview_lines() -> usize {
    2
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            accounts: vec![],
            display: DisplayConfig::default(),
            compose: ComposeConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            date_format: default_date_format(),
            page_size: default_page_size(),
            preview_lines: default_preview_lines(),
            show_sidebar: true,
            thread_view: true,
        }
    }
}

impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            signature: None,
            default_cc: None,
            quote_reply: true,
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

    pub fn config_path() -> Result<PathBuf> {
        ProjectDirs::from("", "", "email-client")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}
