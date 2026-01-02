# server-dashboard-ssh

Multi-server monitoring dashboard via SSH.

## Architecture Decisions

### Failed SSH Connection Display
- **Show stale + badge**: Display last known metrics with 'stale' indicator
- Include timestamp of last successful collection
- Preserves context while clearly showing current state
- Separate visual treatment for unreachable servers

## Features

### SSH-Based Collection

Connect to servers and run commands:
- Reuse ssh-agent credentials
- Respect ~/.ssh/config
- Concurrent connections
- Reconnection on failure

### Metrics Collected

**CPU:**
- Usage percentage
- Load averages
- Per-core stats

**Memory:**
- Used/available/total
- Swap usage
- Buffer/cache

**Disk:**
- Usage per mount
- I/O stats
- Alerts on low space

**Network:**
- Interface stats
- Bandwidth in/out

**Services:**
- systemd unit status
- Custom service checks

### Aggregate Dashboard

```
┌────────────────────────────────────────────────────────────┐
│  Server Dashboard                         Updated: 12:34:56 │
├────────────────────────────────────────────────────────────┤
│  Server       CPU   Mem    Disk   Load   Status            │
│  ─────────────────────────────────────────────────────────  │
│  web-01       45%   2.1G   67%    1.23   ● OK              │
│  web-02       52%   1.8G   65%    0.98   ● OK              │
│  db-01        23%   8.2G   45%    0.45   ● OK              │
│  cache-01     78%   3.9G   12%    2.34   ⚠ HIGH CPU        │
└────────────────────────────────────────────────────────────┘
```

### Per-Server Detail

- Real-time graphs
- Process top N
- Service status
- Recent events

### Alerting

```rust
pub struct AlertRule {
    pub name: String,
    pub metric: MetricType,
    pub condition: Condition,
    pub threshold: f64,
    pub duration: Duration,
    pub severity: Severity,
}

pub enum Condition {
    GreaterThan,
    LessThan,
    Equals,
}
```

### Historical Data

- Store metrics in SQLite
- Configurable retention
- Historical graphs

## Data Model

```rust
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub user: Option<String>,
    pub port: u16,
    pub tags: Vec<String>,
    pub enabled: bool,
}

pub struct ServerMetrics {
    pub server_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub load_1: f32,
    pub load_5: f32,
    pub load_15: f32,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate servers |
| `enter` | View details |
| `g` | Toggle graphs |
| `a` | Add server |
| `e` | Edit server |
| `d` | Delete server |
| `t` | Filter by tag |
| `r` | Refresh now |
| `Space` | Pause updates |
| `A` | Alert rules |
| `h` | Historical view |
| `q` | Quit |

## Configuration

```toml
# ~/.config/server-dashboard-ssh/config.toml
[collection]
interval_secs = 30
concurrent_connections = 10
timeout_secs = 10

[storage]
retention_days = 30

[alerts]
notify_cmd = "notify-send"
email = ""  # Optional email address

[display]
show_graphs = true
graph_width = 20
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
ssh2 = "0.9"
```
