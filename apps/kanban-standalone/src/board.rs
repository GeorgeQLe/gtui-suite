#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn symbol(&self) -> &'static str {
        match self {
            Priority::Urgent => "⚡",
            Priority::High => "↑",
            Priority::Medium => "→",
            Priority::Low => "↓",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Priority::Urgent => Priority::High,
            Priority::High => Priority::Medium,
            Priority::Medium => Priority::Low,
            Priority::Low => Priority::Urgent,
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub labels: Vec<Label>,
    pub checklist: Vec<ChecklistItem>,
    pub comments: Vec<Comment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
}

impl Card {
    pub fn new(column_id: Uuid, title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            column_id,
            title,
            description: None,
            position: 0,
            priority: Priority::default(),
            due_date: None,
            labels: Vec::new(),
            checklist: Vec::new(),
            comments: Vec::new(),
            created_at: now,
            updated_at: now,
            archived: false,
        }
    }

    pub fn checklist_progress(&self) -> (usize, usize) {
        let total = self.checklist.len();
        let completed = self.checklist.iter().filter(|c| c.completed).count();
        (completed, total)
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            due < Utc::now().date_naive()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: i32,
    pub wip_limit: Option<u32>,
    pub color: Option<String>,
}

impl Column {
    pub fn new(board_id: Uuid, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            board_id,
            name,
            position: 0,
            wip_limit: None,
            color: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
}

impl Board {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            archived: false,
            created_at: Utc::now(),
        }
    }
}
