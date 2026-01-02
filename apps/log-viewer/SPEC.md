# log-viewer

Multi-file log viewer with structured parsing and remote streaming.

## Architecture Decisions

### Missing Timestamp Handling
- **Attach to next**: Lines without timestamps group with following timestamped line
- Treats continuation lines naturally (stack traces, multi-line messages)
- Follows typical log semantics where header precedes body

### SSH Connection Loss Buffering
- **Buffer locally**: Continue receiving lines during disconnect
- Reconnect and resume seamlessly
- No data loss during viewing session
- Buffer size configurable to prevent unbounded growth

## Features

### Single File Mode

- Tail file (follow new content)
- Scroll through history
- Search with regex
- Severity-based coloring

### Multi-File Correlation

**Timestamp Synchronization:**
- Auto-detect timestamp format
- Align multiple files by time
- Merged timeline view
- Per-file color coding

**Split View:**
- Side-by-side files
- Synchronized scrolling
- Jump to same timestamp

### Structured Log Parsing

**JSON Logs:**
```rust
pub struct JsonLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}
```

**Logfmt:**
```
level=info msg="request handled" duration=120ms user=john
```

**Field-Based Filtering:**
- Filter by field values
- Aggregate by field
- Show/hide fields

### Remote Streaming

**SSH Mode:**
- Connect to remote server
- Tail remote files
- Multiple remotes simultaneously

**Configuration:**
```rust
pub struct RemoteConfig {
    pub host: String,
    pub user: String,
    pub key_path: Option<PathBuf>,
    pub files: Vec<String>,
}
```

### Features

**Bookmarks:**
- Mark interesting lines
- Navigate between bookmarks
- Export bookmarked lines

**Time Range:**
- Filter by time window
- Zoom in/out on timeline

**Severity Filtering:**
- Filter by log level
- Highlight errors/warnings

## Views

**Follow Mode:**
- Auto-scroll to bottom
- Pause on scroll up
- Resume on key press

**Search Mode:**
- Regex search
- Highlight all matches
- Navigate between matches

**Field View:**
- Table of structured fields
- Sort by column
- Aggregate counts

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Scroll down/up |
| `J/K` | Page down/up |
| `g/G` | Top/bottom |
| `f` | Toggle follow mode |
| `/` | Search |
| `n/N` | Next/previous match |
| `b` | Add bookmark |
| `B` | List bookmarks |
| `'` | Jump to bookmark |
| `l` | Filter by level |
| `t` | Time range filter |
| `Tab` | Switch files (multi) |
| `s` | Open structured view |
| `c` | Clear filters |
| `q` | Quit |

## Configuration

```toml
# ~/.config/log-viewer/config.toml
[display]
line_numbers = true
wrap_lines = false
timestamp_format = "auto"

[colors]
error = "red"
warn = "yellow"
info = "default"
debug = "dim"
trace = "dim"

[parsing]
detect_format = true  # JSON, logfmt, plain
json_timestamp_field = "timestamp"
json_message_field = "message"
json_level_field = "level"

[remote]
ssh_timeout_secs = 10
reconnect_on_failure = true

[files]
# Preset file paths
nginx = "/var/log/nginx/access.log"
syslog = "/var/log/syslog"
```

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
regex = "1"
notify = "6"
tokio = { workspace = true }
ssh2 = "0.9"
```
