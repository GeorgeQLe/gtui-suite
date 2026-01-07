use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::profile::{BackendType, BackupProfile, BackupRun, RunStatus};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let path = Self::db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn db_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "backup-manager")
            .map(|p| p.data_dir().join("backup-manager.db"))
            .unwrap_or_else(|| PathBuf::from("backup-manager.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                backend TEXT NOT NULL,
                source_paths TEXT NOT NULL,
                destination TEXT NOT NULL,
                excludes TEXT,
                schedule TEXT,
                retention TEXT,
                pre_hooks TEXT,
                post_hooks TEXT,
                enabled INTEGER DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS backup_runs (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                status TEXT NOT NULL,
                bytes_transferred INTEGER,
                files_transferred INTEGER,
                error_message TEXT
            );

            CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                size INTEGER,
                paths TEXT
            );
            "
        )?;
        Ok(())
    }

    // Profile operations
    pub fn list_profiles(&self) -> Result<Vec<BackupProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, backend, source_paths, destination, excludes, schedule, retention, pre_hooks, post_hooks, enabled, created_at FROM profiles ORDER BY name"
        )?;

        let profiles = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let backend_str: String = row.get(2)?;
            let source_paths_str: String = row.get(3)?;
            let excludes_str: Option<String> = row.get(5)?;
            let retention_str: Option<String> = row.get(7)?;
            let pre_hooks_str: Option<String> = row.get(8)?;
            let post_hooks_str: Option<String> = row.get(9)?;
            let enabled: i32 = row.get(10)?;
            let created_at_str: String = row.get(11)?;

            Ok(BackupProfile {
                id,
                name: row.get(1)?,
                backend: BackendType::from_str(&backend_str).unwrap_or(BackendType::Rsync),
                source_paths: serde_json::from_str(&source_paths_str).unwrap_or_default(),
                destination: row.get(4)?,
                excludes: excludes_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
                schedule: row.get(6)?,
                retention: retention_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
                pre_hooks: pre_hooks_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
                post_hooks: post_hooks_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
                enabled: enabled != 0,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }

    pub fn insert_profile(&self, profile: &BackupProfile) -> Result<()> {
        self.conn.execute(
            "INSERT INTO profiles (id, name, backend, source_paths, destination, excludes, schedule, retention, pre_hooks, post_hooks, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                profile.id,
                profile.name,
                profile.backend.label(),
                serde_json::to_string(&profile.source_paths)?,
                profile.destination,
                serde_json::to_string(&profile.excludes)?,
                profile.schedule,
                serde_json::to_string(&profile.retention)?,
                serde_json::to_string(&profile.pre_hooks)?,
                serde_json::to_string(&profile.post_hooks)?,
                profile.enabled as i32,
                profile.created_at.to_rfc3339(),
            ]
        )?;
        Ok(())
    }

    pub fn update_profile(&self, profile: &BackupProfile) -> Result<()> {
        self.conn.execute(
            "UPDATE profiles SET name=?2, backend=?3, source_paths=?4, destination=?5, excludes=?6, schedule=?7, retention=?8, pre_hooks=?9, post_hooks=?10, enabled=?11 WHERE id=?1",
            params![
                profile.id,
                profile.name,
                profile.backend.label(),
                serde_json::to_string(&profile.source_paths)?,
                profile.destination,
                serde_json::to_string(&profile.excludes)?,
                profile.schedule,
                serde_json::to_string(&profile.retention)?,
                serde_json::to_string(&profile.pre_hooks)?,
                serde_json::to_string(&profile.post_hooks)?,
                profile.enabled as i32,
            ]
        )?;
        Ok(())
    }

    pub fn delete_profile(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM profiles WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn toggle_profile(&self, id: &str, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE profiles SET enabled = ?2 WHERE id = ?1",
            params![id, enabled as i32]
        )?;
        Ok(())
    }

    // Backup run operations
    pub fn insert_run(&self, run: &BackupRun) -> Result<()> {
        self.conn.execute(
            "INSERT INTO backup_runs (id, profile_id, started_at, finished_at, status, bytes_transferred, files_transferred, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                run.id,
                run.profile_id,
                run.started_at.to_rfc3339(),
                run.finished_at.map(|d| d.to_rfc3339()),
                run.status.label(),
                run.bytes_transferred,
                run.files_transferred,
                run.error_message,
            ]
        )?;
        Ok(())
    }

    pub fn update_run(&self, run: &BackupRun) -> Result<()> {
        self.conn.execute(
            "UPDATE backup_runs SET finished_at=?2, status=?3, bytes_transferred=?4, files_transferred=?5, error_message=?6 WHERE id=?1",
            params![
                run.id,
                run.finished_at.map(|d| d.to_rfc3339()),
                run.status.label(),
                run.bytes_transferred,
                run.files_transferred,
                run.error_message,
            ]
        )?;
        Ok(())
    }

    pub fn get_recent_runs(&self, profile_id: &str, limit: usize) -> Result<Vec<BackupRun>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, profile_id, started_at, finished_at, status, bytes_transferred, files_transferred, error_message
             FROM backup_runs WHERE profile_id = ?1 ORDER BY started_at DESC LIMIT ?2"
        )?;

        let runs = stmt.query_map(params![profile_id, limit], |row| {
            let started_at_str: String = row.get(2)?;
            let finished_at_str: Option<String> = row.get(3)?;
            let status_str: String = row.get(4)?;

            Ok(BackupRun {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                started_at: DateTime::parse_from_rfc3339(&started_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                finished_at: finished_at_str.and_then(|s|
                    DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
                ),
                status: match status_str.as_str() {
                    "Success" => RunStatus::Success,
                    "Failed" => RunStatus::Failed,
                    "Cancelled" => RunStatus::Cancelled,
                    _ => RunStatus::Running,
                },
                bytes_transferred: row.get(5)?,
                files_transferred: row.get(6)?,
                error_message: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    pub fn get_last_run(&self, profile_id: &str) -> Result<Option<BackupRun>> {
        let runs = self.get_recent_runs(profile_id, 1)?;
        Ok(runs.into_iter().next())
    }
}
