use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::time::Duration;

use crate::config::*;
use crate::models::*;

#[async_trait]
pub trait CIProvider: Send + Sync {
    fn system(&self) -> CISystem;
    async fn list_repos(&self) -> Result<Vec<Repository>>;
    async fn list_workflows(&self, repo: &str) -> Result<Vec<Workflow>>;
    async fn get_runs(&self, repo: &str, limit: usize) -> Result<Vec<Run>>;
    async fn get_run_details(&self, run_id: &str) -> Result<Run>;
    async fn get_job_logs(&self, job_id: &str) -> Result<String>;
    async fn retry_run(&self, run_id: &str) -> Result<()>;
    async fn cancel_run(&self, run_id: &str) -> Result<()>;
}

pub struct GitHubProvider {
    config: GitHubConfig,
}

impl GitHubProvider {
    pub fn new(config: GitHubConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CIProvider for GitHubProvider {
    fn system(&self) -> CISystem {
        CISystem::GitHub
    }

    async fn list_repos(&self) -> Result<Vec<Repository>> {
        Ok(self
            .config
            .repos
            .iter()
            .map(|r| {
                let name = r.split('/').last().unwrap_or(r);
                Repository::new(name, r, CISystem::GitHub)
            })
            .collect())
    }

    async fn list_workflows(&self, repo: &str) -> Result<Vec<Workflow>> {
        // Simulated response
        Ok(vec![
            Workflow::new("1", "Build", repo, CISystem::GitHub),
            Workflow::new("2", "Test", repo, CISystem::GitHub),
            Workflow::new("3", "Deploy", repo, CISystem::GitHub),
        ])
    }

    async fn get_runs(&self, repo: &str, _limit: usize) -> Result<Vec<Run>> {
        // Simulated response with demo data
        let mut runs = Vec::new();

        let mut run1 = Run::new("gh-1", "Build", repo, CISystem::GitHub);
        run1.branch = "main".to_string();
        run1.commit_sha = "abc1234".to_string();
        run1.commit_message = "Add new feature".to_string();
        run1.status = RunStatus::Completed;
        run1.conclusion = Some(Conclusion::Success);
        run1.duration = Some(Duration::from_secs(154));
        run1.jobs = vec![
            {
                let mut job = Job::new("j1", "build");
                job.status = RunStatus::Completed;
                job.conclusion = Some(Conclusion::Success);
                job
            },
            {
                let mut job = Job::new("j2", "test");
                job.status = RunStatus::Completed;
                job.conclusion = Some(Conclusion::Success);
                job
            },
        ];
        runs.push(run1);

        let mut run2 = Run::new("gh-2", "Test", repo, CISystem::GitHub);
        run2.branch = "feature/auth".to_string();
        run2.commit_sha = "def5678".to_string();
        run2.commit_message = "Fix authentication bug".to_string();
        run2.status = RunStatus::InProgress;
        run2.started_at = Utc::now() - chrono::Duration::seconds(90);
        run2.jobs = vec![{
            let mut job = Job::new("j3", "test");
            job.status = RunStatus::InProgress;
            job
        }];
        runs.push(run2);

        let mut run3 = Run::new("gh-3", "Deploy", repo, CISystem::GitHub);
        run3.branch = "main".to_string();
        run3.status = RunStatus::Completed;
        run3.conclusion = Some(Conclusion::Failure);
        run3.duration = Some(Duration::from_secs(45));
        runs.push(run3);

        Ok(runs)
    }

    async fn get_run_details(&self, run_id: &str) -> Result<Run> {
        let mut run = Run::new(run_id, "Build", "org/repo", CISystem::GitHub);
        run.status = RunStatus::Completed;
        run.conclusion = Some(Conclusion::Success);
        Ok(run)
    }

    async fn get_job_logs(&self, _job_id: &str) -> Result<String> {
        Ok("Build started...\nCompiling...\nRunning tests...\nBuild successful!".to_string())
    }

    async fn retry_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }

    async fn cancel_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }
}

pub struct GitLabProvider {
    config: GitLabConfig,
}

impl GitLabProvider {
    pub fn new(config: GitLabConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CIProvider for GitLabProvider {
    fn system(&self) -> CISystem {
        CISystem::GitLab
    }

    async fn list_repos(&self) -> Result<Vec<Repository>> {
        Ok(self
            .config
            .projects
            .iter()
            .map(|p| {
                let name = p.split('/').last().unwrap_or(p);
                Repository::new(name, p, CISystem::GitLab)
            })
            .collect())
    }

    async fn list_workflows(&self, repo: &str) -> Result<Vec<Workflow>> {
        Ok(vec![Workflow::new("1", "Pipeline", repo, CISystem::GitLab)])
    }

    async fn get_runs(&self, repo: &str, _limit: usize) -> Result<Vec<Run>> {
        let mut run = Run::new("gl-1", "Pipeline", repo, CISystem::GitLab);
        run.status = RunStatus::Completed;
        run.conclusion = Some(Conclusion::Success);
        run.duration = Some(Duration::from_secs(210));
        Ok(vec![run])
    }

    async fn get_run_details(&self, run_id: &str) -> Result<Run> {
        let mut run = Run::new(run_id, "Pipeline", "group/project", CISystem::GitLab);
        run.status = RunStatus::Completed;
        run.conclusion = Some(Conclusion::Success);
        Ok(run)
    }

    async fn get_job_logs(&self, _job_id: &str) -> Result<String> {
        Ok("Pipeline started...\nJob running...\nCompleted.".to_string())
    }

    async fn retry_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }

    async fn cancel_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }
}

