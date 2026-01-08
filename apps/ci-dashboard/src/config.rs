use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub gitlab: GitLabConfig,
    #[serde(default)]
    pub jenkins: JenkinsConfig,
    #[serde(default)]
    pub circleci: CircleCIConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub repos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    #[serde(default = "default_gitlab_url")]
    pub url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub projects: Vec<String>,
}

impl Default for GitLabConfig {
    fn default() -> Self {
        Self {
            url: default_gitlab_url(),
            token: String::new(),
            projects: Vec::new(),
        }
    }
}

fn default_gitlab_url() -> String {
    "https://gitlab.com".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JenkinsConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub jobs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircleCIConfig {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub on_failure: bool,
    #[serde(default)]
    pub sound: bool,
    #[serde(default)]
    pub command: Option<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            on_failure: true,
            sound: false,
            command: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_refresh")]
    pub refresh_secs: u64,
    #[serde(default = "default_true")]
    pub show_passed: bool,
    #[serde(default = "default_max_runs")]
    pub max_runs: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_secs: default_refresh(),
            show_passed: true,
            max_runs: default_max_runs(),
        }
    }
}

fn default_refresh() -> u64 {
    60
}

fn default_max_runs() -> usize {
    50
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: GitHubConfig::default(),
            gitlab: GitLabConfig::default(),
            jenkins: JenkinsConfig::default(),
            circleci: CircleCIConfig::default(),
            notifications: NotificationConfig::default(),
            display: DisplayConfig::default(),
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
        ProjectDirs::from("", "", "ci-dashboard")
            .map(|p| p.config_dir().join("config.toml"))
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }

    pub fn has_github(&self) -> bool {
        !self.github.token.is_empty() && !self.github.repos.is_empty()
    }

    pub fn has_gitlab(&self) -> bool {
        !self.gitlab.token.is_empty() && !self.gitlab.projects.is_empty()
    }

    pub fn has_jenkins(&self) -> bool {
        !self.jenkins.url.is_empty() && !self.jenkins.jobs.is_empty()
    }

    pub fn has_circleci(&self) -> bool {
        !self.circleci.token.is_empty() && !self.circleci.projects.is_empty()
    }
}
