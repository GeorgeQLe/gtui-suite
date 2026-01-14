use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, PartialEq)]
pub enum WorkflowStatus {
    Success,
    Failure,
    InProgress,
    Queued,
    Cancelled,
}

impl WorkflowStatus {
    pub fn as_str(&self) -> &str {
        match self {
            WorkflowStatus::Success => "success",
            WorkflowStatus::Failure => "failure",
            WorkflowStatus::InProgress => "in_progress",
            WorkflowStatus::Queued => "queued",
            WorkflowStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone)]
pub struct WorkflowRun {
    pub id: u64,
    pub workflow: String,
    pub branch: String,
    pub commit: String,
    pub status: WorkflowStatus,
    pub conclusion: Option<String>,
    pub started: String,
    pub duration: String,
    pub actor: String,
}

#[derive(Clone)]
pub struct Job {
    pub name: String,
    pub status: WorkflowStatus,
    pub started: String,
    pub duration: String,
    pub steps: Vec<Step>,
}

#[derive(Clone)]
pub struct Step {
    pub name: String,
    pub status: WorkflowStatus,
    pub duration: String,
}

pub enum View {
    Runs,
    Jobs,
    Logs,
}

pub struct App {
    pub runs: Vec<WorkflowRun>,
    pub jobs: Vec<Job>,
    pub selected_run: usize,
    pub selected_job: usize,
    pub current_view: View,
    pub show_help: bool,
    pub filter_branch: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            runs: vec![
                WorkflowRun {
                    id: 12345,
                    workflow: "CI".to_string(),
                    branch: "main".to_string(),
                    commit: "abc1234".to_string(),
                    status: WorkflowStatus::Success,
                    conclusion: Some("success".to_string()),
                    started: "2024-01-15 10:30".to_string(),
                    duration: "5m 23s".to_string(),
                    actor: "developer".to_string(),
                },
                WorkflowRun {
                    id: 12344,
                    workflow: "CI".to_string(),
                    branch: "feature/auth".to_string(),
                    commit: "def5678".to_string(),
                    status: WorkflowStatus::InProgress,
                    conclusion: None,
                    started: "2024-01-15 10:25".to_string(),
                    duration: "2m 15s".to_string(),
                    actor: "contributor".to_string(),
                },
                WorkflowRun {
                    id: 12343,
                    workflow: "Deploy".to_string(),
                    branch: "main".to_string(),
                    commit: "ghi9012".to_string(),
                    status: WorkflowStatus::Success,
                    conclusion: Some("success".to_string()),
                    started: "2024-01-15 09:00".to_string(),
                    duration: "12m 45s".to_string(),
                    actor: "developer".to_string(),
                },
                WorkflowRun {
                    id: 12342,
                    workflow: "CI".to_string(),
                    branch: "fix/bug-123".to_string(),
                    commit: "jkl3456".to_string(),
                    status: WorkflowStatus::Failure,
                    conclusion: Some("failure".to_string()),
                    started: "2024-01-15 08:30".to_string(),
                    duration: "3m 12s".to_string(),
                    actor: "developer".to_string(),
                },
                WorkflowRun {
                    id: 12341,
                    workflow: "Release".to_string(),
                    branch: "main".to_string(),
                    commit: "mno7890".to_string(),
                    status: WorkflowStatus::Success,
                    conclusion: Some("success".to_string()),
                    started: "2024-01-14 16:00".to_string(),
                    duration: "8m 34s".to_string(),
                    actor: "maintainer".to_string(),
                },
                WorkflowRun {
                    id: 12340,
                    workflow: "CI".to_string(),
                    branch: "dependabot/npm".to_string(),
                    commit: "pqr1234".to_string(),
                    status: WorkflowStatus::Cancelled,
                    conclusion: Some("cancelled".to_string()),
                    started: "2024-01-14 14:00".to_string(),
                    duration: "1m 05s".to_string(),
                    actor: "dependabot".to_string(),
                },
            ],
            jobs: Vec::new(),
            selected_run: 0,
            selected_job: 0,
            current_view: View::Runs,
            show_help: false,
            filter_branch: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match &self.current_view {
            View::Runs => self.handle_runs_key(key),
            View::Jobs => self.handle_jobs_key(key),
            View::Logs => self.handle_logs_key(key),
        }
    }

