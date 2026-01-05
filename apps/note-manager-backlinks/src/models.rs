//! Data models for backlink-based notes.

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::LazyLock;

pub type NoteId = i64;

static LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[([^\]]+)\]\]").unwrap()
});

/// A note with wiki-style links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            title: title.to_string(),
            content: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Extract all [[links]] from the content.
    pub fn extract_links(&self) -> HashSet<String> {
        LINK_REGEX
            .captures_iter(&self.content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn preview(&self, max_chars: usize) -> String {
        if self.content.len() <= max_chars {
            self.content.clone()
        } else {
            format!("{}...", &self.content[..max_chars])
        }
    }
}

/// A link between two notes.
#[derive(Debug, Clone)]
pub struct Link {
    pub source_id: NoteId,
    pub target_title: String,
    pub target_id: Option<NoteId>,
}

/// Statistics for a note.
#[derive(Debug, Clone, Default)]
pub struct NoteStats {
    pub outgoing_links: usize,
    pub incoming_links: usize,
    pub word_count: usize,
}

/// Search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub note_id: NoteId,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

/// Graph node for visualization.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: NoteId,
    pub title: String,
    pub x: f32,
    pub y: f32,
    pub connections: usize,
}

/// Represents the view mode for notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    List,
    Backlinks,
    Graph,
}
