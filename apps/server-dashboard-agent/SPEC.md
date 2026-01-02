# server-dashboard-agent

Agent-based server monitoring with central collector.

## Architecture Decisions

### Missed Heartbeat Alerting
- **Grace period + alert**: Wait 3× reporting interval before alerting
- Reduces false positives from network jitter
- Configurable grace period per server if needed

### Collector Scaling
- **Single instance**: One collector serves all agents
- Simpler deployment and operation
- SQLite or PostgreSQL storage handles most scales
- HA can be achieved at infrastructure level if needed

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Server A  │    │   Server B  │    │   Server C  │
│   [Agent]   │    │   [Agent]   │    │   [Agent]   │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │
                   ┌──────▼──────┐
                   │  Collector  │
                   │   (TCP/TLS) │
                   └──────┬──────┘
                          │
                   ┌──────▼──────┐
                   │  TUI Client │
                   └─────────────┘
```

## Components

### Agent (Lightweight Binary)

Runs on each monitored server:
- Minimal dependencies
- Low resource usage
- Configurable metrics
- Push to collector

```rust
pub struct AgentConfig {
    pub collector_url: String,
    pub auth_token: String,
    pub hostname: String,
    pub interval_secs: u64,
    pub metrics: Vec<MetricConfig>,
}

pub struct MetricConfig {
    pub name: String,
    pub enabled: bool,
    pub custom_command: Option<String>,
}
```

### Collector

Central aggregation point:
- Receives metrics from agents
- Stores in SQLite/PostgreSQL
- Provides query API
- Alerting engine

### TUI Client

Dashboard connecting to collector:
- Real-time updates
- Historical queries
- Alert management

## Metrics

**Built-in:**
- CPU, Memory, Disk, Network
- Load averages
- Process count
- Uptime

**Custom:**
- User-defined commands
- Parse output as metric
- Custom labels

```toml
# Agent custom metric example
[[metrics.custom]]
name = "nginx_connections"
command = "curl -s localhost/nginx_status | grep Active"
parse = "regex"
pattern = "Active connections: (\\d+)"
```

## Protocol

```rust
pub struct MetricPayload {
    pub hostname: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<Metric>,
}

pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
}
```

TLS-encrypted TCP or HTTPS.

## Alerting

```rust
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub server: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}
```

## Views

**Dashboard:**
- All servers status
- Aggregate metrics
- Active alerts

**Server Detail:**
- Per-server metrics
- Historical graphs
- Alert history

**Alerts:**
- Active alerts
- Alert history
- Rule management

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `g` | Toggle graphs |
| `a` | View alerts |
| `A` | Manage alert rules |
| `h` | Historical view |
| `r` | Refresh |
| `q` | Quit |

## Configuration

### Agent
```toml
# /etc/tui-monitor-agent/config.toml
collector = "https://collector.example.com:9100"
auth_token = "secret-token"
interval_secs = 30

[metrics]
cpu = true
memory = true
disk = true
network = true
```

### Collector
```toml
# ~/.config/server-dashboard-agent/collector.toml
listen = "0.0.0.0:9100"
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

[storage]
driver = "sqlite"  # or "postgres"
path = "/var/lib/tui-monitor/metrics.db"
retention_days = 90
```

## Dependencies

### Agent
```toml
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
reqwest = { workspace = true }
sysinfo = "0.32"
```

### Collector & TUI
```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
rusqlite = { workspace = true }
tokio = { workspace = true }
axum = "0.7"
```
