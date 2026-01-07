#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

use crate::board::{Board, Card, ChecklistItem, Column, Comment, Label, Priority};

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
            "CREATE TABLE IF NOT EXISTS boards (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                archived INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS columns (
                id TEXT PRIMARY KEY,
                board_id TEXT NOT NULL,
                name TEXT NOT NULL,
                position INTEGER NOT NULL,
                wip_limit INTEGER,
                color TEXT,
                FOREIGN KEY (board_id) REFERENCES boards(id)
            );

            CREATE TABLE IF NOT EXISTS cards (
                id TEXT PRIMARY KEY,
                column_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                position INTEGER NOT NULL,
                priority TEXT NOT NULL,
                due_date TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                archived INTEGER DEFAULT 0,
                FOREIGN KEY (column_id) REFERENCES columns(id)
            );

            CREATE TABLE IF NOT EXISTS labels (
                id TEXT PRIMARY KEY,
                card_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                FOREIGN KEY (card_id) REFERENCES cards(id)
            );

            CREATE TABLE IF NOT EXISTS checklist_items (
                id TEXT PRIMARY KEY,
                card_id TEXT NOT NULL,
                text TEXT NOT NULL,
                completed INTEGER DEFAULT 0,
                FOREIGN KEY (card_id) REFERENCES cards(id)
            );

            CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                card_id TEXT NOT NULL,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (card_id) REFERENCES cards(id)
            );",
        )?;
        Ok(())
    }

    // Board operations
    pub fn list_boards(&self) -> Result<Vec<Board>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, archived, created_at FROM boards WHERE archived = 0 ORDER BY name",
        )?;
        let boards = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let created_at: String = row.get(4)?;
                Ok(Board {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    archived: row.get::<_, i32>(3)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(boards)
    }

    pub fn save_board(&self, board: &Board) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO boards (id, name, description, archived, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                board.id.to_string(),
                board.name,
                board.description,
                board.archived as i32,
                board.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_board(&self, id: Uuid) -> Result<()> {
        // Delete all cards in all columns first
        self.conn.execute(
            "DELETE FROM checklist_items WHERE card_id IN (SELECT id FROM cards WHERE column_id IN (SELECT id FROM columns WHERE board_id = ?1))",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM labels WHERE card_id IN (SELECT id FROM cards WHERE column_id IN (SELECT id FROM columns WHERE board_id = ?1))",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM comments WHERE card_id IN (SELECT id FROM cards WHERE column_id IN (SELECT id FROM columns WHERE board_id = ?1))",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM cards WHERE column_id IN (SELECT id FROM columns WHERE board_id = ?1)",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM columns WHERE board_id = ?1",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM boards WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // Column operations
    pub fn list_columns(&self, board_id: Uuid) -> Result<Vec<Column>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, board_id, name, position, wip_limit, color FROM columns WHERE board_id = ?1 ORDER BY position",
        )?;
        let columns = stmt
            .query_map([board_id.to_string()], |row| {
                let id: String = row.get(0)?;
                let bid: String = row.get(1)?;
                Ok(Column {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    board_id: Uuid::parse_str(&bid).unwrap_or_default(),
                    name: row.get(2)?,
                    position: row.get(3)?,
                    wip_limit: row.get(4)?,
                    color: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(columns)
    }

    pub fn save_column(&self, column: &Column) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO columns (id, board_id, name, position, wip_limit, color) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                column.id.to_string(),
                column.board_id.to_string(),
                column.name,
                column.position,
                column.wip_limit,
                column.color,
            ],
        )?;
        Ok(())
    }

    pub fn delete_column(&self, id: Uuid) -> Result<()> {
        // Delete all cards in column first
        self.conn.execute(
            "DELETE FROM checklist_items WHERE card_id IN (SELECT id FROM cards WHERE column_id = ?1)",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM labels WHERE card_id IN (SELECT id FROM cards WHERE column_id = ?1)",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM comments WHERE card_id IN (SELECT id FROM cards WHERE column_id = ?1)",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM cards WHERE column_id = ?1",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM columns WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // Card operations
    pub fn list_cards(&self, column_id: Uuid) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, column_id, title, description, position, priority, due_date, created_at, updated_at, archived
             FROM cards WHERE column_id = ?1 AND archived = 0 ORDER BY position",
        )?;
        let cards: Vec<Card> = stmt
            .query_map([column_id.to_string()], |row| {
                let id: String = row.get(0)?;
                let cid: String = row.get(1)?;
                let priority_str: String = row.get(5)?;
                let due_date: Option<String> = row.get(6)?;
                let created_at: String = row.get(7)?;
                let updated_at: String = row.get(8)?;
                Ok(Card {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    column_id: Uuid::parse_str(&cid).unwrap_or_default(),
                    title: row.get(2)?,
                    description: row.get(3)?,
                    position: row.get(4)?,
                    priority: match priority_str.as_str() {
                        "Urgent" => Priority::Urgent,
                        "High" => Priority::High,
                        "Low" => Priority::Low,
                        _ => Priority::Medium,
                    },
                    due_date: due_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
                    labels: Vec::new(),
                    checklist: Vec::new(),
                    comments: Vec::new(),
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    archived: row.get::<_, i32>(9)? != 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Load labels, checklist items and comments for each card
        let mut result = Vec::new();
        for mut card in cards {
            card.labels = self.list_labels(card.id)?;
            card.checklist = self.list_checklist_items(card.id)?;
            card.comments = self.list_comments(card.id)?;
            result.push(card);
        }
        Ok(result)
    }

    pub fn save_card(&self, card: &Card) -> Result<()> {
        let priority_str = match card.priority {
            Priority::Urgent => "Urgent",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        };

        self.conn.execute(
            "INSERT OR REPLACE INTO cards (id, column_id, title, description, position, priority, due_date, created_at, updated_at, archived)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                card.id.to_string(),
                card.column_id.to_string(),
                card.title,
                card.description,
                card.position,
                priority_str,
                card.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                card.created_at.to_rfc3339(),
                card.updated_at.to_rfc3339(),
                card.archived as i32,
            ],
        )?;

        // Save labels
        self.conn.execute("DELETE FROM labels WHERE card_id = ?1", params![card.id.to_string()])?;
        for label in &card.labels {
            self.conn.execute(
                "INSERT INTO labels (id, card_id, name, color) VALUES (?1, ?2, ?3, ?4)",
                params![label.id.to_string(), card.id.to_string(), label.name, label.color],
            )?;
        }

        // Save checklist items
        self.conn.execute("DELETE FROM checklist_items WHERE card_id = ?1", params![card.id.to_string()])?;
        for item in &card.checklist {
            self.conn.execute(
                "INSERT INTO checklist_items (id, card_id, text, completed) VALUES (?1, ?2, ?3, ?4)",
                params![item.id.to_string(), card.id.to_string(), item.text, item.completed as i32],
            )?;
        }

        // Save comments
        self.conn.execute("DELETE FROM comments WHERE card_id = ?1", params![card.id.to_string()])?;
        for comment in &card.comments {
            self.conn.execute(
                "INSERT INTO comments (id, card_id, author, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![comment.id.to_string(), card.id.to_string(), comment.author, comment.content, comment.created_at.to_rfc3339()],
            )?;
        }

        Ok(())
    }

    pub fn delete_card(&self, id: Uuid) -> Result<()> {
        self.conn.execute("DELETE FROM checklist_items WHERE card_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM labels WHERE card_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM comments WHERE card_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM cards WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn list_labels(&self, card_id: Uuid) -> Result<Vec<Label>> {
        let mut stmt = self.conn.prepare("SELECT id, name, color FROM labels WHERE card_id = ?1")?;
        let labels = stmt
            .query_map([card_id.to_string()], |row| {
                let id: String = row.get(0)?;
                Ok(Label {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(labels)
    }

    fn list_checklist_items(&self, card_id: Uuid) -> Result<Vec<ChecklistItem>> {
        let mut stmt = self.conn.prepare("SELECT id, text, completed FROM checklist_items WHERE card_id = ?1")?;
        let items = stmt
            .query_map([card_id.to_string()], |row| {
                let id: String = row.get(0)?;
                Ok(ChecklistItem {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    text: row.get(1)?,
                    completed: row.get::<_, i32>(2)? != 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    fn list_comments(&self, card_id: Uuid) -> Result<Vec<Comment>> {
        let mut stmt = self.conn.prepare("SELECT id, author, content, created_at FROM comments WHERE card_id = ?1 ORDER BY created_at")?;
        let comments = stmt
            .query_map([card_id.to_string()], |row| {
                let id: String = row.get(0)?;
                let created_at: String = row.get(3)?;
                Ok(Comment {
                    id: Uuid::parse_str(&id).unwrap_or_default(),
                    author: row.get(1)?,
                    content: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(comments)
    }
}
