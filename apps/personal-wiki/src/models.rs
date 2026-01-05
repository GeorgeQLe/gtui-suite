//! Data models for personal wiki.

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::LazyLock;

pub type PageId = i64;
pub type CategoryId = i64;
pub type RevisionId = i64;

static LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap()
});

static CATEGORY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[Category:([^\]]+)\]\]").unwrap()
});

/// A wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub title: String,
    pub content: String,
    pub categories: Vec<String>,
    pub redirect_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Page {
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            title: title.to_string(),
            content: String::new(),
            categories: Vec::new(),
            redirect_to: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Extract all [[links]] from the content (excluding category links).
    pub fn extract_links(&self) -> HashSet<String> {
        LINK_REGEX
            .captures_iter(&self.content)
            .filter_map(|cap| {
                cap.get(1).map(|m| {
                    let link = m.as_str().trim();
                    if !link.starts_with("Category:") {
                        Some(link.to_string())
                    } else {
                        None
                    }
                }).flatten()
            })
            .collect()
    }

    /// Extract all [[Category:Name]] tags from the content.
    pub fn extract_categories(&self) -> Vec<String> {
        CATEGORY_REGEX
            .captures_iter(&self.content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .collect()
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn preview(&self, max_lines: usize) -> String {
        self.content
            .lines()
            .filter(|l| !l.starts_with("[[Category:"))
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn is_redirect(&self) -> bool {
        self.redirect_to.is_some()
    }
}

/// A category for organizing pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub description: String,
    pub page_count: usize,
}

impl Category {
    pub fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            description: String::new(),
            page_count: 0,
        }
    }
}

/// A revision of a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub id: RevisionId,
    pub page_id: PageId,
    pub content: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

/// Wiki statistics.
#[derive(Debug, Clone, Default)]
pub struct WikiStats {
    pub total_pages: usize,
    pub total_categories: usize,
    pub total_revisions: usize,
    pub total_links: usize,
    pub orphan_pages: usize,
}

/// Special page types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialPage {
    AllPages,
    RecentChanges,
    Categories,
    OrphanPages,
    WantedPages,
    Random,
}

impl SpecialPage {
    pub fn title(&self) -> &'static str {
        match self {
            SpecialPage::AllPages => "All Pages",
            SpecialPage::RecentChanges => "Recent Changes",
            SpecialPage::Categories => "Categories",
            SpecialPage::OrphanPages => "Orphan Pages",
            SpecialPage::WantedPages => "Wanted Pages",
            SpecialPage::Random => "Random Page",
        }
    }
}
