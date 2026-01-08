# Tier 3 - Complex Applications

Tier 3 applications are complex tools with multiple views, async operations, and substantial business logic.

## Implemented Apps

### log-viewer

A log file viewer with filtering and live tailing.

**Features:**
- Open log files
- Live tail (follow mode)
- Filter by level (error, warn, info, debug)
- Search with highlighting
- Jump to timestamp
- Wrap/unwrap long lines

**Run:** `cargo run -p log-viewer -- <logfile>`

### file-manager

A dual-pane file manager (like Midnight Commander).

**Features:**
- Two-panel view
- File operations (copy, move, delete, rename)
- Directory navigation
- File preview
- Hidden files toggle
- Sort by name/size/date

**Run:** `cargo run -p file-manager`

### docker-manager

Docker/Podman container management.

**Features:**
- List containers (running/all)
- Start/stop/restart containers
- View container logs (live streaming)
- Container details (ports, mounts, env)
- Image management
- Volume management

**Run:** `cargo run -p docker-manager`

### ssh-hub

SSH connection manager.

**Features:**
- Store SSH host profiles
- Quick connect
- Parse ~/.ssh/config
- Connection history
- Tag-based organization
- Jump host support

**Run:** `cargo run -p ssh-hub`

### backup-manager

Multi-backend backup manager.

**Features:**
- Multiple backup profiles
- rsync backend support
- Cron scheduling
- Retention policies
- Run history tracking
- Backup/restore operations

**Run:** `cargo run -p backup-manager`

### server-dashboard-ssh

Multi-server monitoring via SSH.

**Features:**
- Monitor multiple servers
- CPU, memory, disk metrics
- Process list
- Service status
- Configurable refresh interval
- Server grouping by tags

**Run:** `cargo run -p server-dashboard-ssh`

### diff-tool

File comparison tool.

**Features:**
- Side-by-side diff view
- Line-level highlighting
- Navigate between changes
- Unified diff output
- Support for various file types

**Run:** `cargo run -p diff-tool -- <file1> <file2>`

## Planned Apps (Not Yet Implemented)

- **server-dashboard-agent** - Server monitoring with agents
- **network-monitor** - Network connection monitoring

## Common Patterns

### Async Background Tasks

```rust
use tokio::sync::mpsc;

pub struct App {
    pub rx: mpsc::Receiver<Event>,
    // ...
}

// Background task sends events
tokio::spawn(async move {
    loop {
        let data = fetch_data().await;
        tx.send(Event::DataReceived(data)).await.ok();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
});
```

### Dual-Pane Layout

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(frame.area());

render_left_panel(frame, app, chunks[0]);
render_right_panel(frame, app, chunks[1]);
```

### Live Log Streaming

```rust
use futures_util::StreamExt;

async fn stream_logs(container_id: &str) -> impl Stream<Item = String> {
    docker.logs(container_id, opts)
        .map(|chunk| String::from_utf8_lossy(&chunk).to_string())
}
```

## Running Tier 3 Apps

```bash
# Build all Tier 3 apps
cargo build -p log-viewer -p file-manager -p docker-manager \
    -p ssh-hub -p backup-manager -p server-dashboard-ssh -p diff-tool

# Run specific app
cargo run -p docker-manager
```
