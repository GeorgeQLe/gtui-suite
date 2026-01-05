//! Data models for folder-based notes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for notes/folders.
pub type NodeId = String;

/// A node in the folder tree (either a folder or note).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    Folder(Folder),
    Note(Note),
}

impl Node {
    pub fn id(&self) -> &str {
        match self {
            Node::Folder(f) => &f.id,
            Node::Note(n) => &n.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::Folder(f) => &f.name,
            Node::Note(n) => &n.title,
        }
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, Node::Folder(_))
    }

    pub fn path(&self) -> &PathBuf {
        match self {
            Node::Folder(f) => &f.path,
            Node::Note(n) => &n.path,
        }
    }
}

/// A folder containing notes and subfolders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: NodeId,
    pub name: String,
    pub path: PathBuf,
    pub parent_id: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub expanded: bool,
    pub created_at: DateTime<Utc>,
}

impl Folder {
    pub fn new(name: &str, path: PathBuf, parent_id: Option<NodeId>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            path,
            parent_id,
            children: Vec::new(),
            expanded: false,
            created_at: Utc::now(),
        }
    }
}

/// A markdown note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NodeId,
    pub title: String,
    pub content: String,
    pub path: PathBuf,
    pub parent_id: NodeId,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(title: &str, path: PathBuf, parent_id: NodeId) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: String::new(),
            path,
            parent_id,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn preview(&self, max_lines: usize) -> String {
        self.content
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Tree item for display.
#[derive(Debug, Clone)]
pub struct TreeItem {
    pub id: NodeId,
    pub name: String,
    pub is_folder: bool,
    pub depth: usize,
    pub expanded: bool,
    pub has_children: bool,
}

/// Search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub note_id: NodeId,
    pub title: String,
    pub path: PathBuf,
    pub snippet: String,
    pub match_count: usize,
}
