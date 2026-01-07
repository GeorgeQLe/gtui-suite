use similar::{ChangeTag, TextDiff};

/// A line change in the diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub tag: ChangeTag,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

/// Diff hunk (group of changes)
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

/// File diff result
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
}

impl FileDiff {
    pub fn compute(old_path: &str, old_content: &str, new_path: &str, new_content: &str) -> Self {
        let diff = TextDiff::from_lines(old_content, new_content);
        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut old_line = 0usize;
        let mut new_line = 0usize;

        for change in diff.iter_all_changes() {
            let tag = change.tag();
            let content = change.value().to_string();

            let line = DiffLine {
                tag,
                old_line: if tag != ChangeTag::Insert { old_line += 1; Some(old_line) } else { None },
                new_line: if tag != ChangeTag::Delete { new_line += 1; Some(new_line) } else { None },
                content,
            };

            if tag != ChangeTag::Equal {
                if current_hunk.is_none() {
                    current_hunk = Some(DiffHunk {
                        old_start: old_line.saturating_sub(1),
                        old_count: 0,
                        new_start: new_line.saturating_sub(1),
                        new_count: 0,
                        lines: Vec::new(),
                    });
                }
                if let Some(ref mut hunk) = current_hunk {
                    hunk.lines.push(line);
                    match tag {
                        ChangeTag::Delete => hunk.old_count += 1,
                        ChangeTag::Insert => hunk.new_count += 1,
                        _ => {}
                    }
                }
            } else if let Some(hunk) = current_hunk.take() {
                if !hunk.lines.is_empty() {
                    hunks.push(hunk);
                }
            }
        }

        if let Some(hunk) = current_hunk {
            if !hunk.lines.is_empty() {
                hunks.push(hunk);
            }
        }

        Self {
            old_path: old_path.to_string(),
            new_path: new_path.to_string(),
            hunks,
            is_binary: false,
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.hunks.is_empty()
    }

    pub fn total_additions(&self) -> usize {
        self.hunks.iter().map(|h| h.new_count).sum()
    }

    pub fn total_deletions(&self) -> usize {
        self.hunks.iter().map(|h| h.old_count).sum()
    }
}

/// Directory diff entry
#[derive(Debug, Clone)]
pub struct DirDiffEntry {
    pub path: String,
    pub status: DiffStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

impl DiffStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            DiffStatus::Added => "+",
            DiffStatus::Deleted => "-",
            DiffStatus::Modified => "~",
            DiffStatus::Unchanged => " ",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DiffStatus::Added => "Added",
            DiffStatus::Deleted => "Deleted",
            DiffStatus::Modified => "Modified",
            DiffStatus::Unchanged => "Unchanged",
        }
    }
}
