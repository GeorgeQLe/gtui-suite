//! Database operations for task scheduler.

use crate::models::{RunStatus, Schedule, ScheduledTask, TaskId, TaskRun};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
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
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                schedule TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run TEXT,
                next_run TEXT,
                run_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                status TEXT NOT NULL,
                exit_code INTEGER,
                output TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_runs_task ON runs(task_id);
            "#,
        )
    }

    pub fn insert_task(&self, task: &ScheduledTask) -> SqlResult<TaskId> {
        let schedule_json = serde_json::to_string(&task.schedule).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO tasks (name, command, schedule, enabled, last_run, next_run, run_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                task.name,
                task.command,
                schedule_json,
                task.enabled,
                task.last_run.map(|dt| dt.to_rfc3339()),
                task.next_run.map(|dt| dt.to_rfc3339()),
                task.run_count,
                task.created_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_task(&self, task: &ScheduledTask) -> SqlResult<()> {
        let schedule_json = serde_json::to_string(&task.schedule).unwrap_or_default();
        self.conn.execute(
            "UPDATE tasks SET name = ?1, command = ?2, schedule = ?3, enabled = ?4,
             last_run = ?5, next_run = ?6, run_count = ?7 WHERE id = ?8",
            params![
                task.name,
                task.command,
                schedule_json,
                task.enabled,
                task.last_run.map(|dt| dt.to_rfc3339()),
                task.next_run.map(|dt| dt.to_rfc3339()),
                task.run_count,
                task.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: TaskId) -> SqlResult<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn list_tasks(&self) -> SqlResult<Vec<ScheduledTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, command, schedule, enabled, last_run, next_run, run_count, created_at
             FROM tasks ORDER BY next_run NULLS LAST"
        )?;
        let tasks = stmt.query_map([], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn get_due_tasks(&self) -> SqlResult<Vec<ScheduledTask>> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT id, name, command, schedule, enabled, last_run, next_run, run_count, created_at
             FROM tasks WHERE enabled = 1 AND next_run <= ?1"
        )?;
        let tasks = stmt.query_map([now], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn insert_run(&self, run: &TaskRun) -> SqlResult<i64> {
        self.conn.execute(
            "INSERT INTO runs (task_id, started_at, finished_at, status, exit_code, output)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                run.task_id,
                run.started_at.to_rfc3339(),
                run.finished_at.map(|dt| dt.to_rfc3339()),
                format!("{:?}", run.status),
                run.exit_code,
                run.output,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_task_runs(&self, task_id: TaskId, limit: usize) -> SqlResult<Vec<TaskRun>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, started_at, finished_at, status, exit_code, output
             FROM runs WHERE task_id = ?1 ORDER BY started_at DESC LIMIT ?2"
        )?;
        let runs = stmt.query_map([task_id, limit as i64], |row| self.row_to_run(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(runs)
    }

    fn row_to_task(&self, row: &rusqlite::Row) -> rusqlite::Result<ScheduledTask> {
        let schedule_json: String = row.get(3)?;
        let schedule: Schedule = serde_json::from_str(&schedule_json)
            .unwrap_or(Schedule::Interval { minutes: 60 });

        Ok(ScheduledTask {
            id: row.get(0)?,
            name: row.get(1)?,
            command: row.get(2)?,
            schedule,
            enabled: row.get(4)?,
            last_run: row.get::<_, Option<String>>(5)?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            next_run: row.get::<_, Option<String>>(6)?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            run_count: row.get(7)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn row_to_run(&self, row: &rusqlite::Row) -> rusqlite::Result<TaskRun> {
        let status_str: String = row.get(4)?;
        Ok(TaskRun {
            id: row.get(0)?,
            task_id: row.get(1)?,
            started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            finished_at: row.get::<_, Option<String>>(3)?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            status: match status_str.as_str() {
                "Running" => RunStatus::Running,
                "Success" => RunStatus::Success,
                "Failed" => RunStatus::Failed,
                "Timeout" => RunStatus::Timeout,
                _ => RunStatus::Cancelled,
            },
            exit_code: row.get(5)?,
            output: row.get(6)?,
        })
    }
}
