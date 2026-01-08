#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

use crate::request::{Collection, HistoryEntry, Method, SavedRequest};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                requests TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS requests (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                headers TEXT NOT NULL,
                body TEXT,
                auth TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                request_id TEXT,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp);",
        )?;
        Ok(())
    }

    // Collections
    pub fn list_collections(&self) -> Result<Vec<Collection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, requests, created_at FROM collections ORDER BY name",
        )?;
        let collections = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let requests_json: String = row.get(2)?;
                let created_at: String = row.get(3)?;
                Ok(Collection {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    name: row.get(1)?,
                    requests: serde_json::from_str(&requests_json).unwrap_or_default(),
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(collections)
    }

    pub fn save_collection(&self, collection: &Collection) -> Result<()> {
        let requests_json = serde_json::to_string(&collection.requests)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO collections (id, name, requests, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                collection.id.to_string(),
                collection.name,
                requests_json,
                collection.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_collection(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM collections WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // Requests
    pub fn list_requests(&self) -> Result<Vec<SavedRequest>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, method, url, headers, body, auth, created_at, updated_at FROM requests ORDER BY name",
        )?;
        let requests = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let method_str: String = row.get(2)?;
                let headers_json: String = row.get(4)?;
                let auth_json: String = row.get(6)?;
                let created_at: String = row.get(7)?;
                let updated_at: String = row.get(8)?;
                Ok(SavedRequest {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    name: row.get(1)?,
                    method: match method_str.as_str() {
                        "POST" => Method::POST,
                        "PUT" => Method::PUT,
                        "PATCH" => Method::PATCH,
                        "DELETE" => Method::DELETE,
                        "HEAD" => Method::HEAD,
                        "OPTIONS" => Method::OPTIONS,
                        _ => Method::GET,
                    },
                    url: row.get(3)?,
                    headers: serde_json::from_str(&headers_json).unwrap_or_default(),
                    body: row.get(5)?,
                    auth: serde_json::from_str(&auth_json).unwrap_or_default(),
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(requests)
    }

    pub fn get_request(&self, id: Uuid) -> Result<Option<SavedRequest>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, method, url, headers, body, auth, created_at, updated_at FROM requests WHERE id = ?1",
        )?;
        let request = stmt
            .query_row([id.to_string()], |row| {
                let id: String = row.get(0)?;
                let method_str: String = row.get(2)?;
                let headers_json: String = row.get(4)?;
                let auth_json: String = row.get(6)?;
                let created_at: String = row.get(7)?;
                let updated_at: String = row.get(8)?;
                Ok(SavedRequest {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    name: row.get(1)?,
                    method: match method_str.as_str() {
                        "POST" => Method::POST,
                        "PUT" => Method::PUT,
                        "PATCH" => Method::PATCH,
                        "DELETE" => Method::DELETE,
                        "HEAD" => Method::HEAD,
                        "OPTIONS" => Method::OPTIONS,
                        _ => Method::GET,
                    },
                    url: row.get(3)?,
                    headers: serde_json::from_str(&headers_json).unwrap_or_default(),
                    body: row.get(5)?,
                    auth: serde_json::from_str(&auth_json).unwrap_or_default(),
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })
            .ok();
        Ok(request)
    }

    pub fn save_request(&self, request: &SavedRequest) -> Result<()> {
        let headers_json = serde_json::to_string(&request.headers)?;
        let auth_json = serde_json::to_string(&request.auth)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO requests (id, name, method, url, headers, body, auth, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                request.id.to_string(),
                request.name,
                request.method.as_str(),
                request.url,
                headers_json,
                request.body,
                auth_json,
                request.created_at.to_rfc3339(),
                request.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_request(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM requests WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // History
    pub fn list_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, request_id, method, url, status, duration_ms, timestamp FROM history ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let history = stmt
            .query_map([limit as i64], |row| {
                let id: String = row.get(0)?;
                let request_id: Option<String> = row.get(1)?;
                let method_str: String = row.get(2)?;
                let timestamp: String = row.get(6)?;
                Ok(HistoryEntry {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    request_id: request_id.and_then(|r| Uuid::parse_str(&r).ok()),
                    method: match method_str.as_str() {
                        "POST" => Method::POST,
                        "PUT" => Method::PUT,
                        "PATCH" => Method::PATCH,
                        "DELETE" => Method::DELETE,
                        "HEAD" => Method::HEAD,
                        "OPTIONS" => Method::OPTIONS,
                        _ => Method::GET,
                    },
                    url: row.get(3)?,
                    status: row.get(4)?,
                    duration_ms: row.get(5)?,
                    timestamp: DateTime::parse_from_rfc3339(&timestamp)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(history)
    }

    pub fn add_history(&self, entry: &HistoryEntry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history (id, request_id, method, url, status, duration_ms, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.id.to_string(),
                entry.request_id.map(|r| r.to_string()),
                entry.method.as_str(),
                entry.url,
                entry.status,
                entry.duration_ms,
                entry.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
}
