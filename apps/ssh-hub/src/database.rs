use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::host::{ConnectionHistory, ForwardType, HostProfile, PortForward, Snippet};

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
        directories::ProjectDirs::from("", "", "ssh-hub")
            .map(|p| p.data_dir().join("ssh-hub.db"))
            .unwrap_or_else(|| PathBuf::from("ssh-hub.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS hosts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                user TEXT,
                port INTEGER,
                identity_file TEXT,
                proxy_jump TEXT,
                notes TEXT,
                last_connected TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS host_tags (
                host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
                tag TEXT NOT NULL,
                PRIMARY KEY (host_id, tag)
            );

            CREATE TABLE IF NOT EXISTS snippets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                host_id TEXT,
                description TEXT
            );

            CREATE TABLE IF NOT EXISTS connection_history (
                id TEXT PRIMARY KEY,
                host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
                connected_at TEXT NOT NULL,
                disconnected_at TEXT,
                duration_secs INTEGER
            );

            CREATE TABLE IF NOT EXISTS port_forwards (
                id TEXT PRIMARY KEY,
                host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
                forward_type TEXT NOT NULL,
                local_port INTEGER NOT NULL,
                remote_host TEXT,
                remote_port INTEGER NOT NULL,
                active INTEGER DEFAULT 0
            );
            "
        )?;
        Ok(())
    }

    // Host operations
    pub fn list_hosts(&self) -> Result<Vec<HostProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, host, user, port, identity_file, proxy_jump, notes, last_connected, created_at FROM hosts ORDER BY name"
        )?;

        let hosts = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let identity_file: Option<String> = row.get(5)?;
            let last_connected: Option<String> = row.get(8)?;
            let created_at: String = row.get(9)?;

            Ok(HostProfile {
                id,
                name: row.get(1)?,
                host: row.get(2)?,
                user: row.get(3)?,
                port: row.get(4)?,
                identity_file: identity_file.map(PathBuf::from),
                proxy_jump: row.get(6)?,
                tags: Vec::new(), // Load separately
                notes: row.get(7)?,
                last_connected: last_connected.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                created_at: DateTime::parse_from_rfc3339(&created_at).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        // Load tags for each host
        let mut hosts_with_tags = Vec::new();
        for mut host in hosts {
            host.tags = self.get_host_tags(&host.id)?;
            hosts_with_tags.push(host);
        }

        Ok(hosts_with_tags)
    }

    fn get_host_tags(&self, host_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT tag FROM host_tags WHERE host_id = ?")?;
        let tags = stmt.query_map([host_id], |row| row.get(0))?.collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn insert_host(&self, host: &HostProfile) -> Result<()> {
        self.conn.execute(
            "INSERT INTO hosts (id, name, host, user, port, identity_file, proxy_jump, notes, last_connected, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                host.id,
                host.name,
                host.host,
                host.user,
                host.port,
                host.identity_file.as_ref().map(|p| p.display().to_string()),
                host.proxy_jump,
                host.notes,
                host.last_connected.map(|d| d.to_rfc3339()),
                host.created_at.to_rfc3339(),
            ]
        )?;

        for tag in &host.tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO host_tags (host_id, tag) VALUES (?1, ?2)",
                params![host.id, tag]
            )?;
        }

        Ok(())
    }

    pub fn update_host(&self, host: &HostProfile) -> Result<()> {
        self.conn.execute(
            "UPDATE hosts SET name=?2, host=?3, user=?4, port=?5, identity_file=?6, proxy_jump=?7, notes=?8, last_connected=?9 WHERE id=?1",
            params![
                host.id,
                host.name,
                host.host,
                host.user,
                host.port,
                host.identity_file.as_ref().map(|p| p.display().to_string()),
                host.proxy_jump,
                host.notes,
                host.last_connected.map(|d| d.to_rfc3339()),
            ]
        )?;

        // Update tags
        self.conn.execute("DELETE FROM host_tags WHERE host_id = ?1", params![host.id])?;
        for tag in &host.tags {
            self.conn.execute(
                "INSERT INTO host_tags (host_id, tag) VALUES (?1, ?2)",
                params![host.id, tag]
            )?;
        }

        Ok(())
    }

    pub fn delete_host(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM hosts WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_last_connected(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE hosts SET last_connected = ?2 WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()]
        )?;
        Ok(())
    }

    // Snippet operations
    pub fn list_snippets(&self, host_id: Option<&str>) -> Result<Vec<Snippet>> {
        let mut stmt = if host_id.is_some() {
            self.conn.prepare("SELECT id, name, command, host_id, description FROM snippets WHERE host_id = ?1 OR host_id IS NULL ORDER BY name")?
        } else {
            self.conn.prepare("SELECT id, name, command, host_id, description FROM snippets ORDER BY name")?
        };

        let snippets = if let Some(hid) = host_id {
            stmt.query_map([hid], |row| {
                Ok(Snippet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    command: row.get(2)?,
                    host_id: row.get(3)?,
                    description: row.get(4)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(Snippet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    command: row.get(2)?,
                    host_id: row.get(3)?,
                    description: row.get(4)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?
        };

        Ok(snippets)
    }

    pub fn insert_snippet(&self, snippet: &Snippet) -> Result<()> {
        self.conn.execute(
            "INSERT INTO snippets (id, name, command, host_id, description) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![snippet.id, snippet.name, snippet.command, snippet.host_id, snippet.description]
        )?;
        Ok(())
    }

    pub fn delete_snippet(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Connection history
    pub fn add_connection(&self, history: &ConnectionHistory) -> Result<()> {
        self.conn.execute(
            "INSERT INTO connection_history (id, host_id, connected_at, disconnected_at, duration_secs) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                history.id,
                history.host_id,
                history.connected_at.to_rfc3339(),
                history.disconnected_at.map(|d| d.to_rfc3339()),
                history.duration_secs,
            ]
        )?;
        Ok(())
    }

    pub fn get_recent_connections(&self, limit: usize) -> Result<Vec<ConnectionHistory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, host_id, connected_at, disconnected_at, duration_secs FROM connection_history ORDER BY connected_at DESC LIMIT ?1"
        )?;

        let history = stmt.query_map([limit], |row| {
            let connected_at: String = row.get(2)?;
            let disconnected_at: Option<String> = row.get(3)?;

            Ok(ConnectionHistory {
                id: row.get(0)?,
                host_id: row.get(1)?,
                connected_at: DateTime::parse_from_rfc3339(&connected_at).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                disconnected_at: disconnected_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                duration_secs: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(history)
    }

    // Port forwards
    pub fn list_port_forwards(&self, host_id: &str) -> Result<Vec<PortForward>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, host_id, forward_type, local_port, remote_host, remote_port, active FROM port_forwards WHERE host_id = ?1"
        )?;

        let forwards = stmt.query_map([host_id], |row| {
            let forward_type: String = row.get(2)?;
            let active: i32 = row.get(6)?;

            Ok(PortForward {
                id: row.get(0)?,
                host_id: row.get(1)?,
                forward_type: match forward_type.as_str() {
                    "Remote" => ForwardType::Remote,
                    "Dynamic" => ForwardType::Dynamic,
                    _ => ForwardType::Local,
                },
                local_port: row.get(3)?,
                remote_host: row.get(4)?,
                remote_port: row.get(5)?,
                active: active != 0,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(forwards)
    }

    pub fn insert_port_forward(&self, forward: &PortForward) -> Result<()> {
        let forward_type = match forward.forward_type {
            ForwardType::Local => "Local",
            ForwardType::Remote => "Remote",
            ForwardType::Dynamic => "Dynamic",
        };

        self.conn.execute(
            "INSERT INTO port_forwards (id, host_id, forward_type, local_port, remote_host, remote_port, active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                forward.id,
                forward.host_id,
                forward_type,
                forward.local_port,
                forward.remote_host,
                forward.remote_port,
                forward.active as i32,
            ]
        )?;
        Ok(())
    }

    pub fn delete_port_forward(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM port_forwards WHERE id = ?1", params![id])?;
        Ok(())
    }
}
