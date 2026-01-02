# cheatsheet-browser

Search and browse command cheat sheets from multiple sources.

## Architecture Decisions

### Syntax Highlighting
- **Mapped themes**: Map tui-theme to corresponding syntect theme
- Each tui-theme preset maps to a syntect theme (e.g., Catppuccin -> catppuccin, Nord -> nord)
- Fallback to monokai if no mapping exists
- User can override with explicit syntect theme in config

### Edit Mode
- **Full editor**: Built-in markdown editor with preview for user cheat sheets
- Split view: editor on left, rendered preview on right
- Basic markdown support: headers, code blocks, lists
- Hot-reload preview as user types

## Features

### Unified Search

Search across all sources simultaneously:
- Bundled curated cheat sheets
- cheat.sh API (remote, cached)
- User-defined markdown files

Fuzzy matching for commands and descriptions.

### Sources

**Bundled:**
- Pre-packaged cheat sheets for common tools
- Git, Docker, Kubernetes, vim, tmux, etc.
- Stored as embedded markdown

**cheat.sh Integration:**
```rust
pub struct CheatShClient {
    cache: Cache,
    base_url: String,
}

impl CheatShClient {
    pub async fn query(&self, topic: &str) -> Result<CheatSheet>;
    pub async fn list_topics(&self) -> Result<Vec<String>>;
}
```

**User-Defined:**
- Markdown files in `~/.config/cheatsheets/`
- Support for subdirectories (categories)
- Hot-reload on file changes

### Display

**List View:**
- Topic list with categories
- Search filter
- Source indicator (bundled/remote/user)

**Detail View:**
- Syntax-highlighted code blocks
- Scrollable content
- Section navigation

### Features

**Copy to Clipboard:**
- Copy entire cheat sheet
- Copy selected code block
- Copy single line

**Offline Mode:**
- Falls back to bundled + cached
- Graceful degradation
- Cache persistence

**Syntax Highlighting:**
- Language detection from code fences
- Common languages supported
- Theme-aware colors

## Data Model

```rust
pub struct CheatSheet {
    pub topic: String,
    pub source: Source,
    pub content: String,
    pub sections: Vec<Section>,
    pub language: Option<String>,
    pub cached_at: Option<DateTime<Utc>>,
}

pub enum Source {
    Bundled,
    Remote { url: String },
    User { path: PathBuf },
}

pub struct Section {
    pub title: String,
    pub content: String,
    pub code_blocks: Vec<CodeBlock>,
}

pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
    pub line_start: usize,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate list |
| `enter` | View cheat sheet |
| `/` | Search |
| `esc` | Clear search / back |
| `y` | Copy current block |
| `Y` | Copy entire sheet |
| `r` | Refresh from remote |
| `o` | Open in $EDITOR |
| `n` | Next section |
| `p` | Previous section |
| `g` | Go to top |
| `G` | Go to bottom |
| `q` | Quit |

## Configuration

```toml
# ~/.config/cheatsheet-browser/config.toml
[sources]
bundled = true
cheat_sh = true
user_path = "~/.config/cheatsheets"

[cache]
enabled = true
ttl_hours = 168  # 1 week
max_size_mb = 50

[display]
syntax_theme = "monokai"
wrap_lines = true
show_line_numbers = true

[network]
timeout_secs = 10
proxy = ""  # Optional HTTP proxy
```

## Bundled Cheat Sheets

Initial set to include:
- Shell: bash, zsh, fish
- Version Control: git
- Containers: docker, podman, kubectl
- Editors: vim, nvim, emacs
- Multiplexers: tmux, screen
- Languages: rust, python, go, javascript
- Tools: curl, jq, sed, awk, grep, find
- Package managers: apt, pacman, brew, cargo, npm

## Cache Management

```rust
pub struct Cache {
    path: PathBuf,
    max_size: u64,
    ttl: Duration,
}

impl Cache {
    pub fn get(&self, key: &str) -> Option<CachedSheet>;
    pub fn set(&self, key: &str, sheet: &CheatSheet);
    pub fn cleanup(&self);  // Remove expired entries
    pub fn clear(&self);
}
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
reqwest = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
fuzzy-matcher = { workspace = true }
syntect = "5"
copypasta = "0.10"
directories = "5"
```
