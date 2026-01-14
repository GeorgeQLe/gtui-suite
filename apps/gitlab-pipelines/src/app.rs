use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, PartialEq)]
pub enum PipelineStatus {
    Success,
    Failed,
    Running,
    Pending,
    Cancelled,
    Skipped,
}

impl PipelineStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PipelineStatus::Success => "passed",
            PipelineStatus::Failed => "failed",
            PipelineStatus::Running => "running",
            PipelineStatus::Pending => "pending",
            PipelineStatus::Cancelled => "cancelled",
            PipelineStatus::Skipped => "skipped",
        }
    }
}

#[derive(Clone)]
pub struct Pipeline {
    pub id: u64,
    pub branch: String,
    pub commit: String,
    pub commit_msg: String,
    pub status: PipelineStatus,
    pub stages: Vec<Stage>,
    pub created: String,
    pub duration: String,
    pub author: String,
}

#[derive(Clone)]
pub struct Stage {
    pub name: String,
    pub status: PipelineStatus,
    pub jobs: Vec<Job>,
}

#[derive(Clone)]
pub struct Job {
    pub name: String,
    pub status: PipelineStatus,
    pub duration: String,
    pub runner: String,
}

pub enum View {
    Pipelines,
    Stages,
    Jobs,
}

pub struct App {
    pub pipelines: Vec<Pipeline>,
    pub selected_pipeline: usize,
    pub selected_stage: usize,
    pub selected_job: usize,
    pub current_view: View,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            pipelines: vec![
                Pipeline {
                    id: 98765,
                    branch: "main".to_string(),
                    commit: "a1b2c3d".to_string(),
                    commit_msg: "Merge feature/auth into main".to_string(),
                    status: PipelineStatus::Success,
                    stages: vec![
                        Stage {
                            name: "build".to_string(),
                            status: PipelineStatus::Success,
                            jobs: vec![
                                Job {
                                    name: "compile".to_string(),
                                    status: PipelineStatus::Success,
                                    duration: "2m 15s".to_string(),
                                    runner: "runner-1".to_string(),
                                },
                            ],
                        },
                        Stage {
                            name: "test".to_string(),
                            status: PipelineStatus::Success,
                            jobs: vec![
                                Job {
                                    name: "unit-tests".to_string(),
                                    status: PipelineStatus::Success,
                                    duration: "3m 45s".to_string(),
                                    runner: "runner-2".to_string(),
                                },
                                Job {
                                    name: "integration-tests".to_string(),
                                    status: PipelineStatus::Success,
                                    duration: "5m 20s".to_string(),
                                    runner: "runner-3".to_string(),
                                },
                            ],
                        },
                        Stage {
                            name: "deploy".to_string(),
                            status: PipelineStatus::Success,
                            jobs: vec![
                                Job {
                                    name: "deploy-staging".to_string(),
                                    status: PipelineStatus::Success,
                                    duration: "1m 30s".to_string(),
                                    runner: "runner-1".to_string(),
                                },
                            ],
                        },
                    ],
                    created: "2024-01-15 10:30".to_string(),
                    duration: "12m 50s".to_string(),
                    author: "developer".to_string(),
                },
                Pipeline {
                    id: 98764,
                    branch: "feature/api".to_string(),
                    commit: "e4f5g6h".to_string(),
                    commit_msg: "Add new API endpoints".to_string(),
                    status: PipelineStatus::Running,
                    stages: vec![
                        Stage {
                            name: "build".to_string(),
                            status: PipelineStatus::Success,
                            jobs: vec![
                                Job {
                                    name: "compile".to_string(),
                                    status: PipelineStatus::Success,
                                    duration: "2m 10s".to_string(),
                                    runner: "runner-1".to_string(),
                                },
                            ],
                        },
                        Stage {
                            name: "test".to_string(),
                            status: PipelineStatus::Running,
                            jobs: vec![
                                Job {
                                    name: "unit-tests".to_string(),
                                    status: PipelineStatus::Running,
                                    duration: "1m 25s".to_string(),
                                    runner: "runner-2".to_string(),
                                },
                            ],
                        },
                    ],
                    created: "2024-01-15 10:25".to_string(),
                    duration: "3m 35s".to_string(),
                    author: "contributor".to_string(),
                },
                Pipeline {
                    id: 98763,
                    branch: "fix/login".to_string(),
                    commit: "i7j8k9l".to_string(),
                    commit_msg: "Fix login redirect issue".to_string(),
                    status: PipelineStatus::Failed,
                    stages: vec![
                        Stage {
                            name: "build".to_string(),
                            status: PipelineStatus::Success,
                            jobs: vec![
                                Job {
                                    name: "compile".to_string(),
                                    status: PipelineStatus::Success,
                                    duration: "2m 05s".to_string(),
                                    runner: "runner-1".to_string(),
                                },
                            ],
                        },
                        Stage {
                            name: "test".to_string(),
                            status: PipelineStatus::Failed,
                            jobs: vec![
                                Job {
                                    name: "unit-tests".to_string(),
                                    status: PipelineStatus::Failed,
                                    duration: "1m 45s".to_string(),
                                    runner: "runner-2".to_string(),
                                },
                            ],
                        },
                    ],
                    created: "2024-01-15 09:00".to_string(),
                    duration: "3m 50s".to_string(),
                    author: "developer".to_string(),
                },
                Pipeline {
                    id: 98762,
                    branch: "main".to_string(),
                    commit: "m0n1o2p".to_string(),
                    commit_msg: "Update dependencies".to_string(),
                    status: PipelineStatus::Success,
                    stages: vec![
                        Stage {
                            name: "build".to_string(),
                            status: PipelineStatus::Success,
                            jobs: vec![
                                Job {
                                    name: "compile".to_string(),
                                    status: PipelineStatus::Success,
                                    duration: "2m 30s".to_string(),
                                    runner: "runner-1".to_string(),
                                },
                            ],
                        },
                    ],
                    created: "2024-01-14 16:00".to_string(),
                    duration: "2m 30s".to_string(),
                    author: "maintainer".to_string(),
                },
            ],
            selected_pipeline: 0,
            selected_stage: 0,
            selected_job: 0,
            current_view: View::Pipelines,
            show_help: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match &self.current_view {
            View::Pipelines => self.handle_pipelines_key(key),
            View::Stages => self.handle_stages_key(key),
            View::Jobs => self.handle_jobs_key(key),
        }
    }

    fn handle_pipelines_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_pipeline < self.pipelines.len().saturating_sub(1) {
                    self.selected_pipeline += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_pipeline = self.selected_pipeline.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.selected_stage = 0;
                self.current_view = View::Stages;
            }
            KeyCode::Char('r') => {
                // Retry pipeline (demo)
            }
            KeyCode::Char('c') => {
                // Cancel pipeline (demo)
            }
            _ => {}
        }
        false
    }

    fn handle_stages_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Pipelines;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let pipeline = &self.pipelines[self.selected_pipeline];
                if self.selected_stage < pipeline.stages.len().saturating_sub(1) {
                    self.selected_stage += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_stage = self.selected_stage.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.selected_job = 0;
                self.current_view = View::Jobs;
            }
            _ => {}
        }
        false
    }

    fn handle_jobs_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Stages;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let pipeline = &self.pipelines[self.selected_pipeline];
                let stage = &pipeline.stages[self.selected_stage];
                if self.selected_job < stage.jobs.len().saturating_sub(1) {
                    self.selected_job += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_job = self.selected_job.saturating_sub(1);
            }
            KeyCode::Char('l') => {
                // View logs (demo)
            }
            KeyCode::Char('r') => {
                // Retry job (demo)
            }
            _ => {}
        }
        false
    }

    pub fn current_pipeline(&self) -> &Pipeline {
        &self.pipelines[self.selected_pipeline]
    }

    pub fn current_stage(&self) -> &Stage {
        &self.pipelines[self.selected_pipeline].stages[self.selected_stage]
    }
}
