# flashcard-trainer

Spaced repetition flashcard system with pluggable algorithms.

## Architecture Decisions

### Anki Import
- **Preserve scheduling**: Import due dates, intervals, ease factors from Anki
- Seamless migration for users switching from Anki
- Convert Anki's scheduling data to match selected algorithm (SM-2, FSRS, etc.)

### Study Session Flow
- **Configurable delay**: Auto-advance after N seconds, configurable (default: 1 second)
- User can set delay from 0 (instant) to 5 seconds
- Immediate next card on any keypress during delay

## Features

### Spaced Repetition Algorithms

Pluggable algorithm system:

```rust
pub trait SrsAlgorithm: Send + Sync {
    fn name(&self) -> &str;
    fn calculate_next_review(&self, card: &Card, response: Response) -> ReviewSchedule;
    fn initial_schedule(&self) -> ReviewSchedule;
}

pub enum Response {
    Again,      // Complete failure
    Hard,       // Difficult recall
    Good,       // Normal recall
    Easy,       // Effortless recall
}

pub struct ReviewSchedule {
    pub next_review: DateTime<Utc>,
    pub interval: Duration,
    pub ease_factor: f64,
}
```

**Built-in Algorithms:**
- SM-2 (SuperMemo/Anki classic)
- FSRS (Free Spaced Repetition Scheduler)
- Leitner Box (simple box progression)

### Deck Management

```rust
pub struct Deck {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub algorithm: String,  // Algorithm name
    pub algorithm_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}
```

### Card Types

**Basic:**
- Front: Question
- Back: Answer

**Cloze:**
- Text with `{{cloze}}` deletions
- Multiple clozes per card

**Image Occlusion:**
- Image with hidden regions
- Reveal on flip

### Import/Export

**Anki Import:**
- Parse .apkg files
- Import decks, cards, media
- Preserve scheduling data

**Export:**
- JSON format
- Markdown format
- Anki-compatible export

### Study Session

```rust
pub struct Session {
    pub deck_id: Uuid,
    pub cards_due: Vec<CardId>,
    pub cards_new: Vec<CardId>,
    pub cards_reviewed: usize,
    pub correct: usize,
    pub started_at: DateTime<Utc>,
}

pub struct SessionConfig {
    pub new_cards_per_day: usize,
    pub review_limit: Option<usize>,
    pub order: StudyOrder,
}

pub enum StudyOrder {
    DueFirst,
    NewFirst,
    Random,
}
```

### Statistics

- Retention rate over time
- Review forecast (upcoming reviews)
- Time spent studying
- Cards per state (new, learning, review, relearning)
- Streak tracking

## Data Model

```rust
pub struct Card {
    pub id: Uuid,
    pub deck_id: Uuid,
    pub card_type: CardType,
    pub front: String,      // Markdown
    pub back: String,       // Markdown
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CardSchedule {
    pub card_id: Uuid,
    pub due: DateTime<Utc>,
    pub interval: i64,       // days
    pub ease_factor: f64,
    pub review_count: i32,
    pub lapses: i32,
    pub state: CardState,
}

pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
    Suspended,
}

pub struct Review {
    pub id: Uuid,
    pub card_id: Uuid,
    pub response: Response,
    pub time_taken_ms: i64,
    pub reviewed_at: DateTime<Utc>,
}
```

## Views

**Deck List:**
- Decks with due count
- New cards available
- Progress bars

**Study View:**
- Card front
- Flip to reveal back
- Response buttons
- Progress indicator

**Statistics:**
- Charts and graphs
- Retention heatmap
- Forecast calendar

**Card Browser:**
- Search all cards
- Filter by deck/tag/state
- Edit cards

## Keybindings

| Key | Action |
|-----|--------|
| `space` | Flip card |
| `1` | Again |
| `2` | Hard |
| `3` | Good |
| `4` | Easy |
| `e` | Edit current card |
| `s` | Suspend card |
| `u` | Undo last review |
| `t` | Add tag |
| `i` | Card info |
| `/` | Search |
| `q` | End session |

## Configuration

```toml
# ~/.config/flashcard-trainer/config.toml
[study]
new_cards_per_day = 20
review_limit = 200
order = "due_first"

[algorithm]
default = "sm2"

[algorithm.sm2]
initial_ease = 2.5
easy_bonus = 1.3
hard_multiplier = 1.2

[algorithm.fsrs]
# FSRS-specific parameters

[display]
show_answer_timer = true
show_next_review = true
card_font_size = "normal"
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
rusqlite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
zip = "2"              # For .apkg import
syntect = "5"          # Syntax highlighting
```

## Database Schema

```sql
CREATE TABLE decks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    algorithm TEXT NOT NULL,
    algorithm_config TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE cards (
    id TEXT PRIMARY KEY,
    deck_id TEXT NOT NULL REFERENCES decks(id),
    card_type TEXT NOT NULL,
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE card_schedules (
    card_id TEXT PRIMARY KEY REFERENCES cards(id),
    due TEXT NOT NULL,
    interval INTEGER NOT NULL,
    ease_factor REAL NOT NULL,
    review_count INTEGER DEFAULT 0,
    lapses INTEGER DEFAULT 0,
    state TEXT NOT NULL
);

CREATE TABLE reviews (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id),
    response TEXT NOT NULL,
    time_taken_ms INTEGER,
    reviewed_at TEXT NOT NULL
);

CREATE TABLE card_tags (
    card_id TEXT NOT NULL REFERENCES cards(id),
    tag TEXT NOT NULL,
    PRIMARY KEY (card_id, tag)
);

CREATE INDEX idx_schedules_due ON card_schedules(due);
CREATE INDEX idx_cards_deck ON cards(deck_id);
```
