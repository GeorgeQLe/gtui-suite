use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectStatus { Running, Partial, Stopped }

#[derive(Debug, Clone)]
pub struct ComposeService {
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ComposeProject {
    pub name: String,
    pub path: String,
    pub status: ProjectStatus,
    pub services: Vec<ComposeService>,
    pub running_count: usize,
}

pub struct App {
    pub projects: Vec<ComposeProject>,
    pub selected: usize,
    pub expanded: Option<usize>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            projects: vec![
                ComposeProject {
                    name: "web-app".into(), path: "/home/user/projects/web-app".into(), status: ProjectStatus::Running, running_count: 3,
                    services: vec![
                        ComposeService { name: "nginx".into(), image: "nginx:latest".into(), status: "Up 2 hours".into(), ports: vec!["80:80".into(), "443:443".into()] },
                        ComposeService { name: "api".into(), image: "web-app/api:latest".into(), status: "Up 2 hours".into(), ports: vec!["3000:3000".into()] },
                        ComposeService { name: "db".into(), image: "postgres:15".into(), status: "Up 2 hours".into(), ports: vec!["5432:5432".into()] },
                    ],
                },
                ComposeProject {
                    name: "monitoring".into(), path: "/home/user/infra/monitoring".into(), status: ProjectStatus::Running, running_count: 3,
                    services: vec![
                        ComposeService { name: "prometheus".into(), image: "prom/prometheus".into(), status: "Up 5 days".into(), ports: vec!["9090:9090".into()] },
                        ComposeService { name: "grafana".into(), image: "grafana/grafana".into(), status: "Up 5 days".into(), ports: vec!["3001:3000".into()] },
                        ComposeService { name: "alertmanager".into(), image: "prom/alertmanager".into(), status: "Up 5 days".into(), ports: vec!["9093:9093".into()] },
                    ],
                },
                ComposeProject {
                    name: "dev-stack".into(), path: "/home/user/dev/stack".into(), status: ProjectStatus::Partial, running_count: 1,
                    services: vec![
                        ComposeService { name: "redis".into(), image: "redis:7".into(), status: "Up 1 hour".into(), ports: vec!["6379:6379".into()] },
                        ComposeService { name: "elasticsearch".into(), image: "elasticsearch:8".into(), status: "Exited".into(), ports: vec![] },
                    ],
                },
                ComposeProject {
                    name: "old-project".into(), path: "/home/user/archive/old".into(), status: ProjectStatus::Stopped, running_count: 0,
                    services: vec![
                        ComposeService { name: "app".into(), image: "old-project:v1".into(), status: "Exited".into(), ports: vec![] },
                    ],
                },
            ],
            selected: 0,
            expanded: None,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => if self.selected < self.projects.len().saturating_sub(1) { self.selected += 1; },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.expanded = if self.expanded == Some(self.selected) { None } else { Some(self.selected) };
            },
            KeyCode::Char('u') => {
                self.status_message = Some(format!("docker-compose up -d ({})", self.projects.get(self.selected).map(|p| p.name.as_str()).unwrap_or("")));
            },
            KeyCode::Char('d') => {
                self.status_message = Some(format!("docker-compose down ({})", self.projects.get(self.selected).map(|p| p.name.as_str()).unwrap_or("")));
            },
            KeyCode::Char('r') => {
                self.status_message = Some(format!("docker-compose restart ({})", self.projects.get(self.selected).map(|p| p.name.as_str()).unwrap_or("")));
            },
            KeyCode::Char('l') => self.status_message = Some("Would show logs...".into()),
            KeyCode::Char('p') => self.status_message = Some("docker-compose pull".into()),
            KeyCode::Char('b') => self.status_message = Some("docker-compose build".into()),
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| "j/k:nav enter:expand u:up d:down r:restart l:logs p:pull b:build q:quit".into())
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
