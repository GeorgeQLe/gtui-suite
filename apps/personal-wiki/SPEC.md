# personal-wiki

Wiki-style knowledge base with namespaces, history, and export.

## Architecture Decisions

### History Diff Viewer
- **Adaptive level**: Word-level diff for prose content
- Line-level diff for code blocks and config sections
- Auto-detect content type within document

### Dead Link Page Creation
- **Prompt user**: Ask which namespace when creating from dead link
- Pre-select referring page's namespace as default
- Explicit [[NS/Page]] syntax always respects specified namespace

### Static Site Export Search
- **Client-side search**: Bundle search index (JSON) and JavaScript
- Works offline after export
- Uses lunr.js or similar lightweight search library

### Deletion & Trash
- **30-day trash**: Deleted pages move to .trash/ folder
- Auto-purge after 30 days (configurable)
- Consistent with note-manager apps
- Recoverable deletion protects against accidents

## Features

### Wiki Pages

- Markdown with [[wiki-links]]
- Hierarchical namespaces: `[[Namespace/Page]]`
- Automatic table of contents
- Categories and cross-references

### Namespaces

Organize pages hierarchically:
```
Main/
├── Projects/
│   ├── Alpha
│   └── Beta
├── People/
│   └── John Doe
└── Topics/
    ├── Programming
    └── Design
```

### Page History

Track all changes:
- Git-backed or internal versioning
- View diff between versions
- Restore previous versions
- Blame view (who changed what)

### Templates

Page templates for consistency:
- Person template
- Project template
- Meeting notes
- Custom templates

### Categories

Organize pages by topic:
- Add categories in frontmatter
- Category index pages
- Multi-category assignment

### Search

Full-text search with filtering:
- Search titles
- Search content
- Filter by namespace
- Filter by category

### Export

Generate static site:
- Markdown to HTML
- Index pages
- Search functionality
- Configurable theme

## Data Model

```rust
pub struct WikiPage {
    pub path: String,  // "Namespace/Page"
    pub title: String,
    pub content: String,
    pub categories: Vec<String>,
    pub links: Vec<String>,
    pub backlinks: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub author: String,
}

pub struct PageVersion {
    pub page_path: String,
    pub version: i32,
    pub content: String,
    pub author: String,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate pages |
| `enter` | Open page |
| `Tab` | Toggle edit/preview |
| `n` | New page |
| `e` | Edit page |
| `r` | Rename/move page |
| `d` | Delete page |
| `h` | History |
| `b` | Backlinks |
| `c` | Categories |
| `[[` | Insert link |
| `/` | Search |
| `Ctrl+]` | Follow link |
| `Ctrl+o` | Go back |
| `x` | Export |
| `q` | Quit |

## Configuration

```toml
# ~/.config/personal-wiki/config.toml
[storage]
path = "~/.local/share/personal-wiki"
use_git = true

[pages]
default_namespace = "Main"
home_page = "Main/Home"

[templates]
path = "~/.config/personal-wiki/templates"

[export]
output_path = "~/wiki-export"
format = "html"
theme = "default"

[display]
show_backlinks = true
show_categories = true
toc_depth = 3
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
git2 = { workspace = true }
pulldown-cmark = "0.11"
regex = "1"
handlebars = "6"
```
