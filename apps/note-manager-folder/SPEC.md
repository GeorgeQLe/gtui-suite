# note-manager-folder

Traditional folder-based note organization with markdown support.

## Architecture Decisions

### Storage Format (Shared Across Note Managers)
- **Plain markdown**: Standard markdown with YAML frontmatter
- Maximum compatibility with other tools
- Core format shared with other note-manager variants
- Each variant can add extension fields to frontmatter

### Link Updates on Note Move
- **Warn + manual**: Show which notes contain [[links]] to moved note
- User reviews and decides which references to update
- Prevents silent mass-edits while providing visibility

### Note Deletion
- **Trash folder**: Move deleted notes to .trash/ subdirectory
- 30-day auto-purge of old trash
- Recovery via restore command

## Features

### Folder Hierarchy

- Nested directories for organization
- Create/rename/delete folders
- Move notes between folders
- Drag-style keyboard reordering

### Notes

- Markdown format with syntax highlighting
- YAML frontmatter for metadata
- Full-text search across all notes
- Quick capture to inbox

### Views

**Tree View (Left Pane):**
- Folder tree with expand/collapse
- Note count per folder
- Recently modified indicator

**Note List (Middle Pane):**
- Notes in current folder
- Sort by name/date/size
- Preview snippets

**Editor/Preview (Right Pane):**
- Edit mode: syntax-highlighted markdown
- Preview mode: rendered markdown
- Toggle with Tab

### Templates

Pre-defined note templates:
- Meeting notes
- Daily journal
- Project documentation
- Custom user templates

### Attachments

- Store attachments in note folder
- Reference in markdown: `![](./attachments/image.png)`
- Attachment browser per note

## Data Model

```rust
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

pub struct Frontmatter {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub created: Option<DateTime<Utc>>,
    pub template: Option<String>,
    pub custom: HashMap<String, serde_yaml::Value>,
}
```

## File Structure

```
~/.local/share/note-manager-folder/
├── notes/
│   ├── inbox/
│   │   └── quick-note.md
│   ├── work/
│   │   ├── project-a/
│   │   │   ├── meeting-2024-01-15.md
│   │   │   └── attachments/
│   │   └── project-b/
│   └── personal/
├── templates/
│   ├── meeting.md
│   └── journal.md
└── config.toml
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate notes/folders |
| `h/l` | Collapse/expand folder |
| `enter` | Open note |
| `Tab` | Toggle edit/preview |
| `n` | New note |
| `N` | New folder |
| `e` | Edit note |
| `r` | Rename |
| `d` | Delete (confirm) |
| `m` | Move to folder |
| `/` | Search |
| `t` | Add tags |
| `Ctrl+s` | Save |
| `p` | Toggle preview pane |
| `q` | Quit |

## Configuration

```toml
# ~/.config/note-manager-folder/config.toml
[storage]
notes_path = "~/.local/share/note-manager-folder/notes"
templates_path = "~/.config/note-manager-folder/templates"

[editor]
default_extension = ".md"
auto_save = true
auto_save_interval_secs = 30
spell_check = false

[display]
show_hidden = false
sort_by = "modified"  # name, modified, created
sort_order = "desc"

[search]
include_content = true
include_tags = true
fuzzy = true
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
serde_yaml = "0.9"
walkdir = "2"
notify = "6"
syntect = "5"
pulldown-cmark = "0.11"
fuzzy-matcher = { workspace = true }
```
