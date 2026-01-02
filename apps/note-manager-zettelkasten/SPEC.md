# note-manager-zettelkasten

Luhmann-style Zettelkasten with numeric IDs and explicit linking.

## Architecture Decisions

### Storage Format (Shared Across Note Managers)
- **Plain markdown**: Standard markdown with YAML frontmatter
- SQLite for note index, links, and sequence relationships
- Core format shared with other note-manager variants

### Link Suggestions During Promotion
- **Background index**: Analyze note content and build suggestions silently
- Show potential related notes in sidebar during promotion
- Non-intrusive: respects Zettelkasten manual linking philosophy
- User consciously decides each link

### Sequence Deletion
- **Prompt choice**: When deleting note with children in sequence
- Options: orphan children, reparent to grandparent, delete entire branch
- Prevents accidental loss of connected notes

### Note Deletion
- **Trash folder**: Move deleted notes to .trash/ subdirectory
- 30-day auto-purge, recovery available

## Features

### Numeric ID System

Each note has a unique ID:
- Timestamp-based: `202401151423` (YYYYMMDDHHmm)
- Or sequential with branches: `1`, `1a`, `1a1`, `1b`, `2`

### Note Types

**Fleeting Notes:**
- Quick capture
- Temporary, to be processed
- No ID until promoted

**Literature Notes:**
- Reference to source material
- Bibliographic info
- Key quotes and ideas

**Permanent Notes:**
- Atomic ideas
- Single concept per note
- Linked to other permanent notes

### Linking

Explicit links with IDs:
```markdown
# 202401151423 Concept Title

This relates to [[202401141230]] and builds on [[202401101015]].

See also: [[202401151445]]
```

### Sequence Indicators

Notes can form sequences:
- `1` → `1a` → `1a1` → `1a2`
- `1` → `1b`
- Visualize as tree

### Index Notes

Hub notes that organize topics:
- Link to key notes on a topic
- Provide entry points
- Structure without hierarchy

### Reference Management

Track source materials:
```rust
pub struct Reference {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub source_type: SourceType,  // Book, Article, Web, etc.
    pub url: Option<String>,
    pub notes: Vec<NoteId>,  // Literature notes about this
}
```

## Data Model

```rust
pub struct Note {
    pub id: String,  // Timestamp or sequential ID
    pub note_type: NoteType,
    pub title: String,
    pub content: String,
    pub links: Vec<String>,      // Outgoing links (IDs)
    pub tags: Vec<String>,
    pub reference_id: Option<Uuid>,  // For literature notes
    pub sequence_parent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

pub enum NoteType {
    Fleeting,
    Literature,
    Permanent,
    Index,
}
```

## Views

**Inbox View:**
- Fleeting notes to process
- Promote to permanent/literature

**Browse View:**
- All permanent notes
- Filter by tag
- Sort by ID/date/title

**Sequence View:**
- Tree of note sequences
- Navigate branches

**Reference View:**
- List of sources
- Literature notes per source

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate notes |
| `enter` | Open note |
| `n` | New fleeting note |
| `N` | New permanent note (with ID) |
| `L` | New literature note |
| `I` | New index note |
| `p` | Promote fleeting to permanent |
| `[[` | Insert link (ID autocomplete) |
| `b` | View backlinks |
| `s` | Sequence view |
| `r` | References view |
| `/` | Search |
| `t` | Add tags |
| `q` | Quit |

## Configuration

```toml
# ~/.config/note-manager-zettelkasten/config.toml
[ids]
format = "timestamp"  # or "sequential"
timestamp_format = "%Y%m%d%H%M"

[storage]
path = "~/.local/share/note-manager-zettelkasten"

[note_types]
fleeting_folder = "fleeting"
literature_folder = "literature"
permanent_folder = "permanent"
index_folder = "index"

[workflow]
auto_archive_fleeting_days = 7
prompt_promote_on_open = true
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
regex = "1"
```

## Database Schema

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,  -- The zettel ID
    note_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    reference_id TEXT,
    sequence_parent TEXT,
    created_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

CREATE TABLE links (
    source_id TEXT NOT NULL REFERENCES notes(id),
    target_id TEXT NOT NULL,
    PRIMARY KEY (source_id, target_id)
);

CREATE TABLE tags (
    note_id TEXT NOT NULL REFERENCES notes(id),
    tag TEXT NOT NULL,
    PRIMARY KEY (note_id, tag)
);

CREATE TABLE references (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    authors TEXT,
    year INTEGER,
    source_type TEXT,
    url TEXT
);

CREATE INDEX idx_notes_type ON notes(note_type);
CREATE INDEX idx_links_target ON links(target_id);
CREATE INDEX idx_notes_parent ON notes(sequence_parent);
```