pub struct JenkinsProvider {
    config: JenkinsConfig,
}

impl JenkinsProvider {
    pub fn new(config: JenkinsConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CIProvider for JenkinsProvider {
    fn system(&self) -> CISystem {
        CISystem::Jenkins
    }

    async fn list_repos(&self) -> Result<Vec<Repository>> {
        Ok(self
            .config
            .jobs
            .iter()
            .map(|j| Repository::new(j, j, CISystem::Jenkins))
            .collect())
    }

    async fn list_workflows(&self, repo: &str) -> Result<Vec<Workflow>> {
        Ok(vec![Workflow::new("1", repo, repo, CISystem::Jenkins)])
    }

    async fn get_runs(&self, repo: &str, _limit: usize) -> Result<Vec<Run>> {
        let mut run = Run::new("jk-1", repo, repo, CISystem::Jenkins);
        run.status = RunStatus::Completed;
        run.conclusion = Some(Conclusion::Success);
        run.duration = Some(Duration::from_secs(180));
        Ok(vec![run])
    }

    async fn get_run_details(&self, run_id: &str) -> Result<Run> {
        let mut run = Run::new(run_id, "Build", "job", CISystem::Jenkins);
        run.status = RunStatus::Completed;
        run.conclusion = Some(Conclusion::Success);
        Ok(run)
    }

    async fn get_job_logs(&self, _job_id: &str) -> Result<String> {
        Ok("Started by user...\nBuilding...\nFinished: SUCCESS".to_string())
    }

    async fn retry_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }

    async fn cancel_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }
}

pub struct CircleCIProvider {
    config: CircleCIConfig,
}

impl CircleCIProvider {
    pub fn new(config: CircleCIConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CIProvider for CircleCIProvider {
    fn system(&self) -> CISystem {
        CISystem::CircleCI
    }

    async fn list_repos(&self) -> Result<Vec<Repository>> {
        Ok(self
            .config
            .projects
            .iter()
            .map(|p| {
                let name = p.split('/').last().unwrap_or(p);
                Repository::new(name, p, CISystem::CircleCI)
            })
            .collect())
    }

    async fn list_workflows(&self, repo: &str) -> Result<Vec<Workflow>> {
        Ok(vec![Workflow::new(
            "1",
            "build-test-deploy",
            repo,
            CISystem::CircleCI,
        )])
    }

    async fn get_runs(&self, repo: &str, _limit: usize) -> Result<Vec<Run>> {
        let mut run = Run::new("cc-1", "build-test-deploy", repo, CISystem::CircleCI);
        run.status = RunStatus::Completed;
        run.conclusion = Some(Conclusion::Success);
        run.duration = Some(Duration::from_secs(95));
        Ok(vec![run])
    }

    async fn get_run_details(&self, run_id: &str) -> Result<Run> {
        let mut run = Run::new(run_id, "Workflow", "project", CISystem::CircleCI);
        run.status = RunStatus::Completed;
        run.conclusion = Some(Conclusion::Success);
        Ok(run)
    }

    async fn get_job_logs(&self, _job_id: &str) -> Result<String> {
        Ok("Spin up environment...\nRunning...\nSuccess!".to_string())
    }

    async fn retry_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }

    async fn cancel_run(&self, _run_id: &str) -> Result<()> {
        Ok(())
    }
}

pub fn create_providers(config: &Config) -> Vec<Box<dyn CIProvider>> {
    let mut providers: Vec<Box<dyn CIProvider>> = Vec::new();

    // Always add GitHub with demo data for demonstration
    let github_config = if config.has_github() {
        config.github.clone()
    } else {
        GitHubConfig {
            token: "demo".to_string(),
            repos: vec!["org/repo-a".to_string(), "org/repo-b".to_string()],
        }
    };
    providers.push(Box::new(GitHubProvider::new(github_config)));

    if config.has_gitlab() {
        providers.push(Box::new(GitLabProvider::new(config.gitlab.clone())));
    }

    if config.has_jenkins() {
        providers.push(Box::new(JenkinsProvider::new(config.jenkins.clone())));
    }

    if config.has_circleci() {
        providers.push(Box::new(CircleCIProvider::new(config.circleci.clone())));
    }

    providers
}
