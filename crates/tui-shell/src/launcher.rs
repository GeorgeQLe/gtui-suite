//! Application launcher with registry and fuzzy search.

use crate::app::LaunchMode;
use crate::error::{ShellError, ShellResult};
use crate::workspace::WorkspaceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Application metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMeta {
    /// Unique app identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Category.
    pub category: AppCategory,
    /// Icon (Nerd Font character).
    pub icon: Option<String>,
    /// Keywords for search.
    pub keywords: Vec<String>,
    /// Launch mode preference.
    pub launch_mode: LaunchMode,
    /// Executable path (for subprocess).
    pub executable: Option<PathBuf>,
    /// Plugin ID (for in-process).
    pub plugin_id: Option<String>,
    /// Default arguments.
    pub default_args: Vec<String>,
    /// Whether this is a built-in app.
    pub builtin: bool,
}

impl AppMeta {
    /// Create new app metadata.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category: AppCategory::Other,
            icon: None,
            keywords: Vec::new(),
            launch_mode: LaunchMode::Subprocess,
            executable: None,
            plugin_id: None,
            default_args: Vec::new(),
            builtin: false,
        }
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set category.
    pub fn with_category(mut self, category: AppCategory) -> Self {
        self.category = category;
        self
    }

    /// Set icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Add keywords.
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Set executable path.
    pub fn with_executable(mut self, path: PathBuf) -> Self {
        self.executable = Some(path);
        self.launch_mode = LaunchMode::Subprocess;
        self
    }

    /// Set plugin ID.
    pub fn with_plugin(mut self, plugin_id: impl Into<String>) -> Self {
        self.plugin_id = Some(plugin_id.into());
        self.launch_mode = LaunchMode::InProcess;
        self
    }

    /// Mark as builtin.
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }

    /// Get searchable text.
    pub fn search_text(&self) -> String {
        let mut text = format!("{} {} {}", self.id, self.name, self.description);
        for kw in &self.keywords {
            text.push(' ');
            text.push_str(kw);
        }
        text.to_lowercase()
    }
}

/// App category for grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    /// Development tools.
    Development,
    /// System utilities.
    System,
    /// Productivity apps.
    Productivity,
    /// Communication apps.
    Communication,
    /// File management.
    Files,
    /// Network tools.
    Network,
    /// Data/database tools.
    Data,
    /// Monitoring.
    Monitoring,
    /// Security tools.
    Security,
    /// Other.
    #[default]
    Other,
}

impl AppCategory {
    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Development => "Development",
            Self::System => "System",
            Self::Productivity => "Productivity",
            Self::Communication => "Communication",
            Self::Files => "Files",
            Self::Network => "Network",
            Self::Data => "Data",
            Self::Monitoring => "Monitoring",
            Self::Security => "Security",
            Self::Other => "Other",
        }
    }

    /// Get icon.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Development => "",
            Self::System => "",
            Self::Productivity => "",
            Self::Communication => "",
            Self::Files => "",
            Self::Network => "󰖟",
            Self::Data => "",
            Self::Monitoring => "",
            Self::Security => "",
            Self::Other => "",
        }
    }
}

/// Launch request with parameters.
#[derive(Debug, Clone)]
pub struct LaunchRequest {
    /// App ID.
    pub app_id: String,
    /// Arguments.
    pub args: Vec<String>,
    /// Target workspace.
    pub workspace: Option<WorkspaceId>,
    /// Working directory.
    pub cwd: Option<PathBuf>,
    /// Environment variables.
    pub env: HashMap<String, String>,
}

impl LaunchRequest {
    /// Create new launch request.
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            args: Vec::new(),
            workspace: None,
            cwd: None,
            env: HashMap::new(),
        }
    }

    /// Set arguments.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set workspace.
    pub fn with_workspace(mut self, workspace: WorkspaceId) -> Self {
        self.workspace = Some(workspace);
        self
    }

    /// Set working directory.
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    /// Add environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

/// Recent app entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentApp {
    /// App ID.
    pub app_id: String,
    /// Launch count.
    pub count: u32,
    /// Last launched.
    pub last_used: chrono::DateTime<chrono::Utc>,
}

/// Application launcher.
pub struct AppLauncher {
    /// Registered apps.
    apps: HashMap<String, AppMeta>,
    /// Recent apps (sorted by recency).
    recents: Vec<RecentApp>,
    /// Maximum recent apps to track.
    max_recents: usize,
    /// Fuzzy matcher.
    matcher: nucleo_matcher::Matcher,
}

