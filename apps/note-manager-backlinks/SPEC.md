# note-manager-backlinks

Flat note structure with wiki-style [[backlinks]] for emergent organization.

## Architecture Decisions

### Storage Format (Shared Across Note Managers)
- **Plain markdown**: Standard markdown with YAML frontmatter
- SQLite for link index and metadata (fast graph queries)
- Core format shared with other note-manager variants

### Graph View Navigation
- **Multiple view types**: All navigation modes available
  - Ego-centric: Center on current note, show N levels outward
  - Zoom + pan: Keyboard controls for full graph exploration
  - Progressive disclosure: Start collapsed, expand on navigation
  - Filtered subgraph: Show only notes matching search/tag filter
- User can switch between modes with keybindings

### Rename Operation
- **Progress + summary**: Show each file being updated in real-time
- Summary at end lists any failures
- Rollback on critical failures, continue on minor issues

### Note Deletion
- **Trash folder**: Move deleted notes to .trash/ subdirectory
- 30-day auto-purge, recovery available
- Warn if note has backlinks before deletion

## Features

### Wiki-Style Links

- `[[note-name]]` creates link to note
- Auto-complete while typing links
- Create note from link if doesn't exist
- Rename updates all references

### Backlinks

Automatic backlink detection:
- View all notes that link to current note
- Context preview (surrounding paragraph)
- Navigate to linking note

### Unlinked Mentions

Find notes that mention current note's title without linking:
- Suggest converting to links
- Discover hidden connections

### Graph View

ASCII-art visualization of note connections:
```
    [Note A]
      │
      ├──→ [Note B] ←── [Note D]
      │        │
      └──→ [Note C]
```

### Tags

- Inline tags: `#tag-name`
- Tag search and filtering
- Tag cloud view

### Features

**Daily Notes:**
- Optional daily note creation
- Template for daily entries
- Quick link to today

**Full-Text Search:**
- Search content and titles
- Show match context
- Navigate to match location

## Data Model

```rust
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub links: Vec<String>,      // Outgoing [[links]]
    pub backlinks: Vec<Uuid>,    // Computed: notes linking here
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

pub struct LinkContext {
    pub source_id: Uuid,
    pub target_title: String,
    pub context: String,  // Surrounding text
    pub position: usize,  // Character position
}
```

## Storage

SQLite + files hybrid:
- Note content in markdown files
- Links, backlinks, metadata in SQLite
- Fast graph queries

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate notes |
| `enter` | Open note |
| `[[` | Start link (auto-complete) |
| `Tab` | Toggle edit/preview |
| `n` | New note |
| `b` | View backlinks |
| `g` | Graph view |
| `t` | Today's daily note |
| `#` | Insert tag |
| `/` | Search |
| `Ctrl+]` | Follow link under cursor |
| `Ctrl+o` | Go back |
| `r` | Rename note (updates links) |
| `d` | Delete (warn if has backlinks) |
| `q` | Quit |

## Configuration

```toml
# ~/.config/note-manager-backlinks/config.toml
[storage]
notes_path = "~/.local/share/note-manager-backlinks"

[links]
auto_create = true  # Create note when following dead link
case_sensitive = false
update_on_rename = true

[daily_notes]
enabled = true
folder = "daily"
template = "daily.md"
format = "%Y-%m-%d"

[graph]
max_depth = 3
show_orphans = true
cluster_by_tags = false

[display]
show_backlink_count = true
show_tags_inline = true
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
regex = "1"
pulldown-cmark = "0.11"
fuzzy-matcher = { workspace = true }
petgraph = "0.6"  # For graph operations
```

## Database Schema

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL UNIQUE,
    file_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

CREATE TABLE links (
    source_id TEXT NOT NULL REFERENCES notes(id),
    target_title TEXT NOT NULL,  -- Title, not ID (may not exist yet)
    context TEXT,
    position INTEGER,
    PRIMARY KEY (source_id, target_title, position)
);

CREATE TABLE tags (
    note_id TEXT NOT NULL REFERENCES notes(id),
    tag TEXT NOT NULL,
    PRIMARY KEY (note_id, tag)
);

CREATE INDEX idx_links_target ON links(target_title);
CREATE INDEX idx_tags_tag ON tags(tag);

-- View for resolved backlinks
CREATE VIEW backlinks AS
SELECT
    n.id as target_id,
    l.source_id,
    l.context
FROM notes n
JOIN links l ON lower(l.target_title) = lower(n.title);
```
