# note-manager-daily

Journal-first note system with daily notes and wiki-style linking.

## Architecture Decisions

### Storage Format (Shared Across Note Managers)
- **Plain markdown**: Standard markdown with YAML frontmatter
- Date-based file organization (YYYY/MM/YYYY-MM-DD.md)
- Core format shared with other note-manager variants

### Dead Link Handling
- **Prompt user**: When following [[link]] to non-existent concept note
- Ask "Create note 'X'?" with options to create or cancel
- Allows quick creation without accidental note proliferation

### Weekly/Monthly Task Rollup
- **Highlights only**: Only show tasks explicitly marked important
- Use special syntax (e.g., `- [!]` or frontmatter flag)
- Keeps rollups focused on key items, not overwhelmed by minutiae

### Note Deletion
- **Trash folder**: Move deleted notes to .trash/ subdirectory
- 30-day auto-purge, recovery available

## Features

### Daily Notes

- Auto-create today's note on launch
- Template for daily entries
- Navigate by calendar
- Quick jump to any date

### Daily Structure

Each day gets a note:
```markdown
# 2024-01-15

## Morning
- [ ] Task 1
- [ ] Task 2

## Work
Discussed [[Project Alpha]] with team.
Need to follow up on [[Meeting Notes/2024-01-14]].

## Evening
Read about [[Topic X]].

## Reflections
...
```

### Concept Notes

- Linked from daily notes via `[[concept]]`
- Build knowledge over time
- See all days referencing a concept

### Views

**Calendar View:**
- Month grid with note indicators
- Navigate months
- Quick jump to day

**Timeline View:**
- Chronological scroll
- Aggregate by week/month

**Concept View:**
- List of concept notes
- Backlinks from daily notes

### Weekly/Monthly Rollups

Aggregate view across time periods:
- Weekly summary
- Monthly review
- Custom date ranges

## Data Model

```rust
pub struct DailyNote {
    pub date: NaiveDate,
    pub content: String,
    pub links: Vec<String>,
    pub tasks: Vec<Task>,
    pub word_count: usize,
}

pub struct ConceptNote {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub mentioned_on: Vec<NaiveDate>,  // Days that mention this
}

pub struct Task {
    pub text: String,
    pub completed: bool,
    pub due_date: Option<NaiveDate>,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `t` | Go to today |
| `h/l` | Previous/next day |
| `j/k` | Scroll in note |
| `[/]` | Previous/next week |
| `{/}` | Previous/next month |
| `c` | Calendar view |
| `enter` | Edit current day |
| `[[` | Insert link |
| `space` | Toggle task under cursor |
| `/` | Search all notes |
| `g` | Go to date (prompt) |
| `w` | Weekly rollup |
| `m` | Monthly rollup |
| `n` | New concept note |
| `q` | Quit |

## Configuration

```toml
# ~/.config/note-manager-daily/config.toml
[daily]
template = """
# {{date}}

## Morning

## Work

## Evening

## Reflections

"""
auto_create = true
week_start = "monday"

[display]
date_format = "%Y-%m-%d"
show_word_count = true
show_task_count = true

[rollup]
weekly_on = "sunday"  # Day to show weekly prompt
monthly_on = 1        # Day of month

[storage]
path = "~/.local/share/note-manager-daily"
daily_folder = "daily"
concepts_folder = "concepts"
```

## File Structure

```
~/.local/share/note-manager-daily/
├── daily/
│   ├── 2024/
│   │   ├── 01/
│   │   │   ├── 2024-01-15.md
│   │   │   └── 2024-01-16.md
│   │   └── 02/
│   └── 2023/
├── concepts/
│   ├── project-alpha.md
│   └── topic-x.md
└── templates/
    ├── daily.md
    └── weekly.md
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
pulldown-cmark = "0.11"
regex = "1"
handlebars = "6"  # Template rendering
```
