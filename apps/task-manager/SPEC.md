# task-manager

Kanban-style task management with boards, columns, and cards.

## Architecture Decisions

### Card Movement Between Columns
- **Match cursor position**: When moving card (H/L), insert at cursor's current row in destination column
- Natural placement based on where user is looking
- Preserves spatial context during reorganization

### WIP Limit Enforcement
- **Hard block by default**: Prevent moves to full columns with error message
- Configurable per-board in settings (can set to warning or visual-only)
- Enforces Kanban discipline while allowing flexibility

### Recurring Task Handling
- **Archive + clone**: Completed recurring task moves to archive
- New instance appears in starting column (Backlog) with next due date
- Maintains history while keeping board clean

### Calendar View Undated Tasks
- **Separate section**: Show undated tasks in 'Unscheduled' section below calendar
- Keeps calendar focused on scheduled work
- Easy access to backlog items

## Features

### Kanban Board

**Board Structure:**
- Multiple boards (work, personal, projects)
- Customizable columns per board
- Cards flow between columns

**Cards:**
- Title (required)
- Description (markdown)
- Due date
- Priority (low, medium, high, urgent)
- Tags/labels with colors
- Checklists
- Attachments (file references)

### Quick Capture

Global quick-add (via shell integration):
- Add task without opening full app
- Auto-assign to inbox/default column

### Views

**Board View:**
- Columns side by side
- Cards stacked in columns
- Scrollable columns
- Visual priority indicators

**List View:**
- All tasks in a table
- Sort by any field
- Filter by column/tag/date

**Calendar View:**
- Tasks on due dates
- Month/week views

### Recurring Tasks

```rust
pub struct Recurrence {
    pub pattern: RecurrencePattern,
    pub end_condition: EndCondition,
}

pub enum RecurrencePattern {
    Daily,
    Weekly { days: Vec<Weekday> },
    Monthly { day: u8 },
    Yearly { month: u8, day: u8 },
}
```

## Data Model

```rust
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub columns: Vec<Column>,
    pub created_at: DateTime<Utc>,
    pub archived: bool,
}

pub struct Column {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub position: i32,
    pub wip_limit: Option<usize>,  // Work-in-progress limit
    pub color: Option<Color>,
}

pub struct Task {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub tags: Vec<Tag>,
    pub checklist: Vec<ChecklistItem>,
    pub recurrence: Option<Recurrence>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub archived: bool,
}

pub struct Tag {
    pub name: String,
    pub color: Color,
}

pub struct ChecklistItem {
    pub text: String,
    pub completed: bool,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Move down/up in column |
| `h/l` | Move to prev/next column |
| `J/K` | Move card down/up |
| `H/L` | Move card to prev/next column |
| `enter` | Open card details |
| `a` | Add new card |
| `A` | Add new column |
| `e` | Edit card |
| `d` | Delete card (confirm) |
| `space` | Toggle first checklist item |
| `c` | Complete task (move to done) |
| `p` | Cycle priority |
| `t` | Add/edit tags |
| `/` | Search tasks |
| `f` | Filter tasks |
| `1-9` | Jump to column N |
| `q` | Quit |

## Configuration

```toml
# ~/.config/task-manager/config.toml
[board]
default = "personal"
archive_completed_after_days = 7

[display]
show_due_dates = true
show_tags = true
show_priority_colors = true
cards_per_column = 10

[columns.defaults]
names = ["Backlog", "To Do", "In Progress", "Done"]

[quick_capture]
default_column = "Backlog"
default_priority = "medium"
```

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
CREATE TABLE boards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    archived INTEGER DEFAULT 0
);

CREATE TABLE columns (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id),
    name TEXT NOT NULL,
    position INTEGER NOT NULL,
    wip_limit INTEGER,
    color TEXT
);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    column_id TEXT NOT NULL REFERENCES columns(id),
    title TEXT NOT NULL,
    description TEXT,
    position INTEGER NOT NULL,
    priority TEXT NOT NULL,
    due_date TEXT,
    recurrence TEXT,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    archived INTEGER DEFAULT 0
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    name TEXT NOT NULL,
    color TEXT,
    PRIMARY KEY (task_id, name)
);

CREATE TABLE checklist_items (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    text TEXT NOT NULL,
    completed INTEGER DEFAULT 0,
    position INTEGER NOT NULL
);

CREATE INDEX idx_tasks_column ON tasks(column_id);
CREATE INDEX idx_tasks_due ON tasks(due_date);
```
