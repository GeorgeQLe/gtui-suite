# kanban-standalone

Local Kanban board with team sync capabilities.

## Architecture Decisions

### Sync Conflict Resolution
- **Field-level merge**: Merge at field granularity
- If user A edits title and user B edits description, keep both changes
- Minimizes data loss from concurrent edits
- Show merge notification to users

### Board Authentication
- **Optional auth**: Boards can be public (link-only) or private (auth required)
- Default to private for security
- Clear UI indication when making board public
- Supports both quick sharing and secure collaboration

## Features

### Board Management

**Multiple Boards:**
- Personal, work, project-specific
- Archive old boards
- Board templates

**Columns:**
- Customizable stages
- WIP limits
- Color coding

### Cards

**Content:**
- Title (required)
- Description (markdown)
- Due date and reminders
- Priority (urgent, high, medium, low)
- Labels/tags with colors
- Checklists
- Attachments (file references)
- Comments

**Actions:**
- Drag-style movement (keyboard)
- Quick edit
- Duplicate
- Archive

### Views

**Board View:**
- Columns side by side
- Card previews
- Visual indicators

**Calendar View:**
- Cards by due date
- Overdue highlighting

**List View:**
- All cards as table
- Sort and filter

### Team Sync

**Local-First:**
- Works offline
- Sync when connected

**Sync Protocol:**
```rust
pub struct SyncMessage {
    pub message_id: Uuid,
    pub board_id: Uuid,
    pub operation: Operation,
    pub timestamp: DateTime<Utc>,
    pub author: String,
}

pub enum Operation {
    CreateCard(Card),
    UpdateCard { id: Uuid, changes: CardPatch },
    DeleteCard(Uuid),
    MoveCard { id: Uuid, column: Uuid, position: i32 },
    CreateColumn(Column),
    // ...
}
```

**Conflict Resolution:**
- Last-write-wins for simple fields
- Merge checklists/comments
- User notification for conflicts

**Server:**
- Simple WebSocket server
- SQLite storage
- User authentication

## Data Model

```rust
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<Column>,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
}

pub struct Column {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub wip_limit: Option<u32>,
    pub color: Option<String>,
}

pub struct Card {
    pub id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub labels: Vec<Label>,
    pub checklist: Vec<ChecklistItem>,
    pub comments: Vec<Comment>,
    pub attachments: Vec<Attachment>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Comment {
    pub id: Uuid,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Move down/up in column |
| `h/l` | Previous/next column |
| `J/K` | Move card down/up |
| `H/L` | Move card to prev/next column |
| `enter` | Open card details |
| `a` | Add new card |
| `A` | Add new column |
| `e` | Edit card |
| `d` | Delete card |
| `c` | Add comment |
| `space` | Toggle checklist item |
| `p` | Change priority |
| `t` | Edit labels |
| `/` | Search |
| `f` | Filter |
| `Tab` | Switch view |
| `S` | Sync now |
| `q` | Quit |

## Configuration

```toml
# ~/.config/kanban-standalone/config.toml
[board]
default = "personal"

[sync]
enabled = false
server = "ws://localhost:8080"
username = ""
auto_sync_interval_secs = 30

[display]
show_due_dates = true
show_labels = true
cards_visible = 5
```

## Sync Server

Separate binary for hosting:

```toml
# /etc/kanban-server/config.toml
listen = "0.0.0.0:8080"
database = "/var/lib/kanban-server/data.db"

[auth]
method = "token"  # or "password"
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
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
tokio-tungstenite = "0.24"
```
