use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitType {
    Feature,
    Fix,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Chore,
    Breaking,
}

impl CommitType {
    pub fn name(&self) -> &'static str {
        match self {
            CommitType::Feature => "feat",
            CommitType::Fix => "fix",
            CommitType::Docs => "docs",
            CommitType::Style => "style",
            CommitType::Refactor => "refactor",
            CommitType::Perf => "perf",
            CommitType::Test => "test",
            CommitType::Chore => "chore",
            CommitType::Breaking => "BREAKING",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            CommitType::Feature => "✨",
            CommitType::Fix => "🐛",
            CommitType::Docs => "📚",
            CommitType::Style => "💎",
            CommitType::Refactor => "♻️",
            CommitType::Perf => "⚡",
            CommitType::Test => "🧪",
            CommitType::Chore => "🔧",
            CommitType::Breaking => "💥",
        }
    }

    pub fn section_title(&self) -> &'static str {
        match self {
            CommitType::Feature => "Features",
            CommitType::Fix => "Bug Fixes",
            CommitType::Docs => "Documentation",
            CommitType::Style => "Styles",
            CommitType::Refactor => "Code Refactoring",
            CommitType::Perf => "Performance",
            CommitType::Test => "Tests",
            CommitType::Chore => "Chores",
            CommitType::Breaking => "BREAKING CHANGES",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub message: String,
    pub author: String,
    pub date: String,
    pub included: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    KeepAChangelog,
    Conventional,
}

impl OutputFormat {
    pub fn name(&self) -> &'static str {
        match self {
            OutputFormat::Markdown => "Markdown",
            OutputFormat::KeepAChangelog => "Keep a Changelog",
            OutputFormat::Conventional => "Conventional",
        }
    }
}

pub struct App {
    pub commits: Vec<CommitEntry>,
    pub selected: usize,
    pub version: String,
    pub output_format: OutputFormat,
    pub preview_mode: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            commits: create_demo_commits(),
            selected: 0,
            version: "1.0.0".to_string(),
            output_format: OutputFormat::Markdown,
            preview_mode: false,
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
                if self.selected < self.commits.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                if let Some(commit) = self.commits.get_mut(self.selected) {
                    commit.included = !commit.included;
                }
            }
            KeyCode::Char('a') => {
                for commit in &mut self.commits {
                    commit.included = true;
                }
                self.status_message = Some("All commits included".to_string());
            }
            KeyCode::Char('n') => {
                for commit in &mut self.commits {
                    commit.included = false;
                }
                self.status_message = Some("All commits excluded".to_string());
            }
            KeyCode::Char('1') => {
                self.output_format = OutputFormat::Markdown;
                self.status_message = Some("Format: Markdown".to_string());
            }
            KeyCode::Char('2') => {
                self.output_format = OutputFormat::KeepAChangelog;
                self.status_message = Some("Format: Keep a Changelog".to_string());
            }
            KeyCode::Char('3') => {
                self.output_format = OutputFormat::Conventional;
                self.status_message = Some("Format: Conventional".to_string());
            }
            KeyCode::Char('p') => {
                self.preview_mode = !self.preview_mode;
                self.status_message = Some(if self.preview_mode {
                    "Preview mode enabled".to_string()
                } else {
                    "Preview mode disabled".to_string()
                });
            }
            KeyCode::Char('g') => {
                let included = self.commits.iter().filter(|c| c.included).count();
                self.status_message = Some(format!("Generated changelog with {} entries", included));
            }
            KeyCode::Char('s') => {
                self.status_message = Some("Changelog saved to CHANGELOG.md".to_string());
            }
            _ => {}
        }
        false
    }

    pub fn included_count(&self) -> usize {
        self.commits.iter().filter(|c| c.included).count()
    }

    pub fn breaking_count(&self) -> usize {
        self.commits.iter()
            .filter(|c| c.included && c.commit_type == CommitType::Breaking)
            .count()
    }

    pub fn generate_preview(&self) -> Vec<String> {
        let mut lines = vec![
            format!("# Changelog"),
            String::new(),
            format!("## [{}] - 2024-01-15", self.version),
            String::new(),
        ];

        let types = [
            CommitType::Breaking,
            CommitType::Feature,
            CommitType::Fix,
            CommitType::Docs,
            CommitType::Refactor,
            CommitType::Perf,
        ];

        for commit_type in types {
            let commits: Vec<_> = self.commits.iter()
                .filter(|c| c.included && c.commit_type == commit_type)
                .collect();

            if !commits.is_empty() {
                lines.push(format!("### {}", commit_type.section_title()));
                lines.push(String::new());

                for commit in commits {
                    let scope = commit.scope.as_ref()
                        .map(|s| format!("**{}**: ", s))
                        .unwrap_or_default();
                    lines.push(format!("- {}{} ({})", scope, commit.message, &commit.hash[..7]));
                }
                lines.push(String::new());
            }
        }

        lines
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "j/k:nav space:toggle a:all n:none 1-3:format p:preview g:generate s:save q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_commits() -> Vec<CommitEntry> {
    vec![
        CommitEntry {
            hash: "abc123def456".to_string(),
            commit_type: CommitType::Feature,
            scope: Some("auth".to_string()),
            message: "Add OAuth2 authentication support".to_string(),
            author: "Alice".to_string(),
            date: "2024-01-15".to_string(),
            included: true,
        },
        CommitEntry {
            hash: "def456abc789".to_string(),
            commit_type: CommitType::Fix,
            scope: Some("api".to_string()),
            message: "Fix rate limiting edge case".to_string(),
            author: "Bob".to_string(),
            date: "2024-01-14".to_string(),
            included: true,
        },
        CommitEntry {
            hash: "789abc123def".to_string(),
            commit_type: CommitType::Breaking,
            scope: None,
            message: "Remove deprecated v1 API endpoints".to_string(),
            author: "Charlie".to_string(),
            date: "2024-01-13".to_string(),
            included: true,
        },
        CommitEntry {
            hash: "456def789abc".to_string(),
            commit_type: CommitType::Docs,
            scope: Some("readme".to_string()),
            message: "Update installation instructions".to_string(),
            author: "Alice".to_string(),
            date: "2024-01-12".to_string(),
            included: true,
        },
        CommitEntry {
            hash: "123abc456def".to_string(),
            commit_type: CommitType::Refactor,
            scope: Some("core".to_string()),
            message: "Simplify error handling logic".to_string(),
            author: "Bob".to_string(),
            date: "2024-01-11".to_string(),
            included: false,
        },
        CommitEntry {
            hash: "abc789def123".to_string(),
            commit_type: CommitType::Perf,
            scope: Some("db".to_string()),
            message: "Optimize database queries".to_string(),
            author: "Charlie".to_string(),
            date: "2024-01-10".to_string(),
            included: true,
        },
        CommitEntry {
            hash: "def123abc456".to_string(),
            commit_type: CommitType::Feature,
            scope: Some("ui".to_string()),
            message: "Add dark mode toggle".to_string(),
            author: "Alice".to_string(),
            date: "2024-01-09".to_string(),
            included: true,
        },
        CommitEntry {
            hash: "789def456abc".to_string(),
            commit_type: CommitType::Fix,
            scope: None,
            message: "Fix memory leak in worker pool".to_string(),
            author: "Bob".to_string(),
            date: "2024-01-08".to_string(),
            included: true,
        },
    ]
}
