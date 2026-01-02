//! Data models for flashcard trainer.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifiers.
pub type DeckId = Uuid;
pub type CardId = Uuid;
pub type ReviewId = Uuid;

/// A flashcard deck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    /// Unique identifier.
    pub id: DeckId,
    /// Deck name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Algorithm name.
    pub algorithm: String,
    /// Algorithm-specific configuration.
    pub algorithm_config: serde_json::Value,
    /// When the deck was created.
    pub created_at: DateTime<Utc>,
    /// Tags.
    pub tags: Vec<String>,
}

impl Deck {
    /// Create a new deck.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            algorithm: "sm2".to_string(),
            algorithm_config: serde_json::json!({}),
            created_at: Utc::now(),
            tags: Vec::new(),
        }
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set algorithm.
    pub fn with_algorithm(mut self, algo: impl Into<String>) -> Self {
        self.algorithm = algo.into();
        self
    }
}

/// Card type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    /// Basic front/back.
    Basic,
    /// Cloze deletion.
    Cloze,
    /// Image occlusion.
    ImageOcclusion,
}

impl Default for CardType {
    fn default() -> Self {
        Self::Basic
    }
}

/// A flashcard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    /// Unique identifier.
    pub id: CardId,
    /// Parent deck.
    pub deck_id: DeckId,
    /// Card type.
    pub card_type: CardType,
    /// Front content (Markdown).
    pub front: String,
    /// Back content (Markdown).
    pub back: String,
    /// Tags.
    pub tags: Vec<String>,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last updated.
    pub updated_at: DateTime<Utc>,
}

impl Card {
    /// Create a new basic card.
    pub fn new_basic(deck_id: DeckId, front: impl Into<String>, back: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            deck_id,
            card_type: CardType::Basic,
            front: front.into(),
            back: back.into(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new cloze card.
    pub fn new_cloze(deck_id: DeckId, text: impl Into<String>) -> Self {
        let text = text.into();
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            deck_id,
            card_type: CardType::Cloze,
            front: text.clone(),
            back: text,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Card state in the learning process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardState {
    /// Never reviewed.
    New,
    /// In initial learning phase.
    Learning,
    /// In regular review.
    Review,
    /// Failed review, relearning.
    Relearning,
    /// Manually suspended.
    Suspended,
}

impl Default for CardState {
    fn default() -> Self {
        Self::New
    }
}

impl CardState {
    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Learning => "Learning",
            Self::Review => "Review",
            Self::Relearning => "Relearning",
            Self::Suspended => "Suspended",
        }
    }
}

/// Scheduling information for a card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSchedule {
    /// Card ID.
    pub card_id: CardId,
    /// Next review due date.
    pub due: DateTime<Utc>,
    /// Current interval in days.
    pub interval: i64,
    /// Ease factor.
    pub ease_factor: f64,
    /// Number of reviews.
    pub review_count: i32,
    /// Number of lapses (failed reviews).
    pub lapses: i32,
    /// Current state.
    pub state: CardState,
}

impl CardSchedule {
    /// Create initial schedule for a new card.
    pub fn new(card_id: CardId) -> Self {
        Self {
            card_id,
            due: Utc::now(),
            interval: 0,
            ease_factor: 2.5,
            review_count: 0,
            lapses: 0,
            state: CardState::New,
        }
    }

    /// Check if due for review.
    pub fn is_due(&self) -> bool {
        self.state != CardState::Suspended && self.due <= Utc::now()
    }
}

/// User response to a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    /// Complete failure, need to see again soon.
    Again,
    /// Difficult recall.
    Hard,
    /// Normal recall.
    Good,
    /// Effortless recall.
    Easy,
}

impl Response {
    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Again => "Again",
            Self::Hard => "Hard",
            Self::Good => "Good",
            Self::Easy => "Easy",
        }
    }

    /// Get associated key.
    pub fn key(&self) -> char {
        match self {
            Self::Again => '1',
            Self::Hard => '2',
            Self::Good => '3',
            Self::Easy => '4',
        }
    }
}

/// A review record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Unique identifier.
    pub id: ReviewId,
    /// Card that was reviewed.
    pub card_id: CardId,
    /// User's response.
    pub response: Response,
    /// Time taken in milliseconds.
    pub time_taken_ms: i64,
    /// When the review occurred.
    pub reviewed_at: DateTime<Utc>,
}

impl Review {
    /// Create a new review.
    pub fn new(card_id: CardId, response: Response, time_taken_ms: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            card_id,
            response,
            time_taken_ms,
            reviewed_at: Utc::now(),
        }
    }
}

