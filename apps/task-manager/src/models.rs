//! Data models for task manager.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique task identifier.
pub type TaskId = i64;
/// Unique project identifier.
pub type ProjectId = i64;
/// Unique context identifier.
pub type ContextId = i64;

/// Task priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    None,
    #[default]
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub fn label(&self) -> &'static str {
        match self {
            Priority::None => "",
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Urgent => "Urgent",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Priority::None => " ",
            Priority::Low => "○",
            Priority::Medium => "◐",
            Priority::High => "●",
            Priority::Urgent => "◉",
        }
    }

    pub fn next(&self) -> Priority {
        match self {
            Priority::None => Priority::Low,
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::Urgent,
            Priority::Urgent => Priority::None,
        }
    }
}

/// Task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Status {
    #[default]
    Todo,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

impl Status {
    pub fn label(&self) -> &'static str {
        match self {
            Status::Todo => "Todo",
            Status::InProgress => "In Progress",
            Status::Blocked => "Blocked",
            Status::Done => "Done",
            Status::Cancelled => "Cancelled",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Status::Todo => "☐",
            Status::InProgress => "◔",
            Status::Blocked => "⊘",
            Status::Done => "☑",
            Status::Cancelled => "☒",
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Status::Done | Status::Cancelled)
    }

    pub fn cycle(&self) -> Status {
        match self {
            Status::Todo => Status::InProgress,
            Status::InProgress => Status::Done,
            Status::Blocked => Status::Todo,
            Status::Done => Status::Todo,
            Status::Cancelled => Status::Todo,
        }
    }
}

/// Recurrence pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Recurrence {
    Daily,
    Weekly { days: Vec<u8> }, // 0=Sun, 1=Mon, etc.
    Monthly { day: u8 },
    Yearly { month: u8, day: u8 },
    Every { days: u32 },
}

impl Recurrence {
    pub fn label(&self) -> String {
        match self {
            Recurrence::Daily => "Daily".to_string(),
            Recurrence::Weekly { days } => {
                let day_names: Vec<&str> = days.iter().map(|d| match d {
                    0 => "Sun",
                    1 => "Mon",
                    2 => "Tue",
                    3 => "Wed",
                    4 => "Thu",
                    5 => "Fri",
                    6 => "Sat",
                    _ => "?",
                }).collect();
                format!("Weekly ({})", day_names.join(", "))
            }
            Recurrence::Monthly { day } => format!("Monthly (day {})", day),
            Recurrence::Yearly { month, day } => format!("Yearly ({}/{})", month, day),
            Recurrence::Every { days } => format!("Every {} days", days),
        }
    }
}

/// A task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub project_id: Option<ProjectId>,
    pub context_id: Option<ContextId>,
    pub tags: Vec<String>,
    pub due_date: Option<NaiveDate>,
    pub scheduled_date: Option<NaiveDate>,
    pub recurrence: Option<Recurrence>,
    pub estimated_mins: Option<u32>,
    pub actual_mins: Option<u32>,
    pub parent_id: Option<TaskId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            uuid: Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: String::new(),
            status: Status::Todo,
            priority: Priority::default(),
            project_id: None,
            context_id: None,
            tags: Vec::new(),
            due_date: None,
            scheduled_date: None,
            recurrence: None,
            estimated_mins: None,
            actual_mins: None,
            parent_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            !self.status.is_complete() && due < Utc::now().date_naive()
        } else {
            false
        }
    }

    pub fn is_due_today(&self) -> bool {
        if let Some(due) = self.due_date {
            due == Utc::now().date_naive()
        } else {
            false
        }
    }

    pub fn is_scheduled_today(&self) -> bool {
        if let Some(sched) = self.scheduled_date {
            sched == Utc::now().date_naive()
        } else {
            false
        }
    }

    pub fn days_until_due(&self) -> Option<i64> {
        self.due_date.map(|d| {
            let today = Utc::now().date_naive();
            (d - today).num_days()
        })
    }
}

/// A project groups related tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: String,
    pub color: Option<String>,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            description: String::new(),
            color: None,
            archived: false,
            created_at: Utc::now(),
        }
    }
}

/// A context represents where/how you work (e.g., @home, @office, @phone).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: ContextId,
    pub name: String,
    pub description: String,
}

impl Context {
    pub fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            description: String::new(),
        }
    }
}

/// Statistics for tasks.
#[derive(Debug, Clone, Default)]
pub struct TaskStats {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
    pub overdue: usize,
    pub due_today: usize,
}
