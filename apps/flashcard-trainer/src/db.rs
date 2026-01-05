//! Database operations for flashcard trainer.

use crate::models::{Card, CardId, CardSchedule, CardState, CardType, Deck, DeckId, DeckStats, Review};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type DbResult<T> = Result<T, DbError>;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> DbResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    pub fn in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> DbResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS decks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                algorithm TEXT NOT NULL,
                algorithm_config TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cards (
                id TEXT PRIMARY KEY,
                deck_id TEXT NOT NULL REFERENCES decks(id),
                card_type TEXT NOT NULL,
                front TEXT NOT NULL,
                back TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS card_schedules (
                card_id TEXT PRIMARY KEY REFERENCES cards(id),
                due TEXT NOT NULL,
                interval INTEGER NOT NULL,
                ease_factor REAL NOT NULL,
                review_count INTEGER DEFAULT 0,
                lapses INTEGER DEFAULT 0,
                state TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS reviews (
                id TEXT PRIMARY KEY,
                card_id TEXT NOT NULL REFERENCES cards(id),
                response TEXT NOT NULL,
                time_taken_ms INTEGER,
                reviewed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS card_tags (
                card_id TEXT NOT NULL REFERENCES cards(id),
                tag TEXT NOT NULL,
                PRIMARY KEY (card_id, tag)
            );

            CREATE INDEX IF NOT EXISTS idx_schedules_due ON card_schedules(due);
            CREATE INDEX IF NOT EXISTS idx_cards_deck ON cards(deck_id);
            "#,
        )?;
        Ok(())
    }

    // Deck operations

    pub fn insert_deck(&self, deck: &Deck) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO decks (id, name, description, algorithm, algorithm_config, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                deck.id.to_string(),
                deck.name,
                deck.description,
                deck.algorithm,
                serde_json::to_string(&deck.algorithm_config)?,
                deck.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_deck(&self, deck: &Deck) -> DbResult<()> {
        self.conn.execute(
            "UPDATE decks SET name = ?2, description = ?3, algorithm = ?4, algorithm_config = ?5 WHERE id = ?1",
            params![
                deck.id.to_string(),
                deck.name,
                deck.description,
                deck.algorithm,
                serde_json::to_string(&deck.algorithm_config)?,
            ],
        )?;
        Ok(())
    }

    pub fn delete_deck(&self, id: DeckId) -> DbResult<()> {
        self.conn.execute("DELETE FROM card_schedules WHERE card_id IN (SELECT id FROM cards WHERE deck_id = ?1)", params![id.to_string()])?;
        self.conn.execute("DELETE FROM reviews WHERE card_id IN (SELECT id FROM cards WHERE deck_id = ?1)", params![id.to_string()])?;
        self.conn.execute("DELETE FROM card_tags WHERE card_id IN (SELECT id FROM cards WHERE deck_id = ?1)", params![id.to_string()])?;
        self.conn.execute("DELETE FROM cards WHERE deck_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM decks WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn get_deck(&self, id: DeckId) -> DbResult<Option<Deck>> {
        let mut stmt = self.conn.prepare("SELECT * FROM decks WHERE id = ?1")?;
        let deck = stmt.query_row(params![id.to_string()], |row| Ok(parse_deck_row(row)?));

        match deck {
            Ok(d) => Ok(Some(d)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_decks(&self) -> DbResult<Vec<Deck>> {
        let mut stmt = self.conn.prepare("SELECT * FROM decks ORDER BY name")?;
        let decks = stmt
            .query_map([], |row| Ok(parse_deck_row(row)?))?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(decks)
    }

    // Card operations

    pub fn insert_card(&self, card: &Card) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO cards (id, deck_id, card_type, front, back, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                card.id.to_string(),
                card.deck_id.to_string(),
                format!("{:?}", card.card_type).to_lowercase(),
                card.front,
                card.back,
                card.created_at.to_rfc3339(),
                card.updated_at.to_rfc3339(),
            ],
        )?;

        // Create initial schedule
        let schedule = CardSchedule::new(card.id);
        self.upsert_schedule(&schedule)?;

        // Insert tags
        for tag in &card.tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO card_tags (card_id, tag) VALUES (?1, ?2)",
                params![card.id.to_string(), tag],
            )?;
        }

        Ok(())
    }

    pub fn update_card(&self, card: &Card) -> DbResult<()> {
        self.conn.execute(
            "UPDATE cards SET front = ?2, back = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                card.id.to_string(),
                card.front,
                card.back,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_card(&self, id: CardId) -> DbResult<()> {
        self.conn.execute("DELETE FROM card_schedules WHERE card_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM reviews WHERE card_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM card_tags WHERE card_id = ?1", params![id.to_string()])?;
        self.conn.execute("DELETE FROM cards WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn get_card(&self, id: CardId) -> DbResult<Option<Card>> {
        let mut stmt = self.conn.prepare("SELECT * FROM cards WHERE id = ?1")?;
        let card = stmt.query_row(params![id.to_string()], |row| Ok(parse_card_row(row)?));

        match card {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_cards_for_deck(&self, deck_id: DeckId) -> DbResult<Vec<Card>> {
        let mut stmt = self.conn.prepare("SELECT * FROM cards WHERE deck_id = ?1")?;
        let cards = stmt
            .query_map(params![deck_id.to_string()], |row| Ok(parse_card_row(row)?))?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(cards)
    }

    // Schedule operations

    pub fn upsert_schedule(&self, schedule: &CardSchedule) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO card_schedules (card_id, due, interval, ease_factor, review_count, lapses, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(card_id) DO UPDATE SET
                due = excluded.due, interval = excluded.interval, ease_factor = excluded.ease_factor,
                review_count = excluded.review_count, lapses = excluded.lapses, state = excluded.state",
            params![
                schedule.card_id.to_string(),
                schedule.due.to_rfc3339(),
                schedule.interval,
                schedule.ease_factor,
                schedule.review_count,
                schedule.lapses,
                format!("{:?}", schedule.state).to_lowercase(),
            ],
        )?;
        Ok(())
    }

    pub fn get_schedule(&self, card_id: CardId) -> DbResult<Option<CardSchedule>> {
        let mut stmt = self.conn.prepare("SELECT * FROM card_schedules WHERE card_id = ?1")?;
        let schedule = stmt.query_row(params![card_id.to_string()], |row| Ok(parse_schedule_row(row)?));

        match schedule {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_due_cards(&self, deck_id: DeckId) -> DbResult<Vec<CardId>> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT c.id FROM cards c
             JOIN card_schedules s ON c.id = s.card_id
             WHERE c.deck_id = ?1 AND s.due <= ?2 AND s.state != 'suspended'
             ORDER BY s.due"
        )?;

        let ids = stmt
            .query_map(params![deck_id.to_string(), now], |row| {
                let id_str: String = row.get(0)?;
                Ok(Uuid::parse_str(&id_str).unwrap())
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(ids)
    }

    pub fn get_new_cards(&self, deck_id: DeckId, limit: usize) -> DbResult<Vec<CardId>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id FROM cards c
             JOIN card_schedules s ON c.id = s.card_id
             WHERE c.deck_id = ?1 AND s.state = 'new'
             LIMIT ?2"
        )?;

        let ids = stmt
            .query_map(params![deck_id.to_string(), limit as i64], |row| {
                let id_str: String = row.get(0)?;
                Ok(Uuid::parse_str(&id_str).unwrap())
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(ids)
    }

    // Review operations

    pub fn insert_review(&self, review: &Review) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO reviews (id, card_id, response, time_taken_ms, reviewed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                review.id.to_string(),
                review.card_id.to_string(),
                format!("{:?}", review.response).to_lowercase(),
                review.time_taken_ms,
                review.reviewed_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    // Statistics

    pub fn get_deck_stats(&self, deck_id: DeckId) -> DbResult<DeckStats> {
        let now = Utc::now().to_rfc3339();

        let mut stmt = self.conn.prepare(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN s.state = 'new' THEN 1 ELSE 0 END) as new_count,
                SUM(CASE WHEN s.state = 'learning' THEN 1 ELSE 0 END) as learning_count,
                SUM(CASE WHEN s.state = 'review' THEN 1 ELSE 0 END) as review_count,
                SUM(CASE WHEN s.due <= ?2 AND s.state != 'suspended' THEN 1 ELSE 0 END) as due_count,
                AVG(s.ease_factor) as avg_ease
             FROM cards c
             JOIN card_schedules s ON c.id = s.card_id
             WHERE c.deck_id = ?1"
        )?;

        let stats = stmt.query_row(params![deck_id.to_string(), now], |row| {
            Ok(DeckStats {
                total_cards: row.get::<_, i32>(0)? as usize,
                new_cards: row.get::<_, i32>(1)? as usize,
                learning_cards: row.get::<_, i32>(2)? as usize,
                review_cards: row.get::<_, i32>(3)? as usize,
                due_today: row.get::<_, i32>(4)? as usize,
                average_ease: row.get::<_, f64>(5).unwrap_or(2.5),
                retention_rate: 0.0, // Calculate separately if needed
            })
        })?;

        Ok(stats)
    }
}

fn parse_deck_row(row: &rusqlite::Row) -> SqlResult<Deck> {
    let id_str: String = row.get("id")?;
    let config_str: String = row.get("algorithm_config")?;
    let created_str: String = row.get("created_at")?;

    Ok(Deck {
        id: Uuid::parse_str(&id_str).unwrap(),
        name: row.get("name")?,
        description: row.get("description")?,
        algorithm: row.get("algorithm")?,
        algorithm_config: serde_json::from_str(&config_str).unwrap_or_default(),
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        tags: Vec::new(),
    })
}

fn parse_card_row(row: &rusqlite::Row) -> SqlResult<Card> {
    let id_str: String = row.get("id")?;
    let deck_id_str: String = row.get("deck_id")?;
    let type_str: String = row.get("card_type")?;
    let created_str: String = row.get("created_at")?;
    let updated_str: String = row.get("updated_at")?;

    let card_type = match type_str.as_str() {
        "cloze" => CardType::Cloze,
        "imageocclusion" => CardType::ImageOcclusion,
        _ => CardType::Basic,
    };

    Ok(Card {
        id: Uuid::parse_str(&id_str).unwrap(),
        deck_id: Uuid::parse_str(&deck_id_str).unwrap(),
        card_type,
        front: row.get("front")?,
        back: row.get("back")?,
        tags: Vec::new(),
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

fn parse_schedule_row(row: &rusqlite::Row) -> SqlResult<CardSchedule> {
    let card_id_str: String = row.get("card_id")?;
    let due_str: String = row.get("due")?;
    let state_str: String = row.get("state")?;

    let state = match state_str.as_str() {
        "learning" => CardState::Learning,
        "review" => CardState::Review,
        "relearning" => CardState::Relearning,
        "suspended" => CardState::Suspended,
        _ => CardState::New,
    };

    Ok(CardSchedule {
        card_id: Uuid::parse_str(&card_id_str).unwrap(),
        due: DateTime::parse_from_rfc3339(&due_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        interval: row.get("interval")?,
        ease_factor: row.get("ease_factor")?,
        review_count: row.get("review_count")?,
        lapses: row.get("lapses")?,
        state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_crud() {
        let db = Database::in_memory().unwrap();
        let deck = Deck::new("Test");
        db.insert_deck(&deck).unwrap();

        let loaded = db.get_deck(deck.id).unwrap().unwrap();
        assert_eq!(loaded.name, "Test");

        let decks = db.list_decks().unwrap();
        assert_eq!(decks.len(), 1);
    }

    #[test]
    fn test_card_with_schedule() {
        let db = Database::in_memory().unwrap();
        let deck = Deck::new("Test");
        db.insert_deck(&deck).unwrap();

        let card = Card::new_basic(deck.id, "Q?", "A");
        db.insert_card(&card).unwrap();

        let schedule = db.get_schedule(card.id).unwrap().unwrap();
        assert_eq!(schedule.state, CardState::New);
    }
}
