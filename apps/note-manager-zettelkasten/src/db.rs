//! Database operations for Zettelkasten.

use crate::models::{DbId, LinkType, Zettel, ZettelType, ZkStats};
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
            CREATE TABLE IF NOT EXISTS zettels (
                db_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                zettel_type TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '',
                source TEXT,
                sequence TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                link_type TEXT NOT NULL DEFAULT 'Reference',
                UNIQUE(source_id, target_id)
            );

            CREATE INDEX IF NOT EXISTS idx_zettels_id ON zettels(id);
            CREATE INDEX IF NOT EXISTS idx_zettels_type ON zettels(zettel_type);
            CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_id);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_id);
            "#,
        )
    }

    pub fn insert_zettel(&self, zettel: &Zettel) -> SqlResult<DbId> {
        self.conn.execute(
            "INSERT INTO zettels (id, title, content, zettel_type, tags, source, sequence, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                zettel.id,
                zettel.title,
                zettel.content,
                format!("{:?}", zettel.zettel_type),
                zettel.tags.join(","),
                zettel.source,
                zettel.sequence,
                zettel.created_at.to_rfc3339(),
                zettel.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_zettel(&self, zettel: &Zettel) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE zettels SET title = ?1, content = ?2, zettel_type = ?3, tags = ?4,
             source = ?5, sequence = ?6, updated_at = ?7 WHERE db_id = ?8",
            params![
                zettel.title,
                zettel.content,
                format!("{:?}", zettel.zettel_type),
                zettel.tags.join(","),
                zettel.source,
                zettel.sequence,
                Utc::now().to_rfc3339(),
                zettel.db_id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_zettel(&self, db_id: DbId) -> SqlResult<()> {
        // Get the zettel ID first
        let zettel_id: Option<String> = self.conn.query_row(
            "SELECT id FROM zettels WHERE db_id = ?1",
            [db_id],
            |row| row.get(0),
        ).optional()?;

        if let Some(id) = zettel_id {
            // Delete related links
            self.conn.execute("DELETE FROM links WHERE source_id = ?1 OR target_id = ?1", [&id])?;
        }

        self.conn.execute("DELETE FROM zettels WHERE db_id = ?1", [db_id])?;
        Ok(())
    }

    pub fn get_zettel(&self, db_id: DbId) -> SqlResult<Option<Zettel>> {
        self.conn.query_row(
            "SELECT db_id, id, title, content, zettel_type, tags, source, sequence, created_at, updated_at
             FROM zettels WHERE db_id = ?1",
            [db_id],
            |row| self.row_to_zettel(row),
        ).optional()
    }

    pub fn get_zettel_by_id(&self, id: &str) -> SqlResult<Option<Zettel>> {
        self.conn.query_row(
            "SELECT db_id, id, title, content, zettel_type, tags, source, sequence, created_at, updated_at
             FROM zettels WHERE id = ?1",
            [id],
            |row| self.row_to_zettel(row),
        ).optional()
    }

    pub fn list_zettels(&self) -> SqlResult<Vec<Zettel>> {
        let mut stmt = self.conn.prepare(
            "SELECT db_id, id, title, content, zettel_type, tags, source, sequence, created_at, updated_at
             FROM zettels ORDER BY created_at DESC"
        )?;
        let zettels = stmt.query_map([], |row| self.row_to_zettel(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(zettels)
    }

    pub fn list_by_type(&self, zettel_type: ZettelType) -> SqlResult<Vec<Zettel>> {
        let mut stmt = self.conn.prepare(
            "SELECT db_id, id, title, content, zettel_type, tags, source, sequence, created_at, updated_at
             FROM zettels WHERE zettel_type = ?1 ORDER BY created_at DESC"
        )?;
        let zettels = stmt.query_map([format!("{:?}", zettel_type)], |row| self.row_to_zettel(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(zettels)
    }

    pub fn list_by_tag(&self, tag: &str) -> SqlResult<Vec<Zettel>> {
        let pattern = format!("%{}%", tag);
        let mut stmt = self.conn.prepare(
            "SELECT db_id, id, title, content, zettel_type, tags, source, sequence, created_at, updated_at
             FROM zettels WHERE tags LIKE ?1 ORDER BY created_at DESC"
        )?;
        let zettels = stmt.query_map([pattern], |row| self.row_to_zettel(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(zettels)
    }

    pub fn add_link(&self, source_id: &str, target_id: &str, link_type: LinkType) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO links (source_id, target_id, link_type) VALUES (?1, ?2, ?3)",
            params![source_id, target_id, format!("{:?}", link_type)],
        )?;
        Ok(())
    }

    pub fn remove_link(&self, source_id: &str, target_id: &str) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM links WHERE source_id = ?1 AND target_id = ?2",
            [source_id, target_id],
        )?;
        Ok(())
    }

    pub fn get_outgoing_links(&self, zettel_id: &str) -> SqlResult<Vec<(Zettel, LinkType)>> {
        let mut stmt = self.conn.prepare(
            "SELECT z.db_id, z.id, z.title, z.content, z.zettel_type, z.tags, z.source, z.sequence,
                    z.created_at, z.updated_at, l.link_type
             FROM zettels z
             JOIN links l ON l.target_id = z.id
             WHERE l.source_id = ?1
             ORDER BY z.title"
        )?;
        let links = stmt.query_map([zettel_id], |row| {
            let zettel = self.row_to_zettel(row)?;
            let link_type_str: String = row.get(10)?;
            let link_type = parse_link_type(&link_type_str);
            Ok((zettel, link_type))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(links)
    }

    pub fn get_incoming_links(&self, zettel_id: &str) -> SqlResult<Vec<(Zettel, LinkType)>> {
        let mut stmt = self.conn.prepare(
            "SELECT z.db_id, z.id, z.title, z.content, z.zettel_type, z.tags, z.source, z.sequence,
                    z.created_at, z.updated_at, l.link_type
             FROM zettels z
             JOIN links l ON l.source_id = z.id
             WHERE l.target_id = ?1
             ORDER BY z.title"
        )?;
        let links = stmt.query_map([zettel_id], |row| {
            let zettel = self.row_to_zettel(row)?;
            let link_type_str: String = row.get(10)?;
            let link_type = parse_link_type(&link_type_str);
            Ok((zettel, link_type))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(links)
    }

    pub fn search(&self, query: &str) -> SqlResult<Vec<Zettel>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT db_id, id, title, content, zettel_type, tags, source, sequence, created_at, updated_at
             FROM zettels WHERE title LIKE ?1 OR content LIKE ?1 ORDER BY created_at DESC LIMIT 50"
        )?;
        let zettels = stmt.query_map([pattern], |row| self.row_to_zettel(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(zettels)
    }

    pub fn get_all_tags(&self) -> SqlResult<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare("SELECT tags FROM zettels WHERE tags != ''")?;
        let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        stmt.query_map([], |row| {
            let tags_str: String = row.get(0)?;
            Ok(tags_str)
        })?
        .for_each(|result| {
            if let Ok(tags_str) = result {
                for tag in tags_str.split(',') {
                    let tag = tag.trim().to_string();
                    if !tag.is_empty() {
                        *tag_counts.entry(tag).or_insert(0) += 1;
                    }
                }
            }
        });

        let mut tags: Vec<_> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(tags)
    }

    pub fn get_stats(&self) -> SqlResult<ZkStats> {
        let total_zettels: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM zettels", [], |row| row.get(0)
        )?;

        let fleeting: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM zettels WHERE zettel_type = 'Fleeting'", [], |row| row.get(0)
        )?;

        let literature: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM zettels WHERE zettel_type = 'Literature'", [], |row| row.get(0)
        )?;

        let permanent: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM zettels WHERE zettel_type = 'Permanent'", [], |row| row.get(0)
        )?;

        let hubs: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM zettels WHERE zettel_type = 'Hub'", [], |row| row.get(0)
        )?;

        let total_links: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM links", [], |row| row.get(0)
        )?;

        let total_tags = self.get_all_tags()?.len();

        Ok(ZkStats {
            total_zettels,
            fleeting,
            literature,
            permanent,
            hubs,
            total_links,
            total_tags,
        })
    }

    fn row_to_zettel(&self, row: &rusqlite::Row) -> rusqlite::Result<Zettel> {
        let type_str: String = row.get(4)?;
        let tags_str: String = row.get(5)?;

        Ok(Zettel {
            db_id: row.get(0)?,
            id: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            zettel_type: parse_zettel_type(&type_str),
            tags: if tags_str.is_empty() {
                Vec::new()
            } else {
                tags_str.split(',').map(|s| s.trim().to_string()).collect()
            },
            source: row.get(6)?,
            sequence: row.get(7)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

fn parse_zettel_type(s: &str) -> ZettelType {
    match s {
        "Fleeting" => ZettelType::Fleeting,
        "Literature" => ZettelType::Literature,
        "Permanent" => ZettelType::Permanent,
        "Hub" => ZettelType::Hub,
        _ => ZettelType::Fleeting,
    }
}

fn parse_link_type(s: &str) -> LinkType {
    match s {
        "Reference" => LinkType::Reference,
        "Continues" => LinkType::Continues,
        "Supports" => LinkType::Supports,
        "Contradicts" => LinkType::Contradicts,
        "Related" => LinkType::Related,
        _ => LinkType::Reference,
    }
}
