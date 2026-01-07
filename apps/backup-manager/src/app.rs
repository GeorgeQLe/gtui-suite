use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::config::Config;
use crate::database::Database;
use crate::profile::{BackendType, BackupProfile, BackupRun};

/// Current view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    ProfileDetail,
    Runs,
    Help,
}

impl View {
    pub fn label(&self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::ProfileDetail => "Details",
            View::Runs => "Runs",
            View::Help => "Help",
        }
    }
}

/// Application mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AddProfile(ProfileFormState),
    EditProfile(ProfileFormState),
    Confirm(ConfirmAction),
}

/// Profile form state
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProfileFormState {
    pub name: String,
    pub backend: usize,
    pub sources: String,
    pub destination: String,
    pub excludes: String,
    pub schedule: String,
    pub field: usize,
}

impl ProfileFormState {
    pub fn from_profile(profile: &BackupProfile) -> Self {
        Self {
            name: profile.name.clone(),
            backend: match profile.backend {
                BackendType::Rsync => 0,
                BackendType::Restic => 1,
                BackendType::Borg => 2,
            },
            sources: profile.sources_display(),
            destination: profile.destination.clone(),
            excludes: profile.excludes.join(", "),
            schedule: profile.schedule.clone().unwrap_or_default(),
            field: 0,
        }
    }

    pub fn field_count() -> usize { 6 }

    pub fn field_label(idx: usize) -> &'static str {
        match idx {
            0 => "Name",
            1 => "Backend (0:rsync, 1:restic, 2:borg)",
            2 => "Source Paths (comma-separated)",
            3 => "Destination",
            4 => "Excludes (comma-separated)",
            5 => "Schedule (cron)",
            _ => "",
        }
    }

    pub fn field_value(&self, idx: usize) -> String {
        match idx {
            0 => self.name.clone(),
            1 => self.backend.to_string(),
            2 => self.sources.clone(),
            3 => self.destination.clone(),
            4 => self.excludes.clone(),
            5 => self.schedule.clone(),
            _ => String::new(),
        }
    }

    pub fn set_field_value(&mut self, idx: usize, value: String) {
        match idx {
            0 => self.name = value,
            1 => self.backend = value.parse().unwrap_or(0),
            2 => self.sources = value,
            3 => self.destination = value,
            4 => self.excludes = value,
            5 => self.schedule = value,
            _ => {}
        }
    }
}

/// Action requiring confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteProfile(String),
    RunBackup(String),
}

/// Application state
pub struct App {
    pub config: Config,
    pub db: Database,
    pub view: View,
    pub mode: Mode,
    pub profiles: Vec<BackupProfile>,
    pub selected: usize,
    pub selected_runs: Vec<BackupRun>,
    pub run_selected: usize,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let db = Database::open()?;

        let mut app = Self {
            config,
            db,
            view: View::Dashboard,
            mode: Mode::Normal,
            profiles: Vec::new(),
            selected: 0,
            selected_runs: Vec::new(),
            run_selected: 0,
            message: None,
            error: None,
        };

        app.refresh()?;
        Ok(app)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.profiles = self.db.list_profiles()?;

        if self.selected >= self.profiles.len() {
            self.selected = self.profiles.len().saturating_sub(1);
        }

        // Load runs for selected profile
        if let Some(profile) = self.selected_profile() {
            self.selected_runs = self.db.get_recent_runs(&profile.id, 10)?;
        }

