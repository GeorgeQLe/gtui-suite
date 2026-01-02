//! Configuration for flashcard trainer.

use crate::models::{SessionConfig, StudyOrder};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub study: StudyConfig,
    #[serde(default)]
    pub algorithm: AlgorithmConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            study: StudyConfig::default(),
            algorithm: AlgorithmConfig::default(),
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
        directories::ProjectDirs::from("", "", "flashcard-trainer")
            .map(|d| d.config_dir().join("config.toml"))
    }

    pub fn db_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "flashcard-trainer")
            .map(|d| d.data_dir().join("flashcards.db"))
    }

    pub fn to_session_config(&self) -> SessionConfig {
        SessionConfig {
            new_cards_per_day: self.study.new_cards_per_day,
            review_limit: self.study.review_limit,
            order: self.study.order,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyConfig {
    #[serde(default = "default_new_cards")]
    pub new_cards_per_day: usize,
    #[serde(default = "default_review_limit")]
    pub review_limit: Option<usize>,
    #[serde(default)]
    pub order: StudyOrder,
    #[serde(default = "default_advance_delay")]
    pub auto_advance_delay_ms: u64,
}

fn default_new_cards() -> usize { 20 }
fn default_review_limit() -> Option<usize> { Some(200) }
fn default_advance_delay() -> u64 { 1000 }

impl Default for StudyConfig {
    fn default() -> Self {
        Self {
            new_cards_per_day: 20,
            review_limit: Some(200),
            order: StudyOrder::DueFirst,
            auto_advance_delay_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    #[serde(default = "default_algorithm")]
    pub default_algorithm: String,
    #[serde(default)]
    pub sm2: Sm2Config,
}

fn default_algorithm() -> String { "sm2".to_string() }

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            default_algorithm: "sm2".to_string(),
            sm2: Sm2Config::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sm2Config {
    #[serde(default = "default_initial_ease")]
    pub initial_ease: f64,
    #[serde(default = "default_easy_bonus")]
    pub easy_bonus: f64,
    #[serde(default = "default_hard_multiplier")]
    pub hard_multiplier: f64,
}

fn default_initial_ease() -> f64 { 2.5 }
fn default_easy_bonus() -> f64 { 1.3 }
fn default_hard_multiplier() -> f64 { 1.2 }

impl Default for Sm2Config {
    fn default() -> Self {
        Self {
            initial_ease: 2.5,
            easy_bonus: 1.3,
            hard_multiplier: 1.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_answer_timer: bool,
    #[serde(default = "default_true")]
    pub show_next_review: bool,
    #[serde(default)]
    pub card_font_size: FontSize,
}

fn default_true() -> bool { true }

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_answer_timer: true,
            show_next_review: true,
            card_font_size: FontSize::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FontSize {
    Small,
    #[default]
    Normal,
    Large,
}
