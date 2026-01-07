#![allow(dead_code)]

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub diff: DiffConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_false")]
    pub sign_commits: bool,
    #[serde(default = "default_branch")]
    pub default_branch: String,
}

fn default_false() -> bool {
    false
}

fn default_branch() -> String {
    "main".to_string()
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            sign_commits: false,
            default_branch: "main".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    #[serde(default = "default_diff_view")]
    pub view: String,
    #[serde(default = "default_true")]
    pub word_level: bool,
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
}

fn default_diff_view() -> String {
    "unified".to_string()
}

fn default_true() -> bool {
    true
}

fn default_context_lines() -> usize {
    3
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            view: "unified".to_string(),
            word_level: true,
            context_lines: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub show_untracked: bool,
    #[serde(default = "default_true")]
    pub confirm_dangerous: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_untracked: true,
            confirm_dangerous: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            git: GitConfig::default(),
            diff: DiffConfig::default(),
            ui: UiConfig::default(),
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
        let dirs = ProjectDirs::from("com", "tui-suite", "git-client")
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let config_dir = dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("config.toml"))
    }
}
