//! Database operations for backlink notes.

use crate::models::{Note, NoteId, SearchResult};
use chrono::{DateTime, Utc};
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
            CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                target_title TEXT NOT NULL,
                target_id INTEGER REFERENCES notes(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
            CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_id);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_id);
            CREATE INDEX IF NOT EXISTS idx_links_target_title ON links(target_title);

            CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
                title, content, content='notes', content_rowid='id'
            );

            CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
                INSERT INTO notes_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
                INSERT INTO notes_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
            END;
            "#,
        )
    }

    pub fn insert_note(&self, note: &Note) -> SqlResult<NoteId> {
        self.conn.execute(
            "INSERT INTO notes (title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                note.title,
                note.content,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        self.update_links(id, note)?;
        Ok(id)
    }

    pub fn update_note(&self, note: &Note) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                note.title,
                note.content,
                Utc::now().to_rfc3339(),
                note.id,
            ],
        )?;
        self.update_links(note.id, note)?;
        Ok(())
    }

    fn update_links(&self, note_id: NoteId, note: &Note) -> SqlResult<()> {
        // Delete existing links from this note
        self.conn.execute("DELETE FROM links WHERE source_id = ?1", [note_id])?;

        // Extract and insert new links
        let links = note.extract_links();
        for target_title in links {
            // Check if target note exists
            let target_id: Option<NoteId> = self.conn.query_row(
                "SELECT id FROM notes WHERE title = ?1",
                [&target_title],
                |row| row.get(0),
            ).optional()?;

            self.conn.execute(
                "INSERT INTO links (source_id, target_title, target_id) VALUES (?1, ?2, ?3)",
                params![note_id, target_title, target_id],
            )?;
        }

        // Update links that point to this note's title
        self.conn.execute(
            "UPDATE links SET target_id = ?1 WHERE target_title = ?2 AND target_id IS NULL",
            params![note_id, note.title],
        )?;

        Ok(())
    }

    pub fn delete_note(&self, id: NoteId) -> SqlResult<()> {
        self.conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_note(&self, id: NoteId) -> SqlResult<Option<Note>> {
        self.conn.query_row(
            "SELECT id, title, content, created_at, updated_at FROM notes WHERE id = ?1",
            [id],
            |row| self.row_to_note(row),
        ).optional()
    }

    pub fn get_note_by_title(&self, title: &str) -> SqlResult<Option<Note>> {
        self.conn.query_row(
            "SELECT id, title, content, created_at, updated_at FROM notes WHERE title = ?1",
            [title],
            |row| self.row_to_note(row),
        ).optional()
    }

    pub fn list_notes(&self) -> SqlResult<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, created_at, updated_at FROM notes ORDER BY updated_at DESC"
        )?;
        let notes = stmt.query_map([], |row| self.row_to_note(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn get_backlinks(&self, note_id: NoteId) -> SqlResult<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.title, n.content, n.created_at, n.updated_at
             FROM notes n
             JOIN links l ON l.source_id = n.id
             WHERE l.target_id = ?1
             ORDER BY n.title"
        )?;
        let notes = stmt.query_map([note_id], |row| self.row_to_note(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn get_forward_links(&self, note_id: NoteId) -> SqlResult<Vec<(String, Option<Note>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT l.target_title, n.id, n.title, n.content, n.created_at, n.updated_at
             FROM links l
             LEFT JOIN notes n ON l.target_id = n.id
             WHERE l.source_id = ?1
             ORDER BY l.target_title"
        )?;

        let links = stmt.query_map([note_id], |row| {
            let target_title: String = row.get(0)?;
            let target_note = if let Some(id) = row.get::<_, Option<NoteId>>(1)? {
                Some(Note {
                    id,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            } else {
                None
            };
            Ok((target_title, target_note))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(links)
    }

    pub fn search(&self, query: &str) -> SqlResult<Vec<SearchResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, snippet(notes_fts, 1, '<mark>', '</mark>', '...', 32) as snippet,
                    bm25(notes_fts) as score
             FROM notes_fts WHERE notes_fts MATCH ?1
             ORDER BY score LIMIT 50"
        )?;

        let results = stmt.query_map([query], |row| {
            Ok(SearchResult {
                note_id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                score: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    pub fn get_unlinked_mentions(&self, note_id: NoteId) -> SqlResult<Vec<Note>> {
        // Find notes that mention this note's title but don't have a formal link
        let note = match self.get_note(note_id)? {
            Some(n) => n,
            None => return Ok(Vec::new()),
        };

        let search_term = format!("%{}%", note.title);
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.title, n.content, n.created_at, n.updated_at
             FROM notes n
             WHERE n.id != ?1
             AND n.content LIKE ?2
             AND n.id NOT IN (
                 SELECT source_id FROM links WHERE target_id = ?1
             )
             ORDER BY n.title"
        )?;

        let notes = stmt.query_map(params![note_id, search_term], |row| self.row_to_note(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn get_orphan_notes(&self) -> SqlResult<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.title, n.content, n.created_at, n.updated_at
             FROM notes n
             WHERE n.id NOT IN (SELECT DISTINCT target_id FROM links WHERE target_id IS NOT NULL)
             AND n.id NOT IN (SELECT DISTINCT source_id FROM links)
             ORDER BY n.title"
        )?;
        let notes = stmt.query_map([], |row| self.row_to_note(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn note_count(&self) -> SqlResult<usize> {
        self.conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
    }

    pub fn link_count(&self) -> SqlResult<usize> {
        self.conn.query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))
    }

    fn row_to_note(&self, row: &rusqlite::Row) -> rusqlite::Result<Note> {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}
