use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct RegistryTag {
    pub name: String,
    pub digest: String,
    pub size: u64,
    pub pushed: String,
}

#[derive(Debug, Clone)]
pub struct RegistryRepo {
    pub name: String,
    pub tag_count: usize,
    pub last_updated: String,
}

#[derive(Debug, Clone)]
pub struct Registry {
    pub name: String,
    pub url: String,
    pub authenticated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode { Registries, Repos, Tags }

pub struct App {
    pub registries: Vec<Registry>,
    pub repos: Vec<RegistryRepo>,
    pub tags: Vec<RegistryTag>,
    pub view_mode: ViewMode,
    pub selected: usize,
    pub current_registry: usize,
    pub current_repo: Option<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            registries: vec![
                Registry { name: "Docker Hub".into(), url: "https://registry-1.docker.io".into(), authenticated: true },
                Registry { name: "GitHub Packages".into(), url: "https://ghcr.io".into(), authenticated: true },
                Registry { name: "Private Registry".into(), url: "https://registry.company.com".into(), authenticated: true },
            ],
            repos: vec![
                RegistryRepo { name: "myorg/webapp".into(), tag_count: 15, last_updated: "2024-01-15".into() },
                RegistryRepo { name: "myorg/api".into(), tag_count: 23, last_updated: "2024-01-14".into() },
                RegistryRepo { name: "myorg/worker".into(), tag_count: 8, last_updated: "2024-01-10".into() },
                RegistryRepo { name: "myorg/nginx-custom".into(), tag_count: 5, last_updated: "2024-01-05".into() },
            ],
            tags: vec![
                RegistryTag { name: "latest".into(), digest: "sha256:abc123...".into(), size: 150_000_000, pushed: "2024-01-15".into() },
                RegistryTag { name: "v1.2.3".into(), digest: "sha256:abc123...".into(), size: 150_000_000, pushed: "2024-01-15".into() },
                RegistryTag { name: "v1.2.2".into(), digest: "sha256:def456...".into(), size: 148_000_000, pushed: "2024-01-10".into() },
                RegistryTag { name: "v1.2.1".into(), digest: "sha256:ghi789...".into(), size: 147_000_000, pushed: "2024-01-05".into() },
                RegistryTag { name: "develop".into(), digest: "sha256:jkl012...".into(), size: 152_000_000, pushed: "2024-01-14".into() },
            ],
            view_mode: ViewMode::Registries,
            selected: 0,
            current_registry: 0,
            current_repo: None,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                match self.view_mode {
                    ViewMode::Registries => return true,
                    ViewMode::Repos => { self.view_mode = ViewMode::Registries; self.selected = self.current_registry; },
                    ViewMode::Tags => { self.view_mode = ViewMode::Repos; self.selected = 0; },
                }
            },
            KeyCode::Char('j') | KeyCode::Down => {
                let max = match self.view_mode {
                    ViewMode::Registries => self.registries.len(),
                    ViewMode::Repos => self.repos.len(),
                    ViewMode::Tags => self.tags.len(),
                };
                if self.selected < max.saturating_sub(1) { self.selected += 1; }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter => {
                match self.view_mode {
                    ViewMode::Registries => {
                        self.current_registry = self.selected;
                        self.view_mode = ViewMode::Repos;
                        self.selected = 0;
                    },
                    ViewMode::Repos => {
                        if let Some(repo) = self.repos.get(self.selected) {
                            self.current_repo = Some(repo.name.clone());
                        }
                        self.view_mode = ViewMode::Tags;
                        self.selected = 0;
                    },
                    ViewMode::Tags => {
                        self.status_message = Some("Would pull this tag...".into());
                    },
                }
            },
            KeyCode::Char('p') => self.status_message = Some("Would pull...".into()),
            KeyCode::Char('d') => self.status_message = Some("Would delete...".into()),
            KeyCode::Char('r') => self.status_message = Some("Refreshing...".into()),
            KeyCode::Char('a') => self.status_message = Some("Would add registry...".into()),
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        self.status_message.clone().unwrap_or_else(|| {
            match self.view_mode {
                ViewMode::Registries => "j/k:nav enter:browse a:add-registry q:quit".into(),
                ViewMode::Repos => "j/k:nav enter:tags esc:back r:refresh q:quit".into(),
                ViewMode::Tags => "j/k:nav enter:pull p:pull d:delete esc:back q:quit".into(),
            }
        })
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
