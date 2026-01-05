//! Database operations for personal wiki.

use crate::models::{Category, CategoryId, Page, PageId, Revision, RevisionId, WikiStats};
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
            CREATE TABLE IF NOT EXISTS pages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL UNIQUE COLLATE NOCASE,
                content TEXT NOT NULL DEFAULT '',
                redirect_to TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                description TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS page_categories (
                page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
                PRIMARY KEY (page_id, category_id)
            );

            CREATE TABLE IF NOT EXISTS links (
                source_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                target_title TEXT NOT NULL,
                target_id INTEGER REFERENCES pages(id) ON DELETE SET NULL,
                PRIMARY KEY (source_id, target_title)
            );

            CREATE TABLE IF NOT EXISTS revisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                content TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_pages_title ON pages(title);
            CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_id);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_id);
            CREATE INDEX IF NOT EXISTS idx_revisions_page ON revisions(page_id);
            "#,
        )
    }

    // Pages

    pub fn insert_page(&self, page: &Page) -> SqlResult<PageId> {
        self.conn.execute(
            "INSERT INTO pages (title, content, redirect_to, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                page.title,
                page.content,
                page.redirect_to,
                page.created_at.to_rfc3339(),
                page.updated_at.to_rfc3339(),
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        self.update_page_metadata(id, page)?;
        Ok(id)
    }

    pub fn update_page(&self, page: &Page, summary: &str) -> SqlResult<()> {
        // Save revision first
        self.save_revision(page.id, &page.content, summary)?;

        self.conn.execute(
            "UPDATE pages SET title = ?1, content = ?2, redirect_to = ?3, updated_at = ?4 WHERE id = ?5",
            params![
                page.title,
                page.content,
                page.redirect_to,
                Utc::now().to_rfc3339(),
                page.id,
            ],
        )?;
        self.update_page_metadata(page.id, page)?;
        Ok(())
    }

    fn update_page_metadata(&self, page_id: PageId, page: &Page) -> SqlResult<()> {
        // Update categories
        self.conn.execute("DELETE FROM page_categories WHERE page_id = ?1", [page_id])?;
        let categories = page.extract_categories();
        for cat_name in &categories {
            let cat_id = self.get_or_create_category(cat_name)?;
            self.conn.execute(
                "INSERT OR IGNORE INTO page_categories (page_id, category_id) VALUES (?1, ?2)",
                [page_id, cat_id],
            )?;
        }

        // Update links
        self.conn.execute("DELETE FROM links WHERE source_id = ?1", [page_id])?;
        let links = page.extract_links();
        for target_title in links {
            let target_id: Option<PageId> = self.conn.query_row(
                "SELECT id FROM pages WHERE title = ?1 COLLATE NOCASE",
                [&target_title],
                |row| row.get(0),
            ).optional()?;

            self.conn.execute(
                "INSERT INTO links (source_id, target_title, target_id) VALUES (?1, ?2, ?3)",
                params![page_id, target_title, target_id],
            )?;
        }

        Ok(())
    }

    pub fn delete_page(&self, id: PageId) -> SqlResult<()> {
        self.conn.execute("DELETE FROM pages WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_page(&self, id: PageId) -> SqlResult<Option<Page>> {
        self.conn.query_row(
            "SELECT id, title, content, redirect_to, created_at, updated_at FROM pages WHERE id = ?1",
            [id],
            |row| self.row_to_page(row),
        ).optional()
    }

    pub fn get_page_by_title(&self, title: &str) -> SqlResult<Option<Page>> {
        self.conn.query_row(
            "SELECT id, title, content, redirect_to, created_at, updated_at FROM pages WHERE title = ?1 COLLATE NOCASE",
            [title],
            |row| self.row_to_page(row),
        ).optional()
    }

    pub fn list_pages(&self) -> SqlResult<Vec<Page>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, redirect_to, created_at, updated_at FROM pages ORDER BY title"
        )?;
        let pages = stmt.query_map([], |row| self.row_to_page(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pages)
    }

    pub fn list_recent_pages(&self, limit: usize) -> SqlResult<Vec<Page>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, redirect_to, created_at, updated_at FROM pages ORDER BY updated_at DESC LIMIT ?1"
        )?;
        let pages = stmt.query_map([limit], |row| self.row_to_page(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pages)
    }

    pub fn list_pages_in_category(&self, category_id: CategoryId) -> SqlResult<Vec<Page>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.title, p.content, p.redirect_to, p.created_at, p.updated_at
             FROM pages p
             JOIN page_categories pc ON pc.page_id = p.id
             WHERE pc.category_id = ?1
             ORDER BY p.title"
        )?;
        let pages = stmt.query_map([category_id], |row| self.row_to_page(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pages)
    }

    pub fn get_orphan_pages(&self) -> SqlResult<Vec<Page>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, redirect_to, created_at, updated_at
             FROM pages
             WHERE id NOT IN (SELECT DISTINCT target_id FROM links WHERE target_id IS NOT NULL)
             ORDER BY title"
        )?;
        let pages = stmt.query_map([], |row| self.row_to_page(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pages)
    }

    pub fn get_wanted_pages(&self) -> SqlResult<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT target_title, COUNT(*) as cnt
             FROM links WHERE target_id IS NULL
             GROUP BY target_title
             ORDER BY cnt DESC"
        )?;
        let wanted = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(wanted)
    }

    pub fn get_backlinks(&self, page_id: PageId) -> SqlResult<Vec<Page>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.title, p.content, p.redirect_to, p.created_at, p.updated_at
             FROM pages p
             JOIN links l ON l.source_id = p.id
             WHERE l.target_id = ?1
             ORDER BY p.title"
        )?;
        let pages = stmt.query_map([page_id], |row| self.row_to_page(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pages)
    }

    pub fn search(&self, query: &str) -> SqlResult<Vec<Page>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, redirect_to, created_at, updated_at
             FROM pages WHERE title LIKE ?1 OR content LIKE ?1 ORDER BY title LIMIT 50"
        )?;
        let pages = stmt.query_map([pattern], |row| self.row_to_page(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pages)
    }

    pub fn get_random_page(&self) -> SqlResult<Option<Page>> {
        self.conn.query_row(
            "SELECT id, title, content, redirect_to, created_at, updated_at FROM pages ORDER BY RANDOM() LIMIT 1",
            [],
            |row| self.row_to_page(row),
        ).optional()
    }

    // Categories

    fn get_or_create_category(&self, name: &str) -> SqlResult<CategoryId> {
        if let Some(id) = self.conn.query_row(
            "SELECT id FROM categories WHERE name = ?1 COLLATE NOCASE",
            [name],
            |row| row.get(0),
        ).optional()? {
            return Ok(id);
        }

        self.conn.execute(
            "INSERT INTO categories (name) VALUES (?1)",
            [name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_categories(&self) -> SqlResult<Vec<Category>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.name, c.description,
                    (SELECT COUNT(*) FROM page_categories WHERE category_id = c.id) as page_count
             FROM categories c ORDER BY c.name"
        )?;
        let categories = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                page_count: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(categories)
    }

    // Revisions

    fn save_revision(&self, page_id: PageId, content: &str, summary: &str) -> SqlResult<RevisionId> {
        self.conn.execute(
            "INSERT INTO revisions (page_id, content, summary, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![page_id, content, summary, Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_revisions(&self, page_id: PageId) -> SqlResult<Vec<Revision>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, page_id, content, summary, created_at FROM revisions WHERE page_id = ?1 ORDER BY created_at DESC"
        )?;
        let revisions = stmt.query_map([page_id], |row| {
            Ok(Revision {
                id: row.get(0)?,
                page_id: row.get(1)?,
                content: row.get(2)?,
                summary: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(revisions)
    }

    // Stats

    pub fn get_stats(&self) -> SqlResult<WikiStats> {
        let total_pages: usize = self.conn.query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))?;
        let total_categories: usize = self.conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
        let total_revisions: usize = self.conn.query_row("SELECT COUNT(*) FROM revisions", [], |row| row.get(0))?;
        let total_links: usize = self.conn.query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))?;
        let orphan_pages: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM pages WHERE id NOT IN (SELECT DISTINCT target_id FROM links WHERE target_id IS NOT NULL)",
            [],
            |row| row.get(0),
        )?;

        Ok(WikiStats {
            total_pages,
            total_categories,
            total_revisions,
            total_links,
            orphan_pages,
        })
    }

    fn row_to_page(&self, row: &rusqlite::Row) -> rusqlite::Result<Page> {
        Ok(Page {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            categories: Vec::new(), // Will be populated if needed
            redirect_to: row.get(3)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}
