# time-tracker

Track time with projects, Pomodoro technique, and detailed analytics.

## Architecture Decisions

### Idle Detection
- **No keyboard input**: Track time since last keypress in the app
- TUI-native approach - no system-level dependencies
- Configurable threshold (default: 5 minutes)
- Actions: pause timer, prompt user, or discard idle time

### Audio Cues
- **Both + configurable**: Desktop notification with sound AND terminal bell fallback
- Desktop notification via notify-rust when available
- Terminal bell (\x07) as universal fallback
- User can enable/disable each independently in config

## Features

### Timer Modes

**Simple Timer:**
- Start/stop tracking
- Optional description
- Assign to project/client

**Pomodoro Mode:**
- Configurable work/break intervals (default 25/5)
- Long break after N pomodoros
- Session counting
- Break reminders

### Projects & Clients

```rust
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub hourly_rate: Option<Decimal>,
    pub currency: String,
    pub archived: bool,
}

pub struct Project {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub name: String,
    pub color: Option<Color>,
    pub budget_hours: Option<f64>,
    pub archived: bool,
}
```

### Time Entries

```rust
pub struct TimeEntry {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_secs: Option<i64>,
    pub tags: Vec<String>,
    pub billable: bool,
}
```

### Analytics

**Daily/Weekly/Monthly Reports:**
- Total time per project
- Breakdown by client
- Comparison with previous periods

**Productivity Patterns:**
- Most productive hours
- Days of week analysis
- Trend over time

**Category Breakdown:**
- Time by project
- Time by tag
- Billable vs non-billable

### Invoice Generation

Generate invoices from tracked time:

```rust
pub struct Invoice {
    pub id: Uuid,
    pub client_id: Uuid,
    pub entries: Vec<TimeEntry>,
    pub date_range: (NaiveDate, NaiveDate),
    pub total_hours: f64,
    pub total_amount: Decimal,
    pub status: InvoiceStatus,
}

pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
}
```

Output formats:
- Markdown
- PDF (via external tool)
- CSV

### Idle Detection

Detect when user is idle:

```rust
pub struct IdleConfig {
    pub enabled: bool,
    pub threshold_mins: u32,
    pub action: IdleAction,
}

pub enum IdleAction {
    Pause,              // Pause timer
    Prompt,             // Ask what to do
    DiscardIdle,        // Discard idle time automatically
}
```

## Views

**Timer View:**
- Current timer (if running)
- Quick start buttons for recent projects
- Today's entries

**Entries View:**
- Time entries list
- Filter by date range
- Edit/delete entries

**Reports View:**
- Charts and graphs
- Date range selector
- Export options

**Projects View:**
- Project list with time totals
- Budget progress bars
- Client grouping

## Keybindings

| Key | Action |
|-----|--------|
| `s` | Start/stop timer |
| `p` | Toggle Pomodoro mode |
| `enter` | Add description |
| `t` | Assign to project |
| `j/k` | Navigate entries |
| `e` | Edit entry |
| `d` | Delete entry |
| `r` | Reports view |
| `/` | Search entries |
| `h/l` | Previous/next day |
| `w` | This week |
| `m` | This month |
| `i` | Generate invoice |
| `x` | Export |
| `q` | Quit |

## Configuration

```toml
# ~/.config/time-tracker/config.toml
[timer]
default_project = ""  # Project ID or empty

[pomodoro]
work_mins = 25
short_break_mins = 5
long_break_mins = 15
pomodoros_before_long = 4
auto_start_breaks = false
auto_start_work = false

[idle]
enabled = true
threshold_mins = 5
action = "prompt"

[display]
time_format = "24h"  # or "12h"
week_start = "monday"
show_seconds = true

[invoice]
default_currency = "USD"
template = "default"

[export]
default_format = "csv"
path = "~/time-exports"
```

## Reports

**Daily Summary:**
```
2024-01-15
─────────────────────────────────
Project A          2h 30m  ████████░░
Project B          1h 15m  ████░░░░░░
Meetings           0h 45m  ██░░░░░░░░
─────────────────────────────────
Total:             4h 30m
Billable:          3h 45m (83%)
```

**Weekly Trends:**
- Hours per day bar chart
- Comparison with previous week
- Average daily hours

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
rusqlite = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
rust_decimal = "1"
tokio = { workspace = true }
notify-rust = "4"  # Desktop notifications
```

## Database Schema

```sql
CREATE TABLE clients (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    hourly_rate TEXT,
    currency TEXT DEFAULT 'USD',
    archived INTEGER DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    client_id TEXT REFERENCES clients(id),
    name TEXT NOT NULL,
    color TEXT,
    budget_hours REAL,
    archived INTEGER DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE time_entries (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    description TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_secs INTEGER,
    billable INTEGER DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE TABLE entry_tags (
    entry_id TEXT NOT NULL REFERENCES time_entries(id),
    tag TEXT NOT NULL,
    PRIMARY KEY (entry_id, tag)
);

CREATE TABLE pomodoro_sessions (
    id TEXT PRIMARY KEY,
    entry_id TEXT REFERENCES time_entries(id),
    session_type TEXT NOT NULL,  -- work, short_break, long_break
    started_at TEXT NOT NULL,
    completed INTEGER DEFAULT 0
);

CREATE INDEX idx_entries_start ON time_entries(start_time);
CREATE INDEX idx_entries_project ON time_entries(project_id);
```