/// Review schedule returned by algorithm.
#[derive(Debug, Clone)]
pub struct ReviewSchedule {
    /// Next review date.
    pub next_review: DateTime<Utc>,
    /// Interval until next review.
    pub interval: Duration,
    /// Updated ease factor.
    pub ease_factor: f64,
    /// New state.
    pub state: CardState,
}

/// Study session.
#[derive(Debug, Clone)]
pub struct Session {
    /// Deck being studied.
    pub deck_id: DeckId,
    /// Cards due for review.
    pub cards_due: Vec<CardId>,
    /// New cards to introduce.
    pub cards_new: Vec<CardId>,
    /// Current card index.
    pub current_index: usize,
    /// Cards reviewed this session.
    pub cards_reviewed: usize,
    /// Correct responses.
    pub correct: usize,
    /// When session started.
    pub started_at: DateTime<Utc>,
    /// Whether card is flipped.
    pub flipped: bool,
    /// Time when current card was shown.
    pub card_shown_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session.
    pub fn new(deck_id: DeckId, cards_due: Vec<CardId>, cards_new: Vec<CardId>) -> Self {
        let now = Utc::now();
        Self {
            deck_id,
            cards_due,
            cards_new,
            current_index: 0,
            cards_reviewed: 0,
            correct: 0,
            started_at: now,
            flipped: false,
            card_shown_at: now,
        }
    }

    /// Get current card ID.
    pub fn current_card(&self) -> Option<CardId> {
        // First review due cards, then new cards
        if self.current_index < self.cards_due.len() {
            Some(self.cards_due[self.current_index])
        } else {
            let new_idx = self.current_index - self.cards_due.len();
            self.cards_new.get(new_idx).copied()
        }
    }

    /// Check if session is complete.
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.cards_due.len() + self.cards_new.len()
    }

    /// Get total cards in session.
    pub fn total_cards(&self) -> usize {
        self.cards_due.len() + self.cards_new.len()
    }

    /// Get time spent on current card.
    pub fn card_time(&self) -> Duration {
        Utc::now().signed_duration_since(self.card_shown_at)
    }

    /// Move to next card.
    pub fn next_card(&mut self) {
        self.current_index += 1;
        self.flipped = false;
        self.card_shown_at = Utc::now();
    }

    /// Record a response.
    pub fn record_response(&mut self, response: Response) {
        self.cards_reviewed += 1;
        if matches!(response, Response::Good | Response::Easy) {
            self.correct += 1;
        }
    }

    /// Get accuracy rate.
    pub fn accuracy(&self) -> f64 {
        if self.cards_reviewed == 0 {
            0.0
        } else {
            self.correct as f64 / self.cards_reviewed as f64
        }
    }
}

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// New cards to introduce per day.
    pub new_cards_per_day: usize,
    /// Maximum reviews per session.
    pub review_limit: Option<usize>,
    /// Study order.
    pub order: StudyOrder,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            new_cards_per_day: 20,
            review_limit: Some(200),
            order: StudyOrder::DueFirst,
        }
    }
}

/// Study order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyOrder {
    DueFirst,
    NewFirst,
    Random,
}

/// Deck statistics.
#[derive(Debug, Clone, Default)]
pub struct DeckStats {
    /// Total cards.
    pub total_cards: usize,
    /// New cards.
    pub new_cards: usize,
    /// Cards in learning.
    pub learning_cards: usize,
    /// Cards in review.
    pub review_cards: usize,
    /// Cards due today.
    pub due_today: usize,
    /// Average ease factor.
    pub average_ease: f64,
    /// Retention rate.
    pub retention_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_creation() {
        let deck = Deck::new("Test Deck").with_description("A test deck");
        assert_eq!(deck.name, "Test Deck");
        assert_eq!(deck.description, Some("A test deck".to_string()));
    }

    #[test]
    fn test_card_creation() {
        let deck_id = Uuid::new_v4();
        let card = Card::new_basic(deck_id, "What is 2+2?", "4");
        assert_eq!(card.front, "What is 2+2?");
        assert_eq!(card.back, "4");
        assert_eq!(card.card_type, CardType::Basic);
    }

    #[test]
    fn test_session() {
        let deck_id = Uuid::new_v4();
        let cards = vec![Uuid::new_v4(), Uuid::new_v4()];
        let mut session = Session::new(deck_id, cards.clone(), vec![]);

        assert_eq!(session.current_card(), Some(cards[0]));
        assert!(!session.is_complete());

        session.next_card();
        assert_eq!(session.current_card(), Some(cards[1]));

        session.next_card();
        assert!(session.is_complete());
    }
}
