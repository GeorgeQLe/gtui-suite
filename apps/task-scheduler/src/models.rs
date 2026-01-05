//! Data models for task scheduler.

use chrono::{DateTime, Datelike, Utc, Weekday};
use serde::{Deserialize, Serialize};

pub type TaskId = i64;
pub type RunId = i64;

/// A scheduled task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: TaskId,
    pub name: String,
    pub command: String,
    pub schedule: Schedule,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub run_count: u32,
    pub created_at: DateTime<Utc>,
}

impl ScheduledTask {
    pub fn new(name: &str, command: &str, schedule: Schedule) -> Self {
        let now = Utc::now();
        let next_run = schedule.next_occurrence(&now);
        Self {
            id: 0,
            name: name.to_string(),
            command: command.to_string(),
            schedule,
            enabled: true,
            last_run: None,
            next_run,
            run_count: 0,
            created_at: now,
        }
    }

    pub fn update_next_run(&mut self) {
        let now = Utc::now();
        self.next_run = self.schedule.next_occurrence(&now);
    }

    pub fn is_due(&self) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(next) = self.next_run {
            next <= Utc::now()
        } else {
            false
        }
    }
}

/// Schedule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    Once(DateTime<Utc>),
    Interval { minutes: u32 },
    Daily { hour: u32, minute: u32 },
    Weekly { day: Weekday, hour: u32, minute: u32 },
    Monthly { day: u32, hour: u32, minute: u32 },
    Cron(String), // Cron expression
}

impl Schedule {
    pub fn label(&self) -> String {
        match self {
            Schedule::Once(dt) => format!("Once at {}", dt.format("%Y-%m-%d %H:%M")),
            Schedule::Interval { minutes } => format!("Every {} minutes", minutes),
            Schedule::Daily { hour, minute } => format!("Daily at {:02}:{:02}", hour, minute),
            Schedule::Weekly { day, hour, minute } => {
                format!("Weekly on {:?} at {:02}:{:02}", day, hour, minute)
            }
            Schedule::Monthly { day, hour, minute } => {
                format!("Monthly on day {} at {:02}:{:02}", day, hour, minute)
            }
            Schedule::Cron(expr) => format!("Cron: {}", expr),
        }
    }

    pub fn next_occurrence(&self, from: &DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Schedule::Once(dt) => {
                if dt > from { Some(*dt) } else { None }
            }
            Schedule::Interval { minutes } => {
                Some(*from + chrono::Duration::minutes(*minutes as i64))
            }
            Schedule::Daily { hour, minute } => {
                let today = from.date_naive()
                    .and_hms_opt(*hour, *minute, 0)?;
                let dt = today.and_utc();
                if dt > *from {
                    Some(dt)
                } else {
                    Some(dt + chrono::Duration::days(1))
                }
            }
            Schedule::Weekly { day, hour, minute } => {
                let mut check = from.date_naive();
                for _ in 0..8 {
                    if check.weekday() == *day {
                        let dt = check.and_hms_opt(*hour, *minute, 0)?.and_utc();
                        if dt > *from {
                            return Some(dt);
                        }
                    }
                    check = check.succ_opt()?;
                }
                None
            }
            Schedule::Monthly { day, hour, minute } => {
                let mut check_month = from.date_naive().month();
                let mut check_year = from.date_naive().year();

                for _ in 0..13 {
                    if let Some(date) = chrono::NaiveDate::from_ymd_opt(check_year, check_month, *day) {
                        if let Some(time) = date.and_hms_opt(*hour, *minute, 0) {
                            let dt = time.and_utc();
                            if dt > *from {
                                return Some(dt);
                            }
                        }
                    }
                    check_month += 1;
                    if check_month > 12 {
                        check_month = 1;
                        check_year += 1;
                    }
                }
                None
            }
            Schedule::Cron(_) => {
                // Simplified: just return 1 hour from now for demo
                Some(*from + chrono::Duration::hours(1))
            }
        }
    }
}

/// A task execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRun {
    pub id: RunId,
    pub task_id: TaskId,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Running,
    Success,
    Failed,
    Timeout,
    Cancelled,
}

impl RunStatus {
    pub fn label(&self) -> &'static str {
        match self {
            RunStatus::Running => "Running",
            RunStatus::Success => "Success",
            RunStatus::Failed => "Failed",
            RunStatus::Timeout => "Timeout",
            RunStatus::Cancelled => "Cancelled",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            RunStatus::Running => "◔",
            RunStatus::Success => "✓",
            RunStatus::Failed => "✗",
            RunStatus::Timeout => "⏱",
            RunStatus::Cancelled => "⊘",
        }
    }
}
