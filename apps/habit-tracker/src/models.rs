//! Data models for habit tracking.

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique habit identifier.
pub type HabitId = Uuid;

/// A habit to track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    /// Unique identifier.
    pub id: HabitId,
    /// Habit name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// When the habit is scheduled.
    pub schedule: Schedule,
    /// Type of metric (binary or quantitative).
    pub metric: Metric,
    /// Display color.
    pub color: Option<String>,
    /// When the habit was created.
    pub created_at: DateTime<Utc>,
    /// Whether the habit is archived.
    pub archived: bool,
}

impl Habit {
    /// Create a new binary habit.
    pub fn new_binary(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            schedule: Schedule::Daily,
            metric: Metric::Binary,
            color: None,
            created_at: Utc::now(),
            archived: false,
        }
    }

    /// Create a new quantitative habit.
    pub fn new_quantitative(name: impl Into<String>, goal: f64, unit: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            schedule: Schedule::Daily,
            metric: Metric::Quantity {
                goal,
                unit: unit.into(),
            },
            color: None,
            created_at: Utc::now(),
            archived: false,
        }
    }

    /// Check if the habit is due on a given date.
    pub fn is_due_on(&self, date: NaiveDate) -> bool {
        self.schedule.is_due_on(date)
    }

    /// Check if this is a binary habit.
    pub fn is_binary(&self) -> bool {
        matches!(self.metric, Metric::Binary)
    }

    /// Get the goal for quantitative habits.
    pub fn goal(&self) -> Option<f64> {
        match &self.metric {
            Metric::Binary => None,
            Metric::Quantity { goal, .. } => Some(*goal),
        }
    }

    /// Get the unit for quantitative habits.
    pub fn unit(&self) -> Option<&str> {
        match &self.metric {
            Metric::Binary => None,
            Metric::Quantity { unit, .. } => Some(unit),
        }
    }
}

/// Habit schedule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Schedule {
    /// Every day.
    Daily,
    /// Specific days of the week.
    Weekly { days: Vec<Weekday> },
    /// Specific days of the month.
    Monthly { days: Vec<u8> },
    /// Every N days.
    Interval { every_n_days: u32, start_date: NaiveDate },
}

impl Schedule {
    /// Check if the schedule is due on a given date.
    pub fn is_due_on(&self, date: NaiveDate) -> bool {
        match self {
            Self::Daily => true,
            Self::Weekly { days } => days.contains(&date.weekday()),
            Self::Monthly { days } => days.contains(&(date.day() as u8)),
            Self::Interval { every_n_days, start_date } => {
                let diff = date.signed_duration_since(*start_date).num_days();
                diff >= 0 && diff as u32 % every_n_days == 0
            }
        }
    }

    /// Get display name.
    pub fn display(&self) -> String {
        match self {
            Self::Daily => "Daily".to_string(),
            Self::Weekly { days } => {
                let day_names: Vec<_> = days.iter().map(|d| format!("{:?}", d)).collect();
                format!("Weekly: {}", day_names.join(", "))
            }
            Self::Monthly { days } => {
                let day_strs: Vec<_> = days.iter().map(|d| d.to_string()).collect();
                format!("Monthly: {}", day_strs.join(", "))
            }
            Self::Interval { every_n_days, .. } => format!("Every {} days", every_n_days),
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::Daily
    }
}

/// Metric type for habits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Metric {
    /// Simple yes/no completion.
    Binary,
    /// Numeric goal.
    Quantity { goal: f64, unit: String },
}

impl Metric {
    /// Get display string.
    pub fn display(&self) -> String {
        match self {
            Self::Binary => "Yes/No".to_string(),
            Self::Quantity { goal, unit } => format!("{} {}", goal, unit),
        }
    }
}

impl Default for Metric {
    fn default() -> Self {
        Self::Binary
    }
}

