#![allow(dead_code)]

use crossterm::event::{KeyCode, KeyEvent};

use crate::config::Config;
use crate::git_ops::{BranchInfo, CommitInfo, FileStatus, GitRepo, StashEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Status,
    Log,
    Branches,
    Stash,
    Diff,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSection {
    Staged,
    Unstaged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Commit(String),
    CreateBranch(String),
    StashMessage(String),
}

pub struct App {
    pub repo: GitRepo,
    pub config: Config,
    pub view: View,
    pub mode: Mode,

    // Status view
    pub staged_files: Vec<FileStatus>,
    pub unstaged_files: Vec<FileStatus>,
    pub section: StatusSection,
    pub file_index: usize,

    // Log view
    pub commits: Vec<CommitInfo>,
    pub commit_index: usize,

    // Branch view
    pub branches: Vec<BranchInfo>,
    pub branch_index: usize,

    // Stash view
    pub stashes: Vec<StashEntry>,
    pub stash_index: usize,

    // Diff view
    pub diff_content: String,
    pub diff_scroll: usize,

    pub message: Option<String>,
    pub error: Option<String>,
}

impl App {
    pub fn new(repo: GitRepo, config: Config) -> Self {
        Self {
            repo,
            config,
            view: View::Status,
            mode: Mode::Normal,
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            section: StatusSection::Unstaged,
            file_index: 0,
            commits: Vec::new(),
            commit_index: 0,
            branches: Vec::new(),
            branch_index: 0,
            stashes: Vec::new(),
            stash_index: 0,
            diff_content: String::new(),
            diff_scroll: 0,
            message: None,
            error: None,
        }
    }

    pub fn refresh(&mut self) {
        match self.view {
            View::Status => self.refresh_status(),
            View::Log => self.refresh_log(),
            View::Branches => self.refresh_branches(),
            View::Stash => self.refresh_stash(),
            _ => {}
        }
    }

    fn refresh_status(&mut self) {
        match self.repo.staged_files() {
            Ok(files) => self.staged_files = files,
            Err(e) => self.error = Some(format!("Failed to get staged files: {}", e)),
        }
        match self.repo.unstaged_files() {
            Ok(files) => self.unstaged_files = files,
            Err(e) => self.error = Some(format!("Failed to get unstaged files: {}", e)),
        }
    }

    fn refresh_log(&mut self) {
        match self.repo.log(100) {
            Ok(commits) => self.commits = commits,
            Err(e) => self.error = Some(format!("Failed to get log: {}", e)),
        }
    }

    fn refresh_branches(&mut self) {
        match self.repo.branches() {
            Ok(branches) => self.branches = branches,
            Err(e) => self.error = Some(format!("Failed to get branches: {}", e)),
        }
    }

    fn refresh_stash(&mut self) {
        match self.repo.stash_list() {
            Ok(stashes) => self.stashes = stashes,
            Err(e) => self.error = Some(format!("Failed to get stashes: {}", e)),
        }
    }

    pub fn current_files(&self) -> &[FileStatus] {
        match self.section {
            StatusSection::Staged => &self.staged_files,
            StatusSection::Unstaged => &self.unstaged_files,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None;
        self.error = None;

        match &self.mode {
            Mode::Normal => match self.view {
                View::Status => self.handle_status_key(key),
                View::Log => self.handle_log_key(key),
                View::Branches => self.handle_branches_key(key),
                View::Stash => self.handle_stash_key(key),
                View::Diff => self.handle_diff_key(key),
                View::Help => self.handle_help_key(key),
            },
            Mode::Commit(_) | Mode::CreateBranch(_) | Mode::StashMessage(_) => {
                self.handle_input_key(key)
            }
        }
    }

    fn handle_status_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Tab => {
                self.section = match self.section {
                    StatusSection::Staged => StatusSection::Unstaged,
                    StatusSection::Unstaged => StatusSection::Staged,
                };
                self.file_index = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.current_files().len().saturating_sub(1);
                if self.file_index < max {
                    self.file_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.file_index > 0 {
                    self.file_index -= 1;
                }
            }
            KeyCode::Enter | KeyCode::Char('d') => {
                // Show diff
                if let Some(file) = self.current_files().get(self.file_index) {
                    let staged = self.section == StatusSection::Staged;
                    match self.repo.diff_file(&file.path, staged) {
                        Ok(diff) => {
                            self.diff_content = diff;
                            self.diff_scroll = 0;
                            self.view = View::Diff;
                        }
                        Err(e) => self.error = Some(format!("Failed to get diff: {}", e)),
                    }
                }
            }
            KeyCode::Char('s') => {
                // Stage file
                if let Some(file) = self.current_files().get(self.file_index).cloned() {
                    if !file.staged {
                        if let Err(e) = self.repo.stage_file(&file.path) {
                            self.error = Some(format!("Failed to stage: {}", e));
                        } else {
                            self.message = Some(format!("Staged {}", file.path));
                            self.refresh_status();
                        }
                    }
                }
            }
            KeyCode::Char('u') => {
                // Unstage file
                if let Some(file) = self.current_files().get(self.file_index).cloned() {
                    if file.staged {
                        if let Err(e) = self.repo.unstage_file(&file.path) {
                            self.error = Some(format!("Failed to unstage: {}", e));
                        } else {
                            self.message = Some(format!("Unstaged {}", file.path));
                            self.refresh_status();
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                // Stage all
                if let Err(e) = self.repo.stage_all() {
                    self.error = Some(format!("Failed to stage all: {}", e));
                } else {
                    self.message = Some("Staged all files".to_string());
                    self.refresh_status();
                }
            }
            KeyCode::Char('c') => {
                // Commit
                if !self.staged_files.is_empty() {
                    self.mode = Mode::Commit(String::new());
                } else {
                    self.error = Some("No staged changes to commit".to_string());
                }
            }
            KeyCode::Char('l') => {
                self.refresh_log();
                self.view = View::Log;
            }
            KeyCode::Char('b') => {
                self.refresh_branches();
                self.view = View::Branches;
            }
            KeyCode::Char('S') => {
                self.mode = Mode::StashMessage(String::new());
            }
            KeyCode::Char('t') => {
                self.refresh_stash();
                self.view = View::Stash;
            }
            KeyCode::Char('f') => {
                if let Err(e) = self.repo.fetch("origin") {
                    self.error = Some(format!("Fetch failed: {}", e));
                } else {
                    self.message = Some("Fetch complete".to_string());
                }
            }
            KeyCode::Char('P') => {
                if let Err(e) = self.repo.pull() {
                    self.error = Some(format!("Pull failed: {}", e));
                } else {
                    self.message = Some("Pull complete".to_string());
                    self.refresh_status();
                }
            }
            KeyCode::Char('p') => {
                if let Err(e) = self.repo.push() {
                    self.error = Some(format!("Push failed: {}", e));
                } else {
                    self.message = Some("Push complete".to_string());
                }
            }
            _ => {}
        }
        false
    }

    fn handle_log_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Status;
                self.refresh_status();
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.commit_index < self.commits.len().saturating_sub(1) {
                    self.commit_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.commit_index > 0 {
                    self.commit_index -= 1;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_branches_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Status;
                self.refresh_status();
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.branch_index < self.branches.len().saturating_sub(1) {
                    self.branch_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.branch_index > 0 {
                    self.branch_index -= 1;
                }
            }
            KeyCode::Enter => {
                // Checkout branch
                if let Some(branch) = self.branches.get(self.branch_index) {
                    if !branch.is_current && !branch.is_remote {
                        let name = branch.name.clone();
                        if let Err(e) = self.repo.checkout_branch(&name) {
                            self.error = Some(format!("Checkout failed: {}", e));
                        } else {
                            self.message = Some(format!("Switched to {}", name));
                            self.refresh_branches();
                        }
                    }
                }
            }
            KeyCode::Char('n') => {
                self.mode = Mode::CreateBranch(String::new());
            }
            KeyCode::Char('d') => {
                if let Some(branch) = self.branches.get(self.branch_index) {
                    if !branch.is_current && !branch.is_remote {
                        let name = branch.name.clone();
                        if let Err(e) = self.repo.delete_branch(&name) {
                            self.error = Some(format!("Delete failed: {}", e));
                        } else {
                            self.message = Some(format!("Deleted {}", name));
                            self.refresh_branches();
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn handle_stash_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Status;
                self.refresh_status();
            }
            KeyCode::Char('?') => self.view = View::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.stash_index < self.stashes.len().saturating_sub(1) {
                    self.stash_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.stash_index > 0 {
                    self.stash_index -= 1;
                }
            }
            KeyCode::Char('a') | KeyCode::Enter => {
                // Apply stash
                if let Some(stash) = self.stashes.get(self.stash_index) {
                    let idx = stash.index;
                    if let Err(e) = self.repo.stash_pop(idx) {
                        self.error = Some(format!("Stash pop failed: {}", e));
                    } else {
                        self.message = Some("Stash applied and dropped".to_string());
                        self.refresh_stash();
                    }
                }
            }
            KeyCode::Char('d') => {
                // Drop stash
                if let Some(stash) = self.stashes.get(self.stash_index) {
                    let idx = stash.index;
                    if let Err(e) = self.repo.stash_drop(idx) {
                        self.error = Some(format!("Stash drop failed: {}", e));
                    } else {
                        self.message = Some("Stash dropped".to_string());
                        self.refresh_stash();
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn handle_diff_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Status;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.diff_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.diff_scroll = self.diff_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.diff_scroll += 20;
            }
            KeyCode::PageUp => {
                self.diff_scroll = self.diff_scroll.saturating_sub(20);
            }
            _ => {}
        }
        false
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                self.view = View::Status;
                self.refresh_status();
            }
            _ => {}
        }
        false
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        match mode {
            Mode::Commit(mut text) => match key.code {
                KeyCode::Enter => {
                    if !text.is_empty() {
                        if let Err(e) = self.repo.commit(&text) {
                            self.error = Some(format!("Commit failed: {}", e));
                        } else {
                            self.message = Some("Committed".to_string());
                            self.refresh_status();
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::Commit(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::Commit(text);
                }
                _ => self.mode = Mode::Commit(text),
            },
            Mode::CreateBranch(mut text) => match key.code {
                KeyCode::Enter => {
                    if !text.is_empty() {
                        if let Err(e) = self.repo.create_branch(&text) {
                            self.error = Some(format!("Create branch failed: {}", e));
                        } else {
                            self.message = Some(format!("Created branch {}", text));
                            self.refresh_branches();
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::CreateBranch(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::CreateBranch(text);
                }
                _ => self.mode = Mode::CreateBranch(text),
            },
            Mode::StashMessage(mut text) => match key.code {
                KeyCode::Enter => {
                    let msg = if text.is_empty() { "WIP" } else { &text };
                    if let Err(e) = self.repo.stash_save(msg) {
                        self.error = Some(format!("Stash failed: {}", e));
                    } else {
                        self.message = Some("Changes stashed".to_string());
                        self.refresh_status();
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    text.pop();
                    self.mode = Mode::StashMessage(text);
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.mode = Mode::StashMessage(text);
                }
                _ => self.mode = Mode::StashMessage(text),
            },
            Mode::Normal => {}
        }
        false
    }
}