    fn handle_runs_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_run < self.runs.len().saturating_sub(1) {
                    self.selected_run += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_run = self.selected_run.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.load_jobs();
                self.current_view = View::Jobs;
            }
            KeyCode::Char('r') => {
                // Re-run workflow (demo)
            }
            KeyCode::Char('c') => {
                // Cancel workflow (demo)
            }
            KeyCode::Char('f') => {
                // Filter by branch (demo)
            }
            _ => {}
        }
        false
    }

    fn handle_jobs_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Runs;
                self.jobs.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_job < self.jobs.len().saturating_sub(1) {
                    self.selected_job += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_job = self.selected_job.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !self.jobs.is_empty() {
                    self.current_view = View::Logs;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_logs_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Jobs;
            }
            _ => {}
        }
        false
    }

    fn load_jobs(&mut self) {
        self.selected_job = 0;
        let run = &self.runs[self.selected_run];

        self.jobs = match run.workflow.as_str() {
            "CI" => vec![
                Job {
                    name: "build".to_string(),
                    status: run.status.clone(),
                    started: run.started.clone(),
                    duration: "2m 10s".to_string(),
                    steps: vec![
                        Step {
                            name: "Checkout".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "5s".to_string(),
                        },
                        Step {
                            name: "Setup Node.js".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "15s".to_string(),
                        },
                        Step {
                            name: "Install dependencies".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "45s".to_string(),
                        },
                        Step {
                            name: "Build".to_string(),
                            status: run.status.clone(),
                            duration: "1m 5s".to_string(),
                        },
                    ],
                },
                Job {
                    name: "test".to_string(),
                    status: run.status.clone(),
                    started: run.started.clone(),
                    duration: "3m 13s".to_string(),
                    steps: vec![
                        Step {
                            name: "Checkout".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "5s".to_string(),
                        },
                        Step {
                            name: "Setup Node.js".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "15s".to_string(),
                        },
                        Step {
                            name: "Run tests".to_string(),
                            status: run.status.clone(),
                            duration: "2m 53s".to_string(),
                        },
                    ],
                },
            ],
            "Deploy" => vec![
                Job {
                    name: "deploy-staging".to_string(),
                    status: WorkflowStatus::Success,
                    started: run.started.clone(),
                    duration: "5m 20s".to_string(),
                    steps: vec![
                        Step {
                            name: "Checkout".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "5s".to_string(),
                        },
                        Step {
                            name: "Deploy to staging".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "5m 15s".to_string(),
                        },
                    ],
                },
                Job {
                    name: "deploy-production".to_string(),
                    status: run.status.clone(),
                    started: run.started.clone(),
                    duration: "7m 25s".to_string(),
                    steps: vec![
                        Step {
                            name: "Checkout".to_string(),
                            status: WorkflowStatus::Success,
                            duration: "5s".to_string(),
                        },
                        Step {
                            name: "Deploy to production".to_string(),
                            status: run.status.clone(),
                            duration: "7m 20s".to_string(),
                        },
                    ],
                },
            ],
            _ => vec![Job {
                name: "release".to_string(),
                status: run.status.clone(),
                started: run.started.clone(),
                duration: run.duration.clone(),
                steps: vec![
                    Step {
                        name: "Checkout".to_string(),
                        status: WorkflowStatus::Success,
                        duration: "5s".to_string(),
                    },
                    Step {
                        name: "Build release".to_string(),
                        status: WorkflowStatus::Success,
                        duration: "3m 00s".to_string(),
                    },
                    Step {
                        name: "Publish".to_string(),
                        status: run.status.clone(),
                        duration: "5m 29s".to_string(),
                    },
                ],
            }],
        };
    }

    pub fn current_run(&self) -> &WorkflowRun {
        &self.runs[self.selected_run]
    }

    pub fn current_job(&self) -> Option<&Job> {
        self.jobs.get(self.selected_job)
    }
}
