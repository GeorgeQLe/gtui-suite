//! Data models for Zettelkasten notes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type ZettelId = String;
pub type DbId = i64;

/// Zettel types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZettelType {
    Fleeting,   // Quick capture
    Literature, // From a source
    Permanent,  // Refined, atomic idea
    Hub,        // Index/structure note
}

impl ZettelType {
    pub fn label(&self) -> &'static str {
        match self {
            ZettelType::Fleeting => "Fleeting",
            ZettelType::Literature => "Literature",
            ZettelType::Permanent => "Permanent",
            ZettelType::Hub => "Hub",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ZettelType::Fleeting => "✎",
            ZettelType::Literature => "📖",
            ZettelType::Permanent => "◆",
            ZettelType::Hub => "◎",
        }
    }
}

/// A Zettel (slip/note).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zettel {
    pub db_id: DbId,
    pub id: ZettelId,        // Unique ID (timestamp-based)
    pub title: String,
    pub content: String,
    pub zettel_type: ZettelType,
    pub tags: Vec<String>,
    pub source: Option<String>, // For literature notes
    pub sequence: Option<String>, // For ordering (1a, 1b, 2a, etc.)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Zettel {
    pub fn new(title: &str, zettel_type: ZettelType) -> Self {
        let now = Utc::now();
        let id = now.format("%Y%m%d%H%M%S").to_string();
        Self {
            db_id: 0,
            id,
            title: title.to_string(),
            content: String::new(),
            zettel_type,
            tags: Vec::new(),
            source: None,
            sequence: None,
            created_at: now,
            updated_at: now,
        }
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

    pub fn formatted_id(&self) -> String {
        if let Some(seq) = &self.sequence {
            format!("{} ({})", self.id, seq)
        } else {
            self.id.clone()
        }
    }
}

/// A link between two zettels.
#[derive(Debug, Clone)]
pub struct ZettelLink {
    pub source_id: ZettelId,
    pub target_id: ZettelId,
    pub link_type: LinkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    Reference,   // General reference
    Continues,   // Continuation of idea
    Supports,    // Supporting evidence
    Contradicts, // Counter-argument
    Related,     // Loosely related
}

impl LinkType {
    pub fn label(&self) -> &'static str {
        match self {
            LinkType::Reference => "references",
            LinkType::Continues => "continues",
            LinkType::Supports => "supports",
            LinkType::Contradicts => "contradicts",
            LinkType::Related => "related to",
        }
    }
}

/// Zettelkasten statistics.
#[derive(Debug, Clone, Default)]
pub struct ZkStats {
    pub total_zettels: usize,
    pub fleeting: usize,
    pub literature: usize,
    pub permanent: usize,
    pub hubs: usize,
    pub total_links: usize,
    pub total_tags: usize,
}
