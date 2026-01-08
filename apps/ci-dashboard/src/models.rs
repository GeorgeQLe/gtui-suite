use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CISystem {
    GitHub,
    GitLab,
    Jenkins,
    CircleCI,
}

impl CISystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            CISystem::GitHub => "GitHub Actions",
            CISystem::GitLab => "GitLab CI",
            CISystem::Jenkins => "Jenkins",
            CISystem::CircleCI => "CircleCI",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CISystem::GitHub => "",
            CISystem::GitLab => "",
            CISystem::Jenkins => "",
            CISystem::CircleCI => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub system: CISystem,
    pub url: String,
}

impl Repository {
    pub fn new(name: &str, full_name: &str, system: CISystem) -> Self {
        Self {
            id: format!("{:?}:{}", system, full_name),
            name: name.to_string(),
            full_name: full_name.to_string(),
            system,
            url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub path: String,
    pub repo: String,
    pub system: CISystem,
}

impl Workflow {
    pub fn new(id: &str, name: &str, repo: &str, system: CISystem) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            path: String::new(),
            repo: repo.to_string(),
            system,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Queued,
    InProgress,
    Completed,
}

impl RunStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            RunStatus::Queued => "○",
            RunStatus::InProgress => "⟳",
            RunStatus::Completed => "●",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Conclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
}

impl Conclusion {
    pub fn icon(&self) -> &'static str {
        match self {
            Conclusion::Success => "✓",
            Conclusion::Failure => "✗",
            Conclusion::Cancelled => "⊘",
            Conclusion::Skipped => "⊖",
            Conclusion::TimedOut => "⏱",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Conclusion::Success => "Success",
            Conclusion::Failure => "Failed",
            Conclusion::Cancelled => "Cancelled",
            Conclusion::Skipped => "Skipped",
            Conclusion::TimedOut => "Timed Out",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub repo: String,
    pub branch: String,
    pub commit_sha: String,
    pub commit_message: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub jobs: Vec<Job>,
    pub system: CISystem,
    pub url: String,
}

impl Run {
    pub fn new(id: &str, workflow_name: &str, repo: &str, system: CISystem) -> Self {
        Self {
            id: id.to_string(),
            workflow_id: String::new(),
            workflow_name: workflow_name.to_string(),
            repo: repo.to_string(),
            branch: "main".to_string(),
            commit_sha: String::new(),
            commit_message: String::new(),
            status: RunStatus::Queued,
            conclusion: None,
            started_at: Utc::now(),
            finished_at: None,
            duration: None,
            jobs: Vec::new(),
            system,
            url: String::new(),
        }
    }

    pub fn display_status(&self) -> String {
        match self.status {
            RunStatus::Completed => {
                if let Some(conclusion) = &self.conclusion {
                    format!("{} {}", conclusion.icon(), conclusion.as_str())
                } else {
                    "● Completed".to_string()
                }
            }
            RunStatus::InProgress => "⟳ Running".to_string(),
            RunStatus::Queued => "○ Queued".to_string(),
        }
    }

    pub fn duration_display(&self) -> String {
        if let Some(duration) = self.duration {
            let secs = duration.as_secs();
            if secs >= 3600 {
                format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
            } else if secs >= 60 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            }
        } else if self.status == RunStatus::InProgress {
            let elapsed = Utc::now().signed_duration_since(self.started_at);
            let secs = elapsed.num_seconds() as u64;
            if secs >= 60 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            }
        } else {
            "-".to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub steps: Vec<Step>,
}

impl Job {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: RunStatus::Queued,
            conclusion: None,
            started_at: None,
            finished_at: None,
            steps: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub expired: bool,
}

impl Artifact {
    pub fn size_display(&self) -> String {
        let size = self.size_bytes;
        if size >= 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else if size >= 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{} B", size)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub system: CISystem,
    pub connected: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl SystemStatus {
    pub fn new(system: CISystem) -> Self {
        Self {
            system,
            connected: false,
            last_updated: None,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_duration_display() {
        let mut run = Run::new("1", "build", "org/repo", CISystem::GitHub);
        run.duration = Some(Duration::from_secs(125));
        assert_eq!(run.duration_display(), "2m 5s");

        run.duration = Some(Duration::from_secs(3665));
        assert_eq!(run.duration_display(), "1h 1m");
    }

    #[test]
    fn test_conclusion_display() {
        assert_eq!(Conclusion::Success.as_str(), "Success");
        assert_eq!(Conclusion::Failure.as_str(), "Failed");
    }

    #[test]
    fn test_artifact_size() {
        let artifact = Artifact {
            id: "1".to_string(),
            name: "test.zip".to_string(),
            size_bytes: 1536,
            expired: false,
        };
        assert_eq!(artifact.size_display(), "1.5 KB");
    }
}
