//! Database operations for daily notes.

use crate::models::{DailyEntry, EntryId, JournalStats, MonthCalendar};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL DEFAULT '',
                word_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date);
            "#,
        )
    }

    pub fn get_or_create_entry(&self, date: NaiveDate) -> SqlResult<DailyEntry> {
        if let Some(entry) = self.get_entry_by_date(date)? {
            return Ok(entry);
        }

        let entry = DailyEntry::new(date);
        let id = self.insert_entry(&entry)?;
        Ok(DailyEntry { id, ..entry })
    }

    pub fn insert_entry(&self, entry: &DailyEntry) -> SqlResult<EntryId> {
        self.conn.execute(
            "INSERT INTO entries (date, content, word_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.date.to_string(),
                entry.content,
                entry.word_count,
                entry.created_at.to_rfc3339(),
                entry.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_entry(&self, entry: &DailyEntry) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE entries SET content = ?1, word_count = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                entry.content,
                entry.word_count,
                Utc::now().to_rfc3339(),
                entry.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_entry(&self, id: EntryId) -> SqlResult<()> {
        self.conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_entry_by_date(&self, date: NaiveDate) -> SqlResult<Option<DailyEntry>> {
        self.conn.query_row(
            "SELECT id, date, content, word_count, created_at, updated_at
             FROM entries WHERE date = ?1",
            [date.to_string()],
            |row| self.row_to_entry(row),
        ).optional()
    }

    pub fn list_entries(&self, limit: usize) -> SqlResult<Vec<DailyEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, content, word_count, created_at, updated_at
             FROM entries ORDER BY date DESC LIMIT ?1"
        )?;
        let entries = stmt.query_map([limit], |row| self.row_to_entry(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    pub fn list_entries_for_month(&self, year: i32, month: u32) -> SqlResult<Vec<DailyEntry>> {
        let start = format!("{:04}-{:02}-01", year, month);
        let end = format!("{:04}-{:02}-31", year, month);

        let mut stmt = self.conn.prepare(
            "SELECT id, date, content, word_count, created_at, updated_at
             FROM entries WHERE date >= ?1 AND date <= ?2 ORDER BY date"
        )?;
        let entries = stmt.query_map([&start, &end], |row| self.row_to_entry(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    pub fn get_entries_with_content(&self, year: i32, month: u32) -> SqlResult<Vec<NaiveDate>> {
        let start = format!("{:04}-{:02}-01", year, month);
        let end = format!("{:04}-{:02}-31", year, month);

        let mut stmt = self.conn.prepare(
            "SELECT date FROM entries WHERE date >= ?1 AND date <= ?2 AND content != ''"
        )?;
        let dates = stmt.query_map([&start, &end], |row| {
            let date_str: String = row.get(0)?;
            Ok(NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap())
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(dates)
    }

    pub fn populate_calendar(&self, calendar: &mut MonthCalendar) -> SqlResult<()> {
        let entries = self.list_entries_for_month(calendar.year, calendar.month)?;

        for entry in entries {
            if let Some(day) = calendar.days.iter_mut().find(|d| d.date == entry.date) {
                day.has_entry = !entry.content.is_empty();
                day.word_count = entry.word_count;
            }
        }

        Ok(())
    }

    pub fn get_stats(&self) -> SqlResult<JournalStats> {
        let total_entries: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE content != ''",
            [],
            |row| row.get(0),
        )?;

        let total_words: usize = self.conn.query_row(
            "SELECT COALESCE(SUM(word_count), 0) FROM entries",
            [],
            |row| row.get(0),
        )?;

        let avg_words_per_entry = if total_entries > 0 {
            total_words / total_entries
        } else {
            0
        };

        // Calculate streaks
        let (current_streak, longest_streak) = self.calculate_streaks()?;

        Ok(JournalStats {
            total_entries,
            total_words,
            current_streak,
            longest_streak,
            avg_words_per_entry,
        })
    }

    fn calculate_streaks(&self) -> SqlResult<(usize, usize)> {
        let mut stmt = self.conn.prepare(
            "SELECT date FROM entries WHERE content != '' ORDER BY date DESC"
        )?;
        let dates: Vec<NaiveDate> = stmt.query_map([], |row| {
            let date_str: String = row.get(0)?;
            Ok(NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap())
        })?
        .collect::<Result<Vec<_>, _>>()?;

        if dates.is_empty() {
            return Ok((0, 0));
        }

        let today = Utc::now().date_naive();
        let mut current_streak = 0;
        let mut longest_streak = 0;
        let mut streak = 0;
        let mut prev_date: Option<NaiveDate> = None;

        for date in &dates {
            if let Some(prev) = prev_date {
                let diff = (prev - *date).num_days();
                if diff == 1 {
                    streak += 1;
                } else {
                    longest_streak = longest_streak.max(streak);
                    streak = 1;
                }
            } else {
                streak = 1;
                // Check if current streak includes today or yesterday
                let days_ago = (today - *date).num_days();
                if days_ago <= 1 {
                    current_streak = streak;
                }
            }
            prev_date = Some(*date);
        }

        // Update current streak
        if !dates.is_empty() {
            let days_ago = (today - dates[0]).num_days();
            if days_ago <= 1 {
                current_streak = streak;
            }
        }

        longest_streak = longest_streak.max(streak);

        Ok((current_streak, longest_streak))
    }

    pub fn search(&self, query: &str) -> SqlResult<Vec<DailyEntry>> {
        let search = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, date, content, word_count, created_at, updated_at
             FROM entries WHERE content LIKE ?1 ORDER BY date DESC LIMIT 50"
        )?;
        let entries = stmt.query_map([&search], |row| self.row_to_entry(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    fn row_to_entry(&self, row: &rusqlite::Row) -> rusqlite::Result<DailyEntry> {
        let date_str: String = row.get(1)?;
        Ok(DailyEntry {
            id: row.get(0)?,
            date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap(),
            content: row.get(2)?,
            word_count: row.get(3)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}
