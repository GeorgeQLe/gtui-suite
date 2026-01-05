//! Database operations for time tracking.

use crate::models::{Client, DailySummary, Project, ProjectId, TimeEntry, EntryId};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type DbResult<T> = Result<T, DbError>;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    pub fn in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> DbResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS clients (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                hourly_rate REAL,
                currency TEXT DEFAULT 'USD',
                archived INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                client_id TEXT REFERENCES clients(id),
                name TEXT NOT NULL,
                color TEXT,
                budget_hours REAL,
                archived INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS time_entries (
                id TEXT PRIMARY KEY,
                project_id TEXT REFERENCES projects(id),
                description TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                duration_secs INTEGER,
                billable INTEGER DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entry_tags (
                entry_id TEXT NOT NULL REFERENCES time_entries(id),
                tag TEXT NOT NULL,
                PRIMARY KEY (entry_id, tag)
            );

            CREATE INDEX IF NOT EXISTS idx_entries_start ON time_entries(start_time);
            CREATE INDEX IF NOT EXISTS idx_entries_project ON time_entries(project_id);
            "#,
        )?;
        Ok(())
    }

    // Client operations

    pub fn insert_client(&self, client: &Client) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO clients (id, name, hourly_rate, currency, archived, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                client.id.to_string(),
                client.name,
                client.hourly_rate,
                client.currency,
                client.archived as i32,
                client.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_clients(&self, include_archived: bool) -> DbResult<Vec<Client>> {
        let sql = if include_archived {
            "SELECT * FROM clients ORDER BY name"
        } else {
            "SELECT * FROM clients WHERE archived = 0 ORDER BY name"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let clients = stmt
            .query_map([], |row| Ok(parse_client_row(row)?))?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(clients)
    }

    // Project operations

    pub fn insert_project(&self, project: &Project) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO projects (id, client_id, name, color, budget_hours, archived, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                project.id.to_string(),
                project.client_id.map(|id| id.to_string()),
                project.name,
                project.color,
                project.budget_hours,
                project.archived as i32,
                project.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_project(&self, id: ProjectId) -> DbResult<Option<Project>> {
        let mut stmt = self.conn.prepare("SELECT * FROM projects WHERE id = ?1")?;
        let project = stmt.query_row(params![id.to_string()], |row| Ok(parse_project_row(row)?));
        match project {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_projects(&self, include_archived: bool) -> DbResult<Vec<Project>> {
        let sql = if include_archived {
            "SELECT * FROM projects ORDER BY name"
        } else {
            "SELECT * FROM projects WHERE archived = 0 ORDER BY name"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let projects = stmt
            .query_map([], |row| Ok(parse_project_row(row)?))?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(projects)
    }

    // Time entry operations

    pub fn insert_entry(&self, entry: &TimeEntry) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO time_entries (id, project_id, description, start_time, end_time, duration_secs, billable, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id.to_string(),
                entry.project_id.map(|id| id.to_string()),
                entry.description,
                entry.start_time.to_rfc3339(),
                entry.end_time.map(|t| t.to_rfc3339()),
                entry.duration_secs,
                entry.billable as i32,
                entry.created_at.to_rfc3339(),
            ],
        )?;

        for tag in &entry.tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO entry_tags (entry_id, tag) VALUES (?1, ?2)",
                params![entry.id.to_string(), tag],
            )?;
        }

        Ok(())
    }

    pub fn update_entry(&self, entry: &TimeEntry) -> DbResult<()> {
        self.conn.execute(
            "UPDATE time_entries SET project_id = ?2, description = ?3, start_time = ?4, end_time = ?5, duration_secs = ?6, billable = ?7 WHERE id = ?1",
            params![
                entry.id.to_string(),
                entry.project_id.map(|id| id.to_string()),
                entry.description,
                entry.start_time.to_rfc3339(),
                entry.end_time.map(|t| t.to_rfc3339()),
                entry.duration_secs,
                entry.billable as i32,
            ],
        )?;
        Ok(())
    }

    pub fn delete_entry(&self, id: EntryId) -> DbResult<()> {
        self.conn.execute("DELETE FROM entry_tags WHERE entry_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM time_entries WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn get_entry(&self, id: EntryId) -> DbResult<Option<TimeEntry>> {
        let mut stmt = self.conn.prepare("SELECT * FROM time_entries WHERE id = ?1")?;
        let entry = stmt.query_row(params![id.to_string()], |row| Ok(parse_entry_row(row)?));
        match entry {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_running_entry(&self) -> DbResult<Option<TimeEntry>> {
        let mut stmt = self.conn.prepare("SELECT * FROM time_entries WHERE end_time IS NULL ORDER BY start_time DESC LIMIT 1")?;
        let entry = stmt.query_row([], |row| Ok(parse_entry_row(row)?));
        match entry {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_entries_for_date(&self, date: NaiveDate) -> DbResult<Vec<TimeEntry>> {
        let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        let mut stmt = self.conn.prepare(
            "SELECT * FROM time_entries WHERE start_time >= ?1 AND start_time <= ?2 ORDER BY start_time DESC"
        )?;

        let entries = stmt
            .query_map(params![start.to_rfc3339(), end.to_rfc3339()], |row| {
                Ok(parse_entry_row(row)?)
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(entries)
    }

    pub fn get_entries_range(&self, start: NaiveDate, end: NaiveDate) -> DbResult<Vec<TimeEntry>> {
        let start_dt = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_dt = end.and_hms_opt(23, 59, 59).unwrap().and_utc();

        let mut stmt = self.conn.prepare(
            "SELECT * FROM time_entries WHERE start_time >= ?1 AND start_time <= ?2 ORDER BY start_time"
        )?;

        let entries = stmt
            .query_map(params![start_dt.to_rfc3339(), end_dt.to_rfc3339()], |row| {
                Ok(parse_entry_row(row)?)
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(entries)
    }

    pub fn get_recent_entries(&self, limit: usize) -> DbResult<Vec<TimeEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM time_entries WHERE end_time IS NOT NULL ORDER BY start_time DESC LIMIT ?1"
        )?;

        let entries = stmt
            .query_map(params![limit as i64], |row| Ok(parse_entry_row(row)?))?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(entries)
    }

    pub fn get_daily_summary(&self, date: NaiveDate) -> DbResult<DailySummary> {
        let entries = self.get_entries_for_date(date)?;
        let mut summary = DailySummary::new(date);
        for entry in entries {
            summary.add_entry(entry);
        }
        Ok(summary)
    }

    pub fn get_project_hours(&self, project_id: ProjectId) -> DbResult<f64> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM time_entries WHERE project_id = ?1"
        )?;
        let secs: i64 = stmt.query_row(params![project_id.to_string()], |row| row.get(0))?;
        Ok(secs as f64 / 3600.0)
    }
}

fn parse_client_row(row: &rusqlite::Row) -> SqlResult<Client> {
    let id_str: String = row.get("id")?;
    let created_str: String = row.get("created_at")?;
    let archived: i32 = row.get("archived")?;

    Ok(Client {
        id: Uuid::parse_str(&id_str).unwrap(),
        name: row.get("name")?,
        hourly_rate: row.get("hourly_rate")?,
        currency: row.get("currency")?,
        archived: archived != 0,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

fn parse_project_row(row: &rusqlite::Row) -> SqlResult<Project> {
    let id_str: String = row.get("id")?;
    let client_id_str: Option<String> = row.get("client_id")?;
    let created_str: String = row.get("created_at")?;
    let archived: i32 = row.get("archived")?;

    Ok(Project {
        id: Uuid::parse_str(&id_str).unwrap(),
        client_id: client_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
        name: row.get("name")?,
        color: row.get("color")?,
        budget_hours: row.get("budget_hours")?,
        archived: archived != 0,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

fn parse_entry_row(row: &rusqlite::Row) -> SqlResult<TimeEntry> {
    let id_str: String = row.get("id")?;
    let project_id_str: Option<String> = row.get("project_id")?;
    let start_str: String = row.get("start_time")?;
    let end_str: Option<String> = row.get("end_time")?;
    let created_str: String = row.get("created_at")?;
    let billable: i32 = row.get("billable")?;

    Ok(TimeEntry {
        id: Uuid::parse_str(&id_str).unwrap(),
        project_id: project_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
        description: row.get("description")?,
        start_time: DateTime::parse_from_rfc3339(&start_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        end_time: end_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        duration_secs: row.get("duration_secs")?,
        tags: Vec::new(),
        billable: billable != 0,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_crud() {
        let db = Database::in_memory().unwrap();
        let project = Project::new("Test Project");
        db.insert_project(&project).unwrap();

        let loaded = db.get_project(project.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Test Project");
    }

    #[test]
    fn test_entry_crud() {
        let db = Database::in_memory().unwrap();
        let mut entry = TimeEntry::start("Working");
        entry.stop();
        db.insert_entry(&entry).unwrap();

        let loaded = db.get_entry(entry.id).unwrap().unwrap();
        assert_eq!(loaded.description, "Working");
    }
}
