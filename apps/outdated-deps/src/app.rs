use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    Patch,
    Minor,
    Major,
}

impl UpdateType {
    pub fn name(&self) -> &'static str {
        match self {
            UpdateType::Patch => "Patch",
            UpdateType::Minor => "Minor",
            UpdateType::Major => "Major",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub update_type: UpdateType,
    pub selected: bool,
    pub has_breaking: bool,
}

impl Dependency {
    pub fn version_diff(&self) -> String {
        format!("{} -> {}", self.current, self.latest)
    }
}

pub struct App {
    pub deps: Vec<Dependency>,
    pub selected: usize,
    pub filter_major: bool,
    pub show_breaking_only: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            deps: create_demo_deps(),
            selected: 0,
            filter_major: false,
            show_breaking_only: false,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let filtered = self.filtered_deps();
                if self.selected < filtered.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                let indices = self.filtered_indices();
                if let Some(&idx) = indices.get(self.selected) {
                    self.deps[idx].selected = !self.deps[idx].selected;
                }
            }
            KeyCode::Char('a') => {
                let indices = self.filtered_indices();
                for &idx in &indices {
                    self.deps[idx].selected = true;
                }
                self.status_message = Some("All selected".to_string());
            }
            KeyCode::Char('n') => {
                for dep in &mut self.deps {
                    dep.selected = false;
                }
                self.status_message = Some("None selected".to_string());
            }
            KeyCode::Char('m') => {
                self.filter_major = !self.filter_major;
                self.selected = 0;
                self.status_message = Some(if self.filter_major {
                    "Hiding major updates".to_string()
                } else {
                    "Showing all updates".to_string()
                });
            }
            KeyCode::Char('b') => {
                self.show_breaking_only = !self.show_breaking_only;
                self.selected = 0;
                self.status_message = Some(if self.show_breaking_only {
                    "Showing breaking changes only".to_string()
                } else {
                    "Showing all changes".to_string()
                });
            }
            KeyCode::Char('u') => {
                let count = self.deps.iter().filter(|d| d.selected).count();
                if count > 0 {
                    self.status_message = Some(format!("Would update {} dependencies...", count));
                } else {
                    self.status_message = Some("No dependencies selected".to_string());
                }
            }
            KeyCode::Char('c') => {
                let indices = self.filtered_indices();
                if let Some(&idx) = indices.get(self.selected) {
                    self.status_message = Some(format!("Would show changelog for {}", self.deps[idx].name));
                }
            }
            KeyCode::Char('r') => {
                self.status_message = Some("Checking for updates...".to_string());
            }
            _ => {}
        }
        false
    }

    fn filtered_indices(&self) -> Vec<usize> {
        self.deps
            .iter()
            .enumerate()
            .filter(|(_, d)| {
                let major_ok = !self.filter_major || d.update_type != UpdateType::Major;
                let breaking_ok = !self.show_breaking_only || d.has_breaking;
                major_ok && breaking_ok
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn filtered_deps(&self) -> Vec<&Dependency> {
        self.deps
            .iter()
            .filter(|d| {
                let major_ok = !self.filter_major || d.update_type != UpdateType::Major;
                let breaking_ok = !self.show_breaking_only || d.has_breaking;
                major_ok && breaking_ok
            })
            .collect()
    }

    pub fn selected_count(&self) -> usize {
        self.deps.iter().filter(|d| d.selected).count()
    }

    pub fn major_count(&self) -> usize {
        self.deps.iter().filter(|d| d.update_type == UpdateType::Major).count()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav space:select a:all n:none m:major b:breaking u:update c:changelog r:refresh q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "tokio".to_string(),
            current: "1.35.0".to_string(),
            latest: "1.36.0".to_string(),
            update_type: UpdateType::Minor,
            selected: false,
            has_breaking: false,
        },
        Dependency {
            name: "serde".to_string(),
            current: "1.0.190".to_string(),
            latest: "1.0.195".to_string(),
            update_type: UpdateType::Patch,
            selected: false,
            has_breaking: false,
        },
        Dependency {
            name: "reqwest".to_string(),
            current: "0.11.22".to_string(),
            latest: "0.12.0".to_string(),
            update_type: UpdateType::Major,
            selected: false,
            has_breaking: true,
        },
        Dependency {
            name: "anyhow".to_string(),
            current: "1.0.75".to_string(),
            latest: "1.0.80".to_string(),
            update_type: UpdateType::Patch,
            selected: false,
            has_breaking: false,
        },
        Dependency {
            name: "clap".to_string(),
            current: "4.4.0".to_string(),
            latest: "4.5.0".to_string(),
            update_type: UpdateType::Minor,
            selected: false,
            has_breaking: false,
        },
        Dependency {
            name: "tracing".to_string(),
            current: "0.1.37".to_string(),
            latest: "0.2.0".to_string(),
            update_type: UpdateType::Major,
            selected: false,
            has_breaking: true,
        },
        Dependency {
            name: "chrono".to_string(),
            current: "0.4.31".to_string(),
            latest: "0.4.35".to_string(),
            update_type: UpdateType::Patch,
            selected: false,
            has_breaking: false,
        },
        Dependency {
            name: "regex".to_string(),
            current: "1.10.0".to_string(),
            latest: "1.10.3".to_string(),
            update_type: UpdateType::Patch,
            selected: false,
            has_breaking: false,
        },
    ]
}
