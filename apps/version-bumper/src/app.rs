use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
    PreRelease,
}

impl BumpType {
    pub fn name(&self) -> &'static str {
        match self {
            BumpType::Major => "Major",
            BumpType::Minor => "Minor",
            BumpType::Patch => "Patch",
            BumpType::PreRelease => "Pre-release",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch, pre: None }
    }

    pub fn to_string(&self) -> String {
        match &self.pre {
            Some(pre) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }

    pub fn bump(&self, bump_type: BumpType) -> Self {
        match bump_type {
            BumpType::Major => Self::new(self.major + 1, 0, 0),
            BumpType::Minor => Self::new(self.major, self.minor + 1, 0),
            BumpType::Patch => Self::new(self.major, self.minor, self.patch + 1),
            BumpType::PreRelease => {
                let mut v = self.clone();
                v.pre = Some("alpha.1".to_string());
                v
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangeEntry {
    pub change_type: String,
    pub description: String,
}

pub struct App {
    pub current_version: Version,
    pub bump_type: BumpType,
    pub preview_version: Version,
    pub changes: Vec<ChangeEntry>,
    pub selected_change: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let current = Version::new(1, 2, 3);
        let preview = current.bump(BumpType::Patch);

        Self {
            current_version: current,
            bump_type: BumpType::Patch,
            preview_version: preview,
            changes: create_demo_changes(),
            selected_change: 0,
            status_message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('1') => {
                self.bump_type = BumpType::Major;
                self.update_preview();
            }
            KeyCode::Char('2') => {
                self.bump_type = BumpType::Minor;
                self.update_preview();
            }
            KeyCode::Char('3') => {
                self.bump_type = BumpType::Patch;
                self.update_preview();
            }
            KeyCode::Char('4') => {
                self.bump_type = BumpType::PreRelease;
                self.update_preview();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_change < self.changes.len().saturating_sub(1) {
                    self.selected_change += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_change = self.selected_change.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.apply_bump();
            }
            KeyCode::Char('a') => {
                self.status_message = Some("Would add changelog entry...".to_string());
            }
            KeyCode::Char('g') => {
                self.status_message = Some("Changelog generated!".to_string());
            }
            _ => {}
        }
        false
    }

    fn update_preview(&mut self) {
        self.preview_version = self.current_version.bump(self.bump_type);
        self.status_message = Some(format!(
            "{} bump: {} -> {}",
            self.bump_type.name(),
            self.current_version.to_string(),
            self.preview_version.to_string()
        ));
    }

    fn apply_bump(&mut self) {
        self.current_version = self.preview_version.clone();
        self.update_preview();
        self.status_message = Some(format!(
            "Version bumped to {}",
            self.current_version.to_string()
        ));
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "1:major 2:minor 3:patch 4:pre Enter:apply a:add-entry g:generate q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_changes() -> Vec<ChangeEntry> {
    vec![
        ChangeEntry {
            change_type: "feat".to_string(),
            description: "Add new authentication system".to_string(),
        },
        ChangeEntry {
            change_type: "fix".to_string(),
            description: "Fix memory leak in worker pool".to_string(),
        },
        ChangeEntry {
            change_type: "docs".to_string(),
            description: "Update API documentation".to_string(),
        },
        ChangeEntry {
            change_type: "refactor".to_string(),
            description: "Simplify error handling".to_string(),
        },
        ChangeEntry {
            change_type: "perf".to_string(),
            description: "Optimize database queries".to_string(),
        },
    ]
}
