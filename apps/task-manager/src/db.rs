//! Database operations for task manager.

use crate::models::{Context, ContextId, Priority, Project, ProjectId, Status, Task, TaskId};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::path::Path;

pub type DbResult<T> = SqlResult<T>;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> DbResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                color TEXT,
                archived INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contexts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'Todo',
                priority TEXT NOT NULL DEFAULT 'Low',
                project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
                context_id INTEGER REFERENCES contexts(id) ON DELETE SET NULL,
                tags TEXT NOT NULL DEFAULT '',
                due_date TEXT,
                scheduled_date TEXT,
                recurrence TEXT,
                estimated_mins INTEGER,
                actual_mins INTEGER,
                parent_id INTEGER REFERENCES tasks(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
            CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_parent_id ON tasks(parent_id);
            "#,
        )
    }

    // Projects

    pub fn insert_project(&self, project: &Project) -> DbResult<ProjectId> {
        self.conn.execute(
            "INSERT INTO projects (name, description, color, archived, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                project.name,
                project.description,
                project.color,
                project.archived,
                project.created_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_project(&self, project: &Project) -> DbResult<()> {
        self.conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, color = ?3, archived = ?4 WHERE id = ?5",
            params![
                project.name,
                project.description,
                project.color,
                project.archived,
                project.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: ProjectId) -> DbResult<()> {
        self.conn.execute("DELETE FROM projects WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn list_projects(&self, include_archived: bool) -> DbResult<Vec<Project>> {
        let sql = if include_archived {
            "SELECT id, name, description, color, archived, created_at FROM projects ORDER BY name"
        } else {
            "SELECT id, name, description, color, archived, created_at FROM projects WHERE archived = 0 ORDER BY name"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                archived: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(projects)
    }

    // Contexts

    pub fn insert_context(&self, context: &Context) -> DbResult<ContextId> {
        self.conn.execute(
            "INSERT INTO contexts (name, description) VALUES (?1, ?2)",
            params![context.name, context.description],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_contexts(&self) -> DbResult<Vec<Context>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description FROM contexts ORDER BY name"
        )?;
        let contexts = stmt.query_map([], |row| {
            Ok(Context {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(contexts)
    }

    pub fn delete_context(&self, id: ContextId) -> DbResult<()> {
        self.conn.execute("DELETE FROM contexts WHERE id = ?1", [id])?;
        Ok(())
    }

    // Tasks

    pub fn insert_task(&self, task: &Task) -> DbResult<TaskId> {
        let tags_str = task.tags.join(",");
        let recurrence_json = task.recurrence.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default());

        self.conn.execute(
            "INSERT INTO tasks (uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                task.uuid,
                task.title,
                task.description,
                format!("{:?}", task.status),
                format!("{:?}", task.priority),
                task.project_id,
                task.context_id,
                tags_str,
                task.due_date.map(|d| d.to_string()),
                task.scheduled_date.map(|d| d.to_string()),
                recurrence_json,
                task.estimated_mins,
                task.actual_mins,
                task.parent_id,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.completed_at.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_task(&self, task: &Task) -> DbResult<()> {
        let tags_str = task.tags.join(",");
        let recurrence_json = task.recurrence.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default());

        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, status = ?3, priority = ?4,
             project_id = ?5, context_id = ?6, tags = ?7, due_date = ?8, scheduled_date = ?9,
             recurrence = ?10, estimated_mins = ?11, actual_mins = ?12, parent_id = ?13,
             updated_at = ?14, completed_at = ?15 WHERE id = ?16",
            params![
                task.title,
                task.description,
                format!("{:?}", task.status),
                format!("{:?}", task.priority),
                task.project_id,
                task.context_id,
                tags_str,
                task.due_date.map(|d| d.to_string()),
                task.scheduled_date.map(|d| d.to_string()),
                recurrence_json,
                task.estimated_mins,
                task.actual_mins,
                task.parent_id,
                Utc::now().to_rfc3339(),
                task.completed_at.map(|d| d.to_rfc3339()),
                task.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: TaskId) -> DbResult<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_task(&self, id: TaskId) -> DbResult<Option<Task>> {
        self.conn.query_row(
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks WHERE id = ?1",
            [id],
            |row| self.row_to_task(row),
        ).optional()
    }

    pub fn list_tasks(&self, include_completed: bool) -> DbResult<Vec<Task>> {
        let sql = if include_completed {
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks ORDER BY
             CASE priority WHEN 'Urgent' THEN 0 WHEN 'High' THEN 1 WHEN 'Medium' THEN 2 WHEN 'Low' THEN 3 ELSE 4 END,
             due_date NULLS LAST, created_at"
        } else {
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks WHERE status NOT IN ('Done', 'Cancelled')
             ORDER BY
             CASE priority WHEN 'Urgent' THEN 0 WHEN 'High' THEN 1 WHEN 'Medium' THEN 2 WHEN 'Low' THEN 3 ELSE 4 END,
             due_date NULLS LAST, created_at"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let tasks = stmt.query_map([], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn list_tasks_by_project(&self, project_id: ProjectId) -> DbResult<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks WHERE project_id = ?1
             ORDER BY status, priority DESC, due_date NULLS LAST"
        )?;
        let tasks = stmt.query_map([project_id], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn list_tasks_by_status(&self, status: Status) -> DbResult<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks WHERE status = ?1
             ORDER BY priority DESC, due_date NULLS LAST"
        )?;
        let tasks = stmt.query_map([format!("{:?}", status)], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn list_tasks_due_today(&self) -> DbResult<Vec<Task>> {
        let today = Utc::now().date_naive().to_string();
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks
             WHERE due_date = ?1 AND status NOT IN ('Done', 'Cancelled')
             ORDER BY priority DESC"
        )?;
        let tasks = stmt.query_map([today], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn list_overdue_tasks(&self) -> DbResult<Vec<Task>> {
        let today = Utc::now().date_naive().to_string();
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks
             WHERE due_date < ?1 AND status NOT IN ('Done', 'Cancelled')
             ORDER BY due_date, priority DESC"
        )?;
        let tasks = stmt.query_map([today], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn get_subtasks(&self, parent_id: TaskId) -> DbResult<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, title, description, status, priority, project_id, context_id,
             tags, due_date, scheduled_date, recurrence, estimated_mins, actual_mins, parent_id,
             created_at, updated_at, completed_at FROM tasks WHERE parent_id = ?1
             ORDER BY created_at"
        )?;
        let tasks = stmt.query_map([parent_id], |row| self.row_to_task(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    fn row_to_task(&self, row: &rusqlite::Row) -> rusqlite::Result<Task> {
        let status_str: String = row.get(4)?;
        let priority_str: String = row.get(5)?;
        let tags_str: String = row.get(8)?;
        let due_str: Option<String> = row.get(9)?;
        let sched_str: Option<String> = row.get(10)?;
        let recur_str: Option<String> = row.get(11)?;
        let created_str: String = row.get(15)?;
        let updated_str: String = row.get(16)?;
        let completed_str: Option<String> = row.get(17)?;

        Ok(Task {
            id: row.get(0)?,
            uuid: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: parse_status(&status_str),
            priority: parse_priority(&priority_str),
            project_id: row.get(6)?,
            context_id: row.get(7)?,
            tags: if tags_str.is_empty() {
                Vec::new()
            } else {
                tags_str.split(',').map(|s| s.to_string()).collect()
            },
            due_date: due_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            scheduled_date: sched_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            recurrence: recur_str.and_then(|s| serde_json::from_str(&s).ok()),
            estimated_mins: row.get(12)?,
            actual_mins: row.get(13)?,
            parent_id: row.get(14)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            completed_at: completed_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))
            }),
        })
    }
}

fn parse_status(s: &str) -> Status {
    match s {
        "Todo" => Status::Todo,
        "InProgress" => Status::InProgress,
        "Blocked" => Status::Blocked,
        "Done" => Status::Done,
        "Cancelled" => Status::Cancelled,
        _ => Status::Todo,
    }
}

fn parse_priority(s: &str) -> Priority {
    match s {
        "None" => Priority::None,
        "Low" => Priority::Low,
        "Medium" => Priority::Medium,
        "High" => Priority::High,
        "Urgent" => Priority::Urgent,
        _ => Priority::Low,
    }
}
