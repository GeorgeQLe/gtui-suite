# habit-tracker

Track daily habits with streaks, quantitative goals, and correlation analysis.

## Architecture Decisions

### Correlation Analysis
- **Hybrid compute**: Cache correlation results, recalculate when data changes significantly
- Trigger recalculation on new entries or when cache is stale (>24 hours)
- Display cached results immediately, update in background if stale

### Streak Logic
- **Strict schedule**: Streak breaks if any scheduled day is missed
- For MWF habits, missing Friday breaks the streak even if Saturday is not scheduled
- Most accurate reflection of habit adherence

## Features

### Habit Types

**Binary Habits:**
- Simple yes/no daily checkoff
- Streak counting
- Best streak tracking

**Quantitative Habits:**
- Numeric goals (e.g., "8 glasses of water", "30 minutes reading")
- Progress toward daily goal
- Average/total tracking

### Scheduling

```rust
pub enum Schedule {
    Daily,
    Weekly { days: Vec<Weekday> },     // e.g., Mon, Wed, Fri
    Monthly { days: Vec<u8> },         // e.g., 1st and 15th
    Interval { every_n_days: u32 },    // e.g., every 3 days
}
```

### Views

**Daily View:**
- List of habits due today
- Quick checkoff with space/enter
- Numeric input for quantitative habits
- Today's completion status

**Calendar Heatmap:**
- GitHub-style contribution graph
- Color intensity = completion rate
- Navigate by week/month

**Streak View:**
- Current streak per habit
- Best streak ever
- Streak calendar visualization

**Statistics:**
- Overall completion rate
- Per-habit analytics
- Trend over time (improving/declining)
- Correlation analysis

### Correlation Analysis

Discover relationships between habits:

```rust
pub struct Correlation {
    habit_a: HabitId,
    habit_b: HabitId,
    coefficient: f64,  // -1.0 to 1.0
    significance: f64, // p-value
}

// "When I exercise, I'm 40% more likely to sleep well"
```

## Data Model

```rust
pub struct Habit {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schedule: Schedule,
    pub metric: Metric,
    pub color: Option<Color>,
    pub created_at: DateTime<Utc>,
    pub archived: bool,
}

pub enum Metric {
    Binary,
    Quantity { goal: f64, unit: String },
}

pub struct HabitEntry {
    pub id: Uuid,
    pub habit_id: Uuid,
    pub date: NaiveDate,
    pub completed: bool,
    pub value: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate habits |
| `space` | Toggle completion |
| `enter` | Open habit details |
| `a` | Add new habit |
| `e` | Edit habit |
| `d` | Delete habit (confirm) |
| `n` | Add note to today's entry |
| `c` | Calendar view |
| `s` | Statistics view |
| `h/l` | Previous/next day |
| `t` | Jump to today |
| `/` | Search habits |
| `q` | Quit |

## Configuration

```toml
# ~/.config/habit-tracker/config.toml
[display]
week_start = "monday"  # or "sunday"
date_format = "%Y-%m-%d"

[notifications]
remind_time = "20:00"  # Daily reminder
incomplete_warning = true

[export]
format = "csv"  # or "json"
path = "~/habits-export"
```

## Export

- CSV export for spreadsheet analysis
- JSON export for backup/migration
- Include date range selection

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
tui-keybinds = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
rusqlite = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
```

## Database Schema

```sql
CREATE TABLE habits (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    schedule_type TEXT NOT NULL,
    schedule_data TEXT,  -- JSON for schedule details
    metric_type TEXT NOT NULL,
    metric_goal REAL,
    metric_unit TEXT,
    color TEXT,
    created_at TEXT NOT NULL,
    archived INTEGER DEFAULT 0
);

CREATE TABLE habit_entries (
    id TEXT PRIMARY KEY,
    habit_id TEXT NOT NULL REFERENCES habits(id),
    date TEXT NOT NULL,
    completed INTEGER NOT NULL,
    value REAL,
    notes TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(habit_id, date)
);

CREATE INDEX idx_entries_date ON habit_entries(date);
CREATE INDEX idx_entries_habit ON habit_entries(habit_id);
```
