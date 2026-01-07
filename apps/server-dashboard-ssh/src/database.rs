use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::server::{Server, ServerMetrics, ServerStatus};

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
        directories::ProjectDirs::from("", "", "server-dashboard-ssh")
            .map(|p| p.data_dir().join("dashboard.db"))
            .unwrap_or_else(|| PathBuf::from("dashboard.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                user TEXT,
                port INTEGER DEFAULT 22,
                tags TEXT,
                enabled INTEGER DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
                timestamp TEXT NOT NULL,
                cpu_percent REAL,
                memory_used INTEGER,
                memory_total INTEGER,
                disk_used INTEGER,
                disk_total INTEGER,
                load_1 REAL,
                load_5 REAL,
                load_15 REAL,
                status TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_metrics_server_time ON metrics(server_id, timestamp);
            "
        )?;
        Ok(())
    }

    pub fn list_servers(&self) -> Result<Vec<Server>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, host, user, port, tags, enabled, created_at FROM servers ORDER BY name"
        )?;

        let servers = stmt.query_map([], |row| {
            let tags_str: Option<String> = row.get(5)?;
            let enabled: i32 = row.get(6)?;
            let created_at_str: String = row.get(7)?;

            Ok(Server {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                user: row.get(3)?,
                port: row.get(4)?,
                tags: tags_str.map(|s| serde_json::from_str(&s).unwrap_or_default()).unwrap_or_default(),
                enabled: enabled != 0,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(servers)
    }

    pub fn insert_server(&self, server: &Server) -> Result<()> {
        self.conn.execute(
            "INSERT INTO servers (id, name, host, user, port, tags, enabled, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                server.id,
                server.name,
                server.host,
                server.user,
                server.port,
                serde_json::to_string(&server.tags)?,
                server.enabled as i32,
                server.created_at.to_rfc3339(),
            ]
        )?;
        Ok(())
    }

    pub fn update_server(&self, server: &Server) -> Result<()> {
        self.conn.execute(
            "UPDATE servers SET name=?2, host=?3, user=?4, port=?5, tags=?6, enabled=?7 WHERE id=?1",
            params![
                server.id,
                server.name,
                server.host,
                server.user,
                server.port,
                serde_json::to_string(&server.tags)?,
                server.enabled as i32,
            ]
        )?;
        Ok(())
    }

    pub fn delete_server(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM servers WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn insert_metrics(&self, metrics: &ServerMetrics) -> Result<()> {
        self.conn.execute(
            "INSERT INTO metrics (server_id, timestamp, cpu_percent, memory_used, memory_total, disk_used, disk_total, load_1, load_5, load_15, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                metrics.server_id,
                metrics.timestamp.to_rfc3339(),
                metrics.cpu_percent,
                metrics.memory_used,
                metrics.memory_total,
                metrics.disk_used,
                metrics.disk_total,
                metrics.load_1,
                metrics.load_5,
                metrics.load_15,
                metrics.status.label(),
            ]
        )?;
        Ok(())
    }

    pub fn get_latest_metrics(&self, server_id: &str) -> Result<Option<ServerMetrics>> {
        let mut stmt = self.conn.prepare(
            "SELECT server_id, timestamp, cpu_percent, memory_used, memory_total, disk_used, disk_total, load_1, load_5, load_15, status
             FROM metrics WHERE server_id = ?1 ORDER BY timestamp DESC LIMIT 1"
        )?;

        let metrics = stmt.query_map([server_id], |row| {
            let timestamp_str: String = row.get(1)?;
            let status_str: String = row.get(10)?;

            Ok(ServerMetrics {
                server_id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                cpu_percent: row.get(2)?,
                memory_used: row.get(3)?,
                memory_total: row.get(4)?,
                disk_used: row.get(5)?,
                disk_total: row.get(6)?,
                load_1: row.get(7)?,
                load_5: row.get(8)?,
                load_15: row.get(9)?,
                status: match status_str.as_str() {
                    "OK" => ServerStatus::Ok,
                    "WARNING" => ServerStatus::Warning,
                    "CRITICAL" => ServerStatus::Critical,
                    "UNREACHABLE" => ServerStatus::Unreachable,
                    _ => ServerStatus::Unknown,
                },
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(metrics.into_iter().next())
    }

    pub fn cleanup_old_metrics(&self, days: i64) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        self.conn.execute(
            "DELETE FROM metrics WHERE timestamp < ?1",
            params![cutoff.to_rfc3339()]
        )?;
        Ok(())
    }
}
