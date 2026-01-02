//! Data models for time tracking.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ClientId = Uuid;
pub type ProjectId = Uuid;
pub type EntryId = Uuid;

/// A client for billing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
    pub hourly_rate: Option<f64>,
    pub currency: String,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
}

impl Client {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            hourly_rate: None,
            currency: "USD".to_string(),
            archived: false,
            created_at: Utc::now(),
        }
    }

    pub fn with_rate(mut self, rate: f64, currency: impl Into<String>) -> Self {
        self.hourly_rate = Some(rate);
        self.currency = currency.into();
        self
    }
}

/// A project for time tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub client_id: Option<ClientId>,
    pub name: String,
    pub color: Option<String>,
    pub budget_hours: Option<f64>,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id: None,
            name: name.into(),
            color: None,
            budget_hours: None,
            archived: false,
            created_at: Utc::now(),
        }
    }

    pub fn with_client(mut self, client_id: ClientId) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn with_budget(mut self, hours: f64) -> Self {
        self.budget_hours = Some(hours);
        self
    }
}

/// A time entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: EntryId,
    pub project_id: Option<ProjectId>,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_secs: Option<i64>,
    pub tags: Vec<String>,
    pub billable: bool,
    pub created_at: DateTime<Utc>,
}

impl TimeEntry {
    /// Create a new running entry.
    pub fn start(description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id: None,
            description: description.into(),
            start_time: now,
            end_time: None,
            duration_secs: None,
            tags: Vec::new(),
            billable: true,
            created_at: now,
        }
    }

    /// Create a completed entry.
    pub fn create(description: impl Into<String>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let duration = end.signed_duration_since(start);
        Self {
            id: Uuid::new_v4(),
            project_id: None,
            description: description.into(),
            start_time: start,
            end_time: Some(end),
            duration_secs: Some(duration.num_seconds()),
            tags: Vec::new(),
            billable: true,
            created_at: Utc::now(),
        }
    }

    /// Stop a running entry.
    pub fn stop(&mut self) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.duration_secs = Some(now.signed_duration_since(self.start_time).num_seconds());
    }

    /// Check if entry is running.
    pub fn is_running(&self) -> bool {
        self.end_time.is_none()
    }

    /// Get duration.
    pub fn duration(&self) -> Duration {
        if let Some(secs) = self.duration_secs {
            Duration::seconds(secs)
        } else {
            Utc::now().signed_duration_since(self.start_time)
        }
    }

    /// Get duration as hours.
    pub fn hours(&self) -> f64 {
        self.duration().num_minutes() as f64 / 60.0
    }

    /// Format duration as HH:MM:SS.
    pub fn format_duration(&self) -> String {
        let dur = self.duration();
        let hours = dur.num_hours();
        let mins = dur.num_minutes() % 60;
        let secs = dur.num_seconds() % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }

    /// Format duration as HH:MM.
    pub fn format_duration_short(&self) -> String {
        let dur = self.duration();
        let hours = dur.num_hours();
        let mins = dur.num_minutes() % 60;
        format!("{}h {:02}m", hours, mins)
    }
}

/// Report for a time period.
#[derive(Debug, Clone, Default)]
pub struct TimeReport {
    /// Total hours tracked.
    pub total_hours: f64,
    /// Billable hours.
    pub billable_hours: f64,
    /// Hours per project.
    pub by_project: Vec<(Option<ProjectId>, String, f64)>,
    /// Hours per day.
    pub by_day: Vec<(NaiveDate, f64)>,
}

impl TimeReport {
    pub fn billable_percent(&self) -> f64 {
        if self.total_hours > 0.0 {
            self.billable_hours / self.total_hours * 100.0
        } else {
            0.0
        }
    }
}

/// Daily summary.
#[derive(Debug, Clone)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub entries: Vec<TimeEntry>,
    pub total_hours: f64,
    pub billable_hours: f64,
}

impl DailySummary {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            entries: Vec::new(),
            total_hours: 0.0,
            billable_hours: 0.0,
        }
    }

    pub fn add_entry(&mut self, entry: TimeEntry) {
        let hours = entry.hours();
        self.total_hours += hours;
        if entry.billable {
            self.billable_hours += hours;
        }
        self.entries.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_entry_duration() {
        let start = Utc::now() - Duration::hours(2);
        let end = Utc::now();
        let entry = TimeEntry::create("Test", start, end);

        assert!(entry.duration().num_hours() >= 1);
        assert!(!entry.is_running());
    }

    #[test]
    fn test_running_entry() {
        let mut entry = TimeEntry::start("Working");
        assert!(entry.is_running());

        entry.stop();
        assert!(!entry.is_running());
        assert!(entry.duration_secs.is_some());
    }
}