impl AppLauncher {
    /// Create new launcher.
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
            recents: Vec::new(),
            max_recents: 20,
            matcher: nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT),
        }
    }

    /// Register an app.
    pub fn register(&mut self, meta: AppMeta) {
        self.apps.insert(meta.id.clone(), meta);
    }

    /// Unregister an app.
    pub fn unregister(&mut self, id: &str) {
        self.apps.remove(id);
    }

    /// Get app metadata.
    pub fn get(&self, id: &str) -> Option<&AppMeta> {
        self.apps.get(id)
    }

    /// List all apps.
    pub fn list(&self) -> Vec<&AppMeta> {
        self.apps.values().collect()
    }

    /// List apps by category.
    pub fn by_category(&self, category: AppCategory) -> Vec<&AppMeta> {
        self.apps
            .values()
            .filter(|app| app.category == category)
            .collect()
    }

    /// Search apps with fuzzy matching.
    pub fn search(&mut self, query: &str) -> Vec<(&AppMeta, u32)> {
        if query.is_empty() {
            return self.list().into_iter().map(|app| (app, 0)).collect();
        }

        let pattern = nucleo_matcher::pattern::Pattern::parse(
            query,
            nucleo_matcher::pattern::CaseMatching::Ignore,
            nucleo_matcher::pattern::Normalization::Smart,
        );

        let mut results: Vec<_> = self
            .apps
            .values()
            .filter_map(|app| {
                let haystack = nucleo_matcher::Utf32Str::new(&app.search_text(), &mut vec![]);
                pattern
                    .score(haystack, &mut self.matcher)
                    .map(|score| (app, score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    /// Record app launch (for recents).
    pub fn record_launch(&mut self, app_id: &str) {
        let now = chrono::Utc::now();

        if let Some(recent) = self.recents.iter_mut().find(|r| r.app_id == app_id) {
            recent.count += 1;
            recent.last_used = now;
        } else {
            self.recents.push(RecentApp {
                app_id: app_id.to_string(),
                count: 1,
                last_used: now,
            });
        }

        // Sort by last used
        self.recents.sort_by(|a, b| b.last_used.cmp(&a.last_used));

        // Trim
        self.recents.truncate(self.max_recents);
    }

    /// Get recent apps.
    pub fn recents(&self) -> Vec<(&AppMeta, &RecentApp)> {
        self.recents
            .iter()
            .filter_map(|recent| {
                self.apps.get(&recent.app_id).map(|app| (app, recent))
            })
            .collect()
    }

    /// Get frequent apps (sorted by count).
    pub fn frequent(&self) -> Vec<(&AppMeta, u32)> {
        let mut sorted = self.recents.clone();
        sorted.sort_by(|a, b| b.count.cmp(&a.count));

        sorted
            .iter()
            .filter_map(|recent| {
                self.apps.get(&recent.app_id).map(|app| (app, recent.count))
            })
            .collect()
    }

    /// Validate launch request.
    pub fn validate(&self, request: &LaunchRequest) -> ShellResult<&AppMeta> {
        let meta = self
            .apps
            .get(&request.app_id)
            .ok_or_else(|| ShellError::App(format!("Unknown app: {}", request.app_id)))?;

        match meta.launch_mode {
            LaunchMode::Subprocess => {
                if meta.executable.is_none() {
                    return Err(ShellError::App(format!(
                        "App {} has no executable configured",
                        request.app_id
                    )));
                }
            }
            LaunchMode::InProcess => {
                if meta.plugin_id.is_none() {
                    return Err(ShellError::App(format!(
                        "App {} has no plugin configured",
                        request.app_id
                    )));
                }
            }
        }

        Ok(meta)
    }

    /// Register built-in apps.
    pub fn register_builtins(&mut self) {
        // Help viewer
        self.register(
            AppMeta::new("help", "Help")
                .with_description("View help and documentation")
                .with_category(AppCategory::System)
                .with_icon("")
                .builtin(),
        );

        // Settings
        self.register(
            AppMeta::new("settings", "Settings")
                .with_description("Configure shell settings")
                .with_category(AppCategory::System)
                .with_icon("")
                .builtin(),
        );

        // App browser
        self.register(
            AppMeta::new("app-browser", "App Browser")
                .with_description("Browse and install applications")
                .with_category(AppCategory::System)
                .with_icon("")
                .builtin(),
        );
    }

    /// Load recents from file.
    pub fn load_recents(&mut self, path: &std::path::Path) -> ShellResult<()> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            self.recents = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// Save recents to file.
    pub fn save_recents(&self, path: &std::path::Path) -> ShellResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.recents)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for AppLauncher {
    fn default() -> Self {
        let mut launcher = Self::new();
        launcher.register_builtins();
        launcher
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_meta() {
        let meta = AppMeta::new("test", "Test App")
            .with_description("A test app")
            .with_category(AppCategory::Development)
            .with_keywords(vec!["testing".to_string()]);

        assert_eq!(meta.id, "test");
        assert_eq!(meta.name, "Test App");
        assert!(meta.search_text().contains("testing"));
    }

    #[test]
    fn test_launcher_register() {
        let mut launcher = AppLauncher::new();
        launcher.register(AppMeta::new("app1", "App One"));
        launcher.register(AppMeta::new("app2", "App Two"));

        assert_eq!(launcher.list().len(), 2);
        assert!(launcher.get("app1").is_some());
    }

    #[test]
    fn test_launcher_search() {
        let mut launcher = AppLauncher::new();
        launcher.register(
            AppMeta::new("git", "Git Client")
                .with_keywords(vec!["vcs".to_string(), "version control".to_string()]),
        );
        launcher.register(AppMeta::new("docker", "Docker Manager"));

        let results = launcher.search("git");
        assert!(!results.is_empty());
        assert_eq!(results[0].0.id, "git");
    }

    #[test]
    fn test_recents() {
        let mut launcher = AppLauncher::new();
        launcher.register(AppMeta::new("app1", "App One"));
        launcher.register(AppMeta::new("app2", "App Two"));

        launcher.record_launch("app1");
        launcher.record_launch("app2");
        launcher.record_launch("app1");

        let recents = launcher.recents();
        assert_eq!(recents.len(), 2);

        let frequent = launcher.frequent();
        assert_eq!(frequent[0].0.id, "app1");
        assert_eq!(frequent[0].1, 2);
    }

    #[test]
    fn test_category() {
        assert_eq!(AppCategory::Development.name(), "Development");
        assert!(!AppCategory::Development.icon().is_empty());
    }
}