        Ok(())
    }

    pub fn selected_profile(&self) -> Option<&BackupProfile> {
        self.profiles.get(self.selected)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::AddProfile(_) | Mode::EditProfile(_) => self.handle_form_key(key),
            Mode::Confirm(_) => self.handle_confirm_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Esc => {
                if self.view != View::Dashboard {
                    self.view = View::Dashboard;
                }
            }

            // Navigation
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Home | KeyCode::Char('g') => self.selected = 0,
            KeyCode::End | KeyCode::Char('G') => {
                self.selected = self.profiles.len().saturating_sub(1);
            }

            // View details
            KeyCode::Enter => {
                if self.view == View::Dashboard && self.selected_profile().is_some() {
                    self.view = View::ProfileDetail;
                    // Load runs for this profile
                    if let Some(profile) = self.selected_profile() {
                        let id = profile.id.clone();
                        if let Ok(runs) = self.db.get_recent_runs(&id, 20) {
                            self.selected_runs = runs;
                            self.run_selected = 0;
                        }
                    }
                }
            }

            // Profile actions
            KeyCode::Char('a') if self.view == View::Dashboard => {
                self.mode = Mode::AddProfile(ProfileFormState::default());
            }
            KeyCode::Char('e') if self.view == View::Dashboard => {
                if let Some(profile) = self.selected_profile() {
                    self.mode = Mode::EditProfile(ProfileFormState::from_profile(profile));
                }
            }
            KeyCode::Char('d') if self.view == View::Dashboard => {
                if let Some(profile) = self.selected_profile() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteProfile(profile.id.clone()));
                }
            }
            KeyCode::Char('t') if self.view == View::Dashboard => {
                self.toggle_selected();
            }

            // Backup actions
            KeyCode::Char('b') => {
                if let Some(profile) = self.selected_profile() {
                    self.mode = Mode::Confirm(ConfirmAction::RunBackup(profile.id.clone()));
                }
            }

            // View runs
            KeyCode::Char('l') => {
                if self.selected_profile().is_some() {
                    self.view = View::Runs;
                }
            }

            // Refresh
            KeyCode::F(5) => {
                if let Err(e) = self.refresh() {
                    self.error = Some(format!("Refresh failed: {}", e));
                } else {
                    self.message = Some("Refreshed".to_string());
                }
            }

            _ => {}
        }

        false
    }

    fn handle_form_key(&mut self, key: KeyEvent) -> bool {
        let is_add = matches!(self.mode, Mode::AddProfile(_));

        // Check if we need to submit
        let should_submit = match &self.mode {
            Mode::AddProfile(form) | Mode::EditProfile(form) => {
                key.code == KeyCode::Enter && form.field == ProfileFormState::field_count() - 1
            }
            _ => false,
        };

        if should_submit {
            let form = match &self.mode {
                Mode::AddProfile(f) | Mode::EditProfile(f) => f.clone(),
                _ => return false,
            };
            if let Err(e) = self.submit_form(&form, is_add) {
                self.error = Some(format!("Save failed: {}", e));
            } else {
                self.message = Some(if is_add { "Profile added" } else { "Profile updated" }.to_string());
                let _ = self.refresh();
            }
            self.mode = Mode::Normal;
            return false;
        }

        match &mut self.mode {
            Mode::AddProfile(ref mut form) | Mode::EditProfile(ref mut form) => {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Enter | KeyCode::Tab | KeyCode::Down => {
                        form.field = (form.field + 1) % ProfileFormState::field_count();
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        form.field = form.field.checked_sub(1).unwrap_or(ProfileFormState::field_count() - 1);
                    }
                    KeyCode::Backspace => {
                        let mut val = form.field_value(form.field);
                        val.pop();
                        form.set_field_value(form.field, val);
                    }
                    KeyCode::Char(c) => {
                        let mut val = form.field_value(form.field);
                        val.push(c);
                        form.set_field_value(form.field, val);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }

    fn submit_form(&mut self, form: &ProfileFormState, is_add: bool) -> Result<()> {
        let backend = match form.backend {
            1 => BackendType::Restic,
            2 => BackendType::Borg,
            _ => BackendType::Rsync,
        };

        let mut profile = if is_add {
            BackupProfile::new(form.name.clone(), backend, form.destination.clone())
        } else {
            self.selected_profile().cloned().ok_or_else(|| anyhow::anyhow!("No profile selected"))?
        };

        profile.name = form.name.clone();
        profile.backend = backend;
        profile.source_paths = form.sources.split(',')
            .map(|s| s.trim().into())
            .filter(|p: &std::path::PathBuf| !p.as_os_str().is_empty())
            .collect();
        profile.destination = form.destination.clone();
        profile.excludes = form.excludes.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        profile.schedule = if form.schedule.is_empty() { None } else { Some(form.schedule.clone()) };

        if is_add {
            self.db.insert_profile(&profile)?;
        } else {
            self.db.update_profile(&profile)?;
        }

        Ok(())
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Mode::Confirm(action) = &self.mode {
                    match action {
                        ConfirmAction::DeleteProfile(id) => {
                            let id = id.clone();
                            if let Err(e) = self.db.delete_profile(&id) {
                                self.error = Some(format!("Delete failed: {}", e));
                            } else {
                                self.message = Some("Profile deleted".to_string());
                                let _ = self.refresh();
                            }
                        }
                        ConfirmAction::RunBackup(id) => {
                            self.message = Some(format!("Starting backup for profile {}...", &id[..8.min(id.len())]));
                            // In a real implementation, we would spawn the backup process here
                        }
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        false
    }

    fn move_down(&mut self) {
        match self.view {
            View::Dashboard | View::ProfileDetail => {
                if self.selected < self.profiles.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            View::Runs => {
                if self.run_selected < self.selected_runs.len().saturating_sub(1) {
                    self.run_selected += 1;
                }
            }
            View::Help => {}
        }
    }

    fn move_up(&mut self) {
        match self.view {
            View::Dashboard | View::ProfileDetail => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            View::Runs => {
                if self.run_selected > 0 {
                    self.run_selected -= 1;
                }
            }
            View::Help => {}
        }
    }

    fn toggle_selected(&mut self) {
        if let Some(profile) = self.selected_profile() {
            let id = profile.id.clone();
            let new_enabled = !profile.enabled;
            if let Err(e) = self.db.toggle_profile(&id, new_enabled) {
                self.error = Some(format!("Toggle failed: {}", e));
            } else {
                let _ = self.refresh();
                self.message = Some(if new_enabled { "Profile enabled" } else { "Profile disabled" }.to_string());
            }
        }
    }

    pub fn get_last_run(&self, profile_id: &str) -> Option<BackupRun> {
        self.db.get_last_run(profile_id).ok().flatten()
    }
}
