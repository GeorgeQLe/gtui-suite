use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub columns: Vec<Column>,
    pub sources: Vec<ExternalSource>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Board {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            columns: Vec::new(),
            sources: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_column(&mut self, name: &str) {
        let column = Column::new(name);
        self.columns.push(column);
        self.updated_at = Utc::now();
    }

    pub fn card_count(&self) -> usize {
        self.columns.iter().map(|c| c.cards.len()).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: Uuid,
    pub name: String,
    pub cards: Vec<Card>,
    pub limit: Option<usize>,
}

impl Column {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            cards: Vec::new(),
            limit: None,
        }
    }

    pub fn is_over_limit(&self) -> bool {
        if let Some(limit) = self.limit {
            self.cards.len() > limit
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub checklist: Vec<ChecklistItem>,
    pub link: Option<ExternalLink>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Card {
    pub fn new(title: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: String::new(),
            labels: Vec::new(),
            assignee: None,
            due_date: None,
            priority: Priority::Medium,
            checklist: Vec::new(),
            link: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_linked(&self) -> bool {
        self.link.is_some()
    }

    pub fn checklist_progress(&self) -> (usize, usize) {
        let done = self.checklist.iter().filter(|i| i.done).count();
        (done, self.checklist.len())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Priority::Low => "↓",
            Priority::Medium => "─",
            Priority::High => "↑",
            Priority::Critical => "⚠",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: String,
    pub done: bool,
}

impl ChecklistItem {
    pub fn new(text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.to_string(),
            done: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSource {
    pub id: Uuid,
    pub source_type: SourceType,
    pub name: String,
    pub config: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
}

impl ExternalSource {
    pub fn new(source_type: SourceType, name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_type,
            name: name.to_string(),
            config: String::new(),
            last_sync: None,
            sync_status: SyncStatus::Synced,
        }
    }

    pub fn last_sync_display(&self) -> String {
        if let Some(last) = self.last_sync {
            let elapsed = Utc::now().signed_duration_since(last);
            let secs = elapsed.num_seconds();

            if secs < 60 {
                format!("{}s ago", secs)
            } else if secs < 3600 {
                format!("{}m ago", secs / 60)
            } else {
                format!("{}h ago", secs / 3600)
            }
        } else {
            "Never".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    GitHub,
    GitLab,
    Trello,
    Jira,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::GitHub => "GitHub",
            SourceType::GitLab => "GitLab",
            SourceType::Trello => "Trello",
            SourceType::Jira => "Jira",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SourceType::GitHub => "",
            SourceType::GitLab => "",
            SourceType::Trello => "",
            SourceType::Jira => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalLink {
    pub source_id: Uuid,
    pub external_id: String,
    pub external_url: String,
    pub last_synced: DateTime<Utc>,
    pub sync_status: SyncStatus,
}

impl ExternalLink {
    pub fn new(source_id: Uuid, external_id: &str, url: &str) -> Self {
        Self {
            source_id,
            external_id: external_id.to_string(),
            external_url: url.to_string(),
            last_synced: Utc::now(),
            sync_status: SyncStatus::Synced,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    LocalChanges,
    RemoteChanges,
    Conflict,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncStatus::Synced => "Synced",
            SyncStatus::LocalChanges => "Local changes",
            SyncStatus::RemoteChanges => "Remote changes",
            SyncStatus::Conflict => "Conflict",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SyncStatus::Synced => "✓",
            SyncStatus::LocalChanges => "↑",
            SyncStatus::RemoteChanges => "↓",
            SyncStatus::Conflict => "⚠",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub card_id: Uuid,
    pub local_title: String,
    pub remote_title: String,
    pub field: String,
    pub local_value: String,
    pub remote_value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation() {
        let mut board = Board::new("Test Board");
        board.add_column("Todo");
        board.add_column("Done");

        assert_eq!(board.columns.len(), 2);
    }

    #[test]
    fn test_card_checklist() {
        let mut card = Card::new("Test");
        card.checklist.push(ChecklistItem::new("Item 1"));
        card.checklist.push(ChecklistItem::new("Item 2"));
        card.checklist[0].done = true;

        assert_eq!(card.checklist_progress(), (1, 2));
    }

    #[test]
    fn test_column_limit() {
        let mut column = Column::new("Test");
        column.limit = Some(2);
        column.cards.push(Card::new("Card 1"));
        column.cards.push(Card::new("Card 2"));
        column.cards.push(Card::new("Card 3"));

        assert!(column.is_over_limit());
    }
}
