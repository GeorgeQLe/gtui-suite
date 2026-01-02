//! Database operations for habit tracking.

use crate::models::{Habit, HabitEntry, HabitId, HabitStats, Metric, Schedule, StreakInfo};
use chrono::{NaiveDate, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

/// Database errors.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Habit not found: {0}")]
    NotFound(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Database connection wrapper.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at path.
    pub fn open(path: &Path) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    /// Create in-memory database (for testing).
    pub fn in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    /// Initialize database schema.
    fn init(&self) -> DbResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS habits (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                schedule_type TEXT NOT NULL,
                schedule_data TEXT,
                metric_type TEXT NOT NULL,
                metric_goal REAL,
                metric_unit TEXT,
                color TEXT,
                created_at TEXT NOT NULL,
                archived INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS habit_entries (
                id TEXT PRIMARY KEY,
                habit_id TEXT NOT NULL REFERENCES habits(id),
                date TEXT NOT NULL,
                completed INTEGER NOT NULL,
                value REAL,
                notes TEXT,
                created_at TEXT NOT NULL,
                UNIQUE(habit_id, date)
            );

            CREATE INDEX IF NOT EXISTS idx_entries_date ON habit_entries(date);
            CREATE INDEX IF NOT EXISTS idx_entries_habit ON habit_entries(habit_id);
            "#,
        )?;
        Ok(())
    }

    /// Insert a new habit.
    pub fn insert_habit(&self, habit: &Habit) -> DbResult<()> {
        let (schedule_type, schedule_data) = serialize_schedule(&habit.schedule)?;
        let (metric_type, metric_goal, metric_unit) = serialize_metric(&habit.metric);

        self.conn.execute(
            r#"
            INSERT INTO habits (id, name, description, schedule_type, schedule_data,
                               metric_type, metric_goal, metric_unit, color, created_at, archived)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                habit.id.to_string(),
                habit.name,
                habit.description,
                schedule_type,
                schedule_data,
                metric_type,
                metric_goal,
                metric_unit,
                habit.color,
                habit.created_at.to_rfc3339(),
                habit.archived as i32,
            ],
        )?;
        Ok(())
    }

    /// Update an existing habit.
    pub fn update_habit(&self, habit: &Habit) -> DbResult<()> {
        let (schedule_type, schedule_data) = serialize_schedule(&habit.schedule)?;
        let (metric_type, metric_goal, metric_unit) = serialize_metric(&habit.metric);

        self.conn.execute(
            r#"
            UPDATE habits SET
                name = ?2, description = ?3, schedule_type = ?4, schedule_data = ?5,
                metric_type = ?6, metric_goal = ?7, metric_unit = ?8, color = ?9, archived = ?10
            WHERE id = ?1
            "#,
            params![
                habit.id.to_string(),
                habit.name,
                habit.description,
                schedule_type,
                schedule_data,
                metric_type,
                metric_goal,
                metric_unit,
                habit.color,
                habit.archived as i32,
            ],
        )?;
        Ok(())
    }

    /// Delete a habit and all its entries.
    pub fn delete_habit(&self, id: HabitId) -> DbResult<()> {
        self.conn.execute(
            "DELETE FROM habit_entries WHERE habit_id = ?1",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM habits WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    /// Get a habit by ID.
    pub fn get_habit(&self, id: HabitId) -> DbResult<Option<Habit>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM habits WHERE id = ?1",
        )?;

        let habit = stmt.query_row(params![id.to_string()], |row| {
            Ok(parse_habit_row(row)?)
        });

        match habit {
            Ok(h) => Ok(Some(h)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all habits (optionally including archived).
    pub fn list_habits(&self, include_archived: bool) -> DbResult<Vec<Habit>> {
        let sql = if include_archived {
            "SELECT * FROM habits ORDER BY created_at"
        } else {
            "SELECT * FROM habits WHERE archived = 0 ORDER BY created_at"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let habits = stmt
            .query_map([], |row| Ok(parse_habit_row(row)?))?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(habits)
    }

    /// Get habits due on a specific date.
    pub fn get_habits_due_on(&self, date: NaiveDate) -> DbResult<Vec<Habit>> {
        let all_habits = self.list_habits(false)?;
        Ok(all_habits.into_iter().filter(|h| h.is_due_on(date)).collect())
    }

    /// Insert or update a habit entry.
    pub fn upsert_entry(&self, entry: &HabitEntry) -> DbResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO habit_entries (id, habit_id, date, completed, value, notes, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(habit_id, date) DO UPDATE SET
                completed = excluded.completed,
                value = excluded.value,
                notes = excluded.notes
            "#,
            params![
                entry.id.to_string(),
                entry.habit_id.to_string(),
                entry.date.to_string(),
                entry.completed as i32,
                entry.value,
                entry.notes,
                entry.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get entry for a habit on a specific date.
    pub fn get_entry(&self, habit_id: HabitId, date: NaiveDate) -> DbResult<Option<HabitEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM habit_entries WHERE habit_id = ?1 AND date = ?2",
        )?;

        let entry = stmt.query_row(
            params![habit_id.to_string(), date.to_string()],
            |row| Ok(parse_entry_row(row)?),
        );

        match entry {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get entries for a habit in a date range.
    pub fn get_entries_range(
        &self,
        habit_id: HabitId,
        start: NaiveDate,
        end: NaiveDate,
    ) -> DbResult<Vec<HabitEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM habit_entries WHERE habit_id = ?1 AND date >= ?2 AND date <= ?3 ORDER BY date",
        )?;

        let entries = stmt
            .query_map(
                params![habit_id.to_string(), start.to_string(), end.to_string()],
                |row| Ok(parse_entry_row(row)?),
            )?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(entries)
    }

    /// Get all entries for a date.
    pub fn get_entries_for_date(&self, date: NaiveDate) -> DbResult<Vec<HabitEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM habit_entries WHERE date = ?1",
        )?;

        let entries = stmt
            .query_map(params![date.to_string()], |row| {
                Ok(parse_entry_row(row)?)
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(entries)
    }

    /// Calculate streak for a habit.
    pub fn calculate_streak(&self, habit: &Habit) -> DbResult<StreakInfo> {
        let mut stmt = self.conn.prepare(
            "SELECT date, completed FROM habit_entries WHERE habit_id = ?1 ORDER BY date DESC",
        )?;

        let entries: Vec<(NaiveDate, bool)> = stmt
            .query_map(params![habit.id.to_string()], |row| {
                let date_str: String = row.get(0)?;
                let completed: i32 = row.get(1)?;
                Ok((
                    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap(),
                    completed != 0,
                ))
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        let today = Utc::now().date_naive();
        let mut current_streak = 0u32;
        let mut best_streak = 0u32;
        let mut temp_streak = 0u32;
        let mut last_date = None;

        // Calculate current streak (from today backwards)
        let mut check_date = today;
        for (date, completed) in &entries {
            if !habit.is_due_on(*date) {
                continue;
            }

            // Skip future dates
            if *date > today {
                continue;
            }

            // Check for gaps in scheduled days
            while check_date > *date {
                if habit.is_due_on(check_date) {
                    // Missed a scheduled day
                    break;
                }
                check_date = check_date.pred_opt().unwrap();
            }

            if check_date != *date {
                break; // Gap found
            }

            if *completed {
                current_streak += 1;
                last_date = Some(*date);
                check_date = date.pred_opt().unwrap();
            } else {
                break;
            }
        }

        // Calculate best streak
        for (date, completed) in &entries {
            if !habit.is_due_on(*date) {
                continue;
            }

            if *completed {
                temp_streak += 1;
                best_streak = best_streak.max(temp_streak);
            } else {
                temp_streak = 0;
            }
        }

        best_streak = best_streak.max(current_streak);

        Ok(StreakInfo {
            current: current_streak,
            best: best_streak,
            last_date,
        })
    }

    /// Calculate statistics for a habit.
    pub fn calculate_stats(&self, habit: &Habit) -> DbResult<HabitStats> {
        let streak = self.calculate_streak(habit)?;

        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*), SUM(completed), AVG(value), SUM(value) FROM habit_entries WHERE habit_id = ?1",
        )?;

        let (total, completed, avg_value, total_value): (u32, u32, Option<f64>, Option<f64>) =
            stmt.query_row(params![habit.id.to_string()], |row| {
                Ok((
                    row.get::<_, i32>(0)? as u32,
                    row.get::<_, i32>(1)? as u32,
                    row.get(2)?,
                    row.get(3)?,
                ))
            })?;

        let completion_rate = if total > 0 {
            completed as f64 / total as f64
        } else {
            0.0
        };

        Ok(HabitStats {
            total_entries: total,
            completed_entries: completed,
            completion_rate,
            current_streak: streak.current,
            best_streak: streak.best,
            average_value: avg_value,
            total_value,
        })
    }
}

// Helper functions

fn serialize_schedule(schedule: &Schedule) -> DbResult<(String, Option<String>)> {
    match schedule {
        Schedule::Daily => Ok(("daily".to_string(), None)),
        _ => {
            let data = serde_json::to_string(schedule)?;
            let type_name = match schedule {
                Schedule::Daily => "daily",
                Schedule::Weekly { .. } => "weekly",
                Schedule::Monthly { .. } => "monthly",
                Schedule::Interval { .. } => "interval",
            };
            Ok((type_name.to_string(), Some(data)))
        }
    }
}

fn deserialize_schedule(schedule_type: &str, data: Option<String>) -> Schedule {
    if schedule_type == "daily" {
        return Schedule::Daily;
    }

    data.and_then(|d| serde_json::from_str(&d).ok())
        .unwrap_or(Schedule::Daily)
}

fn serialize_metric(metric: &Metric) -> (String, Option<f64>, Option<String>) {
    match metric {
        Metric::Binary => ("binary".to_string(), None, None),
        Metric::Quantity { goal, unit } => ("quantity".to_string(), Some(*goal), Some(unit.clone())),
    }
}

fn deserialize_metric(metric_type: &str, goal: Option<f64>, unit: Option<String>) -> Metric {
    match metric_type {
        "quantity" => Metric::Quantity {
            goal: goal.unwrap_or(1.0),
            unit: unit.unwrap_or_default(),
        },
        _ => Metric::Binary,
    }
}

fn parse_habit_row(row: &rusqlite::Row) -> SqlResult<Habit> {
    let id_str: String = row.get("id")?;
    let schedule_type: String = row.get("schedule_type")?;
    let schedule_data: Option<String> = row.get("schedule_data")?;
    let metric_type: String = row.get("metric_type")?;
    let metric_goal: Option<f64> = row.get("metric_goal")?;
    let metric_unit: Option<String> = row.get("metric_unit")?;
    let created_str: String = row.get("created_at")?;
    let archived: i32 = row.get("archived")?;

    Ok(Habit {
        id: Uuid::parse_str(&id_str).unwrap(),
        name: row.get("name")?,
        description: row.get("description")?,
        schedule: deserialize_schedule(&schedule_type, schedule_data),
        metric: deserialize_metric(&metric_type, metric_goal, metric_unit),
        color: row.get("color")?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        archived: archived != 0,
    })
}

fn parse_entry_row(row: &rusqlite::Row) -> SqlResult<HabitEntry> {
    let id_str: String = row.get("id")?;
    let habit_id_str: String = row.get("habit_id")?;
    let date_str: String = row.get("date")?;
    let completed: i32 = row.get("completed")?;
    let created_str: String = row.get("created_at")?;

    Ok(HabitEntry {
        id: Uuid::parse_str(&id_str).unwrap(),
        habit_id: Uuid::parse_str(&habit_id_str).unwrap(),
        date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap(),
        completed: completed != 0,
        value: row.get("value")?,
        notes: row.get("notes")?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_list_habits() {
        let db = Database::in_memory().unwrap();

        let habit = Habit::new_binary("Exercise");
        db.insert_habit(&habit).unwrap();

        let habits = db.list_habits(false).unwrap();
        assert_eq!(habits.len(), 1);
        assert_eq!(habits[0].name, "Exercise");
    }

    #[test]
    fn test_entries() {
        let db = Database::in_memory().unwrap();

        let habit = Habit::new_binary("Exercise");
        db.insert_habit(&habit).unwrap();

        let today = Utc::now().date_naive();
        let entry = HabitEntry::new_binary(habit.id, today, true);
        db.upsert_entry(&entry).unwrap();

        let loaded = db.get_entry(habit.id, today).unwrap().unwrap();
        assert!(loaded.completed);
    }
}
