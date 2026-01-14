use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub templates: TemplatesConfig,
    #[serde(default)]
    pub output: OutputOptions,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            templates: TemplatesConfig::default(),
            output: OutputOptions::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_dir() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "tui-suite", "cli-wizard-generator")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(dirs.config_dir().to_path_buf())
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn templates_dir() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("templates"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            syntax_theme: default_syntax_theme(),
            tab_size: default_tab_size(),
        }
    }
}

fn default_syntax_theme() -> String {
    "monokai".to_string()
}

fn default_tab_size() -> usize {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesConfig {
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub helpers_path: Option<PathBuf>,
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            path: None,
            helpers_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputOptions {
    #[serde(default = "default_true")]
    pub preview_before_write: bool,
    #[serde(default = "default_true")]
    pub backup_existing: bool,
    #[serde(default)]
    pub dry_run: bool,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            preview_before_write: true,
            backup_existing: true,
            dry_run: false,
        }
    }
}

fn default_true() -> bool {
    true
}
