#![allow(dead_code)]

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use git2::{BranchType, DiffOptions, Repository, StatusOptions};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: FileState,
    pub staged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileState {
    New,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Conflicted,
    Untracked,
}

impl FileState {
    pub fn symbol(&self) -> &'static str {
        match self {
            FileState::New => "A",
            FileState::Modified => "M",
            FileState::Deleted => "D",
            FileState::Renamed => "R",
            FileState::Typechange => "T",
            FileState::Conflicted => "U",
            FileState::Untracked => "?",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
}

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path).context("Could not find git repository")?;
        Ok(Self { repo })
    }

    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    pub fn head_name(&self) -> String {
        self.repo
            .head()
            .ok()
            .and_then(|r| r.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "HEAD".to_string())
    }

    pub fn is_rebasing(&self) -> bool {
        self.repo.state() == git2::RepositoryState::Rebase
            || self.repo.state() == git2::RepositoryState::RebaseInteractive
            || self.repo.state() == git2::RepositoryState::RebaseMerge
    }

    pub fn is_merging(&self) -> bool {
        self.repo.state() == git2::RepositoryState::Merge
    }

    pub fn status(&self) -> Result<Vec<FileStatus>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_unmodified(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            // Check index (staged) status
            if status.is_index_new() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::New,
                    staged: true,
                });
            } else if status.is_index_modified() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Modified,
                    staged: true,
                });
            } else if status.is_index_deleted() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Deleted,
                    staged: true,
                });
            } else if status.is_index_renamed() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Renamed,
                    staged: true,
                });
            } else if status.is_index_typechange() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Typechange,
                    staged: true,
                });
            }

            // Check worktree (unstaged) status
            if status.is_wt_new() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Untracked,
                    staged: false,
                });
            } else if status.is_wt_modified() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Modified,
                    staged: false,
                });
            } else if status.is_wt_deleted() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Deleted,
                    staged: false,
                });
            } else if status.is_wt_renamed() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Renamed,
                    staged: false,
                });
            } else if status.is_wt_typechange() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileState::Typechange,
                    staged: false,
                });
            }

            if status.is_conflicted() {
                files.push(FileStatus {
                    path,
                    status: FileState::Conflicted,
                    staged: false,
                });
            }
        }

        Ok(files)
    }

    pub fn staged_files(&self) -> Result<Vec<FileStatus>> {
        Ok(self.status()?.into_iter().filter(|f| f.staged).collect())
    }

    pub fn unstaged_files(&self) -> Result<Vec<FileStatus>> {
        Ok(self.status()?.into_iter().filter(|f| !f.staged).collect())
    }

    pub fn stage_file(&self, path: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_path(Path::new(path))?;
        index.write()?;
        Ok(())
    }

    pub fn unstage_file(&self, path: &str) -> Result<()> {
        let head = self.repo.head()?.peel_to_tree()?;
        self.repo.reset_default(Some(&head.into_object()), [path])?;
        Ok(())
    }

    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let signature = self.repo.signature()?;
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let parent_commit = self.repo.head()?.peel_to_commit()?;
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;
        Ok(())
    }

    pub fn log(&self, count: usize) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let commits: Vec<CommitInfo> = revwalk
            .take(count)
            .filter_map(|oid| {
                let oid = oid.ok()?;
                let commit = self.repo.find_commit(oid).ok()?;
                let time = Utc
                    .timestamp_opt(commit.time().seconds(), 0)
                    .single()
                    .unwrap_or_else(Utc::now);

                let id = commit.id().to_string();
                let short_id = id[..7].to_string();
                let message = commit.summary().unwrap_or("").to_string();
                let author_sig = commit.author();
                let author = author_sig.name().unwrap_or("").to_string();
                let email = author_sig.email().unwrap_or("").to_string();

                Some(CommitInfo {
                    id,
                    short_id,
                    message,
                    author,
                    email,
                    time,
                })
            })
            .collect();

        Ok(commits)
    }

    pub fn branches(&self) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();
        let head = self.repo.head().ok();
        let head_name = head
            .as_ref()
            .and_then(|r| r.shorthand())
            .unwrap_or("");

        // Local branches
        for branch in self.repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch?;
            let name = branch.name()?.unwrap_or("").to_string();
            let is_current = name == head_name;

            let (ahead, behind) = if let Ok(upstream) = branch.upstream() {
                let local_oid = branch.get().target();
                let upstream_oid = upstream.get().target();
                if let (Some(l), Some(u)) = (local_oid, upstream_oid) {
                    self.repo.graph_ahead_behind(l, u).unwrap_or((0, 0))
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            branches.push(BranchInfo {
                name,
                is_current,
                is_remote: false,
                upstream: branch.upstream().ok().and_then(|u| u.name().ok().flatten().map(|s| s.to_string())),
                ahead,
                behind,
            });
        }

        // Remote branches
        for branch in self.repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch?;
            let name = branch.name()?.unwrap_or("").to_string();
            branches.push(BranchInfo {
                name,
                is_current: false,
                is_remote: true,
                upstream: None,
                ahead: 0,
                behind: 0,
            });
        }

        Ok(branches)
    }

    pub fn create_branch(&self, name: &str) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo.branch(name, &head, false)?;
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let (object, reference) = self.repo.revparse_ext(name)?;
        self.repo.checkout_tree(&object, None)?;
        if let Some(reference) = reference {
            self.repo.set_head(reference.name().unwrap_or(name))?;
        } else {
            self.repo.set_head_detached(object.id())?;
        }
        Ok(())
    }

    pub fn delete_branch(&self, name: &str) -> Result<()> {
        let mut branch = self.repo.find_branch(name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    pub fn stash_list(&mut self) -> Result<Vec<StashEntry>> {
        let mut stashes = Vec::new();
        self.repo.stash_foreach(|index, message, _oid| {
            stashes.push(StashEntry {
                index,
                message: message.to_string(),
            });
            true
        })?;
        Ok(stashes)
    }

    pub fn stash_save(&mut self, message: &str) -> Result<()> {
        let signature = self.repo.signature()?;
        self.repo.stash_save(&signature, message, None)?;
        Ok(())
    }

    pub fn stash_pop(&mut self, index: usize) -> Result<()> {
        self.repo.stash_pop(index, None)?;
        Ok(())
    }

    pub fn stash_drop(&mut self, index: usize) -> Result<()> {
        self.repo.stash_drop(index)?;
        Ok(())
    }

    pub fn diff_file(&self, path: &str, staged: bool) -> Result<String> {
        let mut opts = DiffOptions::new();
        opts.pathspec(path);

        let diff = if staged {
            let head_tree = self.repo.head()?.peel_to_tree()?;
            self.repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut opts))?
        } else {
            self.repo.diff_index_to_workdir(None, Some(&mut opts))?
        };

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let prefix = match line.origin() {
                '+' => "+",
                '-' => "-",
                ' ' => " ",
                _ => "",
            };
            diff_text.push_str(prefix);
            if let Ok(content) = std::str::from_utf8(line.content()) {
                diff_text.push_str(content);
            }
            true
        })?;

        Ok(diff_text)
    }

    pub fn fetch(&self, remote_name: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name)?;
        remote.fetch(&[] as &[&str], None, None)?;
        Ok(())
    }

    pub fn pull(&self) -> Result<()> {
        // Simplified pull - fetch + merge
        self.fetch("origin")?;

        // Get current branch's upstream
        let head = self.repo.head()?;
        let branch_name = head.shorthand().unwrap_or("main");
        let remote_ref = format!("origin/{}", branch_name);

        // Merge
        let fetch_head = self.repo.find_reference(&format!("refs/remotes/{}", remote_ref))?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;

        let (analysis, _) = self.repo.merge_analysis(&[&fetch_commit])?;

        if analysis.is_fast_forward() {
            let mut reference = self.repo.find_reference(&format!("refs/heads/{}", branch_name))?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else if analysis.is_normal() {
            self.repo.merge(&[&fetch_commit], None, None)?;
        }

        Ok(())
    }

    pub fn push(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let head = self.repo.head()?;
        let branch_name = head.shorthand().unwrap_or("main");
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        remote.push(&[&refspec], None)?;
        Ok(())
    }
}
