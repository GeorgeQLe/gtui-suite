use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, PartialEq)]
pub enum BuildStatus {
    Success,
    Failure,
    Unstable,
    Building,
    Aborted,
    NotBuilt,
}

impl BuildStatus {
    pub fn as_str(&self) -> &str {
        match self {
            BuildStatus::Success => "SUCCESS",
            BuildStatus::Failure => "FAILURE",
            BuildStatus::Unstable => "UNSTABLE",
            BuildStatus::Building => "BUILDING",
            BuildStatus::Aborted => "ABORTED",
            BuildStatus::NotBuilt => "NOT_BUILT",
        }
    }
}

#[derive(Clone)]
pub struct JenkinsJob {
    pub name: String,
    pub folder: String,
    pub last_build: u32,
    pub status: BuildStatus,
    pub duration: String,
    pub timestamp: String,
    pub health: u8,
}

#[derive(Clone)]
pub struct Build {
    pub number: u32,
    pub status: BuildStatus,
    pub duration: String,
    pub timestamp: String,
    pub cause: String,
}

pub enum View {
    Jobs,
    Builds,
    Console,
}

pub struct App {
    pub jobs: Vec<JenkinsJob>,
    pub builds: Vec<Build>,
    pub selected_job: usize,
    pub selected_build: usize,
    pub current_view: View,
    pub show_help: bool,
    pub console_output: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            jobs: vec![
                JenkinsJob {
                    name: "api-service".to_string(),
                    folder: "backend".to_string(),
                    last_build: 156,
                    status: BuildStatus::Success,
                    duration: "3m 45s".to_string(),
                    timestamp: "2024-01-15 10:30".to_string(),
                    health: 100,
                },
                JenkinsJob {
                    name: "web-frontend".to_string(),
                    folder: "frontend".to_string(),
                    last_build: 89,
                    status: BuildStatus::Building,
                    duration: "2m 15s".to_string(),
                    timestamp: "2024-01-15 10:28".to_string(),
                    health: 80,
                },
                JenkinsJob {
                    name: "database-migrations".to_string(),
                    folder: "backend".to_string(),
                    last_build: 45,
                    status: BuildStatus::Success,
                    duration: "1m 20s".to_string(),
                    timestamp: "2024-01-15 09:00".to_string(),
                    health: 100,
                },
                JenkinsJob {
                    name: "integration-tests".to_string(),
                    folder: "qa".to_string(),
                    last_build: 234,
                    status: BuildStatus::Failure,
                    duration: "15m 30s".to_string(),
                    timestamp: "2024-01-15 08:45".to_string(),
                    health: 40,
                },
                JenkinsJob {
                    name: "deploy-staging".to_string(),
                    folder: "deploy".to_string(),
                    last_build: 78,
                    status: BuildStatus::Success,
                    duration: "5m 10s".to_string(),
                    timestamp: "2024-01-14 16:00".to_string(),
                    health: 80,
                },
                JenkinsJob {
                    name: "deploy-production".to_string(),
                    folder: "deploy".to_string(),
                    last_build: 56,
                    status: BuildStatus::Success,
                    duration: "8m 25s".to_string(),
                    timestamp: "2024-01-14 14:30".to_string(),
                    health: 100,
                },
                JenkinsJob {
                    name: "security-scan".to_string(),
                    folder: "security".to_string(),
                    last_build: 12,
                    status: BuildStatus::Unstable,
                    duration: "20m 00s".to_string(),
                    timestamp: "2024-01-14 12:00".to_string(),
                    health: 60,
                },
            ],
            builds: Vec::new(),
            selected_job: 0,
            selected_build: 0,
            current_view: View::Jobs,
            show_help: false,
            console_output: Vec::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match &self.current_view {
            View::Jobs => self.handle_jobs_key(key),
            View::Builds => self.handle_builds_key(key),
            View::Console => self.handle_console_key(key),
        }
    }

    fn handle_jobs_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_job < self.jobs.len().saturating_sub(1) {
                    self.selected_job += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_job = self.selected_job.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.load_builds();
                self.current_view = View::Builds;
            }
            KeyCode::Char('b') => {
                // Trigger build (demo)
            }
            KeyCode::Char('r') => {
                // Refresh (demo)
            }
            _ => {}
        }
        false
    }

    fn handle_builds_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Jobs;
                self.builds.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_build < self.builds.len().saturating_sub(1) {
                    self.selected_build += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_build = self.selected_build.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('c') => {
                self.load_console();
                self.current_view = View::Console;
            }
            KeyCode::Char('a') => {
                // Abort build (demo)
            }
            _ => {}
        }
        false
    }

    fn handle_console_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_view = View::Builds;
                self.console_output.clear();
            }
            _ => {}
        }
        false
    }

    fn load_builds(&mut self) {
        self.selected_build = 0;
        let job = &self.jobs[self.selected_job];

        self.builds = (0..10)
            .map(|i| {
                let num = job.last_build - i;
                let status = if i == 0 {
                    job.status.clone()
                } else if i % 5 == 0 {
                    BuildStatus::Failure
                } else if i % 7 == 0 {
                    BuildStatus::Unstable
                } else {
                    BuildStatus::Success
                };

                Build {
                    number: num,
                    status,
                    duration: format!("{}m {}s", 2 + (i % 5), 10 + (i * 7) % 50),
                    timestamp: format!("2024-01-{:02} {:02}:00", 15 - (i / 3), 10 - (i % 10)),
                    cause: if i == 0 {
                        "Started by user admin".to_string()
                    } else if i % 2 == 0 {
                        "Started by SCM change".to_string()
                    } else {
                        "Started by timer".to_string()
                    },
                }
            })
            .collect();
    }

    fn load_console(&mut self) {
        let job = &self.jobs[self.selected_job];
        let build = &self.builds[self.selected_build];

        self.console_output = vec![
            format!("Started by {}", build.cause),
            format!("Building in workspace /var/jenkins/workspace/{}", job.name),
            "[Pipeline] Start of Pipeline".to_string(),
            "[Pipeline] node".to_string(),
            "Running on Jenkins in /var/jenkins/workspace/...".to_string(),
            "[Pipeline] {".to_string(),
            "[Pipeline] stage".to_string(),
            "[Pipeline] { (Build)".to_string(),
            "[Pipeline] sh".to_string(),
            "+ npm install".to_string(),
            "added 1234 packages in 45s".to_string(),
            "+ npm run build".to_string(),
            "Build successful".to_string(),
            "[Pipeline] }".to_string(),
            "[Pipeline] // stage".to_string(),
            "[Pipeline] stage".to_string(),
            "[Pipeline] { (Test)".to_string(),
            "[Pipeline] sh".to_string(),
            "+ npm test".to_string(),
            "PASS  src/App.test.js".to_string(),
            "Tests: 42 passed, 42 total".to_string(),
            "[Pipeline] }".to_string(),
            "[Pipeline] // stage".to_string(),
            format!("Finished: {}", build.status.as_str()),
        ];
    }

    pub fn current_job(&self) -> &JenkinsJob {
        &self.jobs[self.selected_job]
    }
}