/// A single entry for a habit on a specific date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitEntry {
    /// Unique identifier.
    pub id: Uuid,
    /// Associated habit.
    pub habit_id: HabitId,
    /// Date of the entry.
    pub date: NaiveDate,
    /// Whether the habit was completed.
    pub completed: bool,
    /// Value for quantitative habits.
    pub value: Option<f64>,
    /// Optional notes.
    pub notes: Option<String>,
    /// When the entry was created.
    pub created_at: DateTime<Utc>,
}

impl HabitEntry {
    /// Create a new binary entry.
    pub fn new_binary(habit_id: HabitId, date: NaiveDate, completed: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            habit_id,
            date,
            completed,
            value: None,
            notes: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new quantitative entry.
    pub fn new_quantity(habit_id: HabitId, date: NaiveDate, value: f64, goal: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            habit_id,
            date,
            completed: value >= goal,
            value: Some(value),
            notes: None,
            created_at: Utc::now(),
        }
    }

    /// Get completion percentage.
    pub fn completion_percent(&self, goal: Option<f64>) -> f64 {
        if self.completed {
            return 100.0;
        }

        match (self.value, goal) {
            (Some(v), Some(g)) if g > 0.0 => (v / g * 100.0).min(100.0),
            _ => 0.0,
        }
    }
}

/// Streak information for a habit.
#[derive(Debug, Clone, Default)]
pub struct StreakInfo {
    /// Current streak count.
    pub current: u32,
    /// Best streak ever.
    pub best: u32,
    /// Last date in current streak.
    pub last_date: Option<NaiveDate>,
}

/// Correlation between two habits.
#[derive(Debug, Clone)]
pub struct Correlation {
    /// First habit.
    pub habit_a: HabitId,
    /// Second habit.
    pub habit_b: HabitId,
    /// Correlation coefficient (-1.0 to 1.0).
    pub coefficient: f64,
    /// Statistical significance.
    pub significance: f64,
}

impl Correlation {
    /// Check if correlation is significant.
    pub fn is_significant(&self) -> bool {
        self.significance < 0.05
    }

    /// Get description of correlation.
    pub fn description(&self) -> &'static str {
        let abs = self.coefficient.abs();
        if abs < 0.3 {
            "weak"
        } else if abs < 0.7 {
            "moderate"
        } else {
            "strong"
        }
    }
}

/// Statistics for a habit.
#[derive(Debug, Clone, Default)]
pub struct HabitStats {
    /// Total entries.
    pub total_entries: u32,
    /// Completed entries.
    pub completed_entries: u32,
    /// Completion rate (0.0 to 1.0).
    pub completion_rate: f64,
    /// Current streak.
    pub current_streak: u32,
    /// Best streak.
    pub best_streak: u32,
    /// Average value (for quantitative).
    pub average_value: Option<f64>,
    /// Total value (for quantitative).
    pub total_value: Option<f64>,
}

impl HabitStats {
    /// Calculate completion percentage.
    pub fn completion_percent(&self) -> f64 {
        self.completion_rate * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_schedule() {
        let schedule = Schedule::Daily;
        assert!(schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert!(schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()));
    }

    #[test]
    fn test_weekly_schedule() {
        let schedule = Schedule::Weekly {
            days: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
        };
        // Jan 1, 2024 is Monday
        assert!(schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        // Jan 2, 2024 is Tuesday
        assert!(!schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()));
        // Jan 3, 2024 is Wednesday
        assert!(schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap()));
    }

    #[test]
    fn test_interval_schedule() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let schedule = Schedule::Interval {
            every_n_days: 3,
            start_date: start,
        };

        assert!(schedule.is_due_on(start));
        assert!(!schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()));
        assert!(!schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap()));
        assert!(schedule.is_due_on(NaiveDate::from_ymd_opt(2024, 1, 4).unwrap()));
    }

    #[test]
    fn test_habit_creation() {
        let habit = Habit::new_binary("Exercise");
        assert_eq!(habit.name, "Exercise");
        assert!(habit.is_binary());

        let habit = Habit::new_quantitative("Water", 8.0, "glasses");
        assert!(!habit.is_binary());
        assert_eq!(habit.goal(), Some(8.0));
        assert_eq!(habit.unit(), Some("glasses"));
    }
}
