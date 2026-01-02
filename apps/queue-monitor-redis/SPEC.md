# queue-monitor-redis

Redis Streams and Pub/Sub monitoring.

## Architecture Decisions

### Message Preview Display
- **Truncate + expand**: Show first N bytes inline, press key to see full content in modal
- Smart truncation at natural JSON boundaries with clear "[+2.3KB more...]" indicator
- Handles huge payloads without overwhelming display

### Connection Failure Recovery
- **Auto-reconnect with backoff**: Exponential backoff (1s→2s→4s→max 60s)
- Status bar shows connection state with reconnect count
- After N consecutive failures, prompt user rather than continuing silently

### Dangerous Operation Protection
- **Double confirmation + count**: First confirm shows impact (stream length, key count), second requires typing resource name
- Quick-delete keybind for empty streams or keys under threshold
- Prevents accidental data loss while allowing fast operations on safe targets

## Features

### Redis Connection

- Single instance or cluster
- Sentinel support
- Authentication
- TLS

### Redis Streams

**Stream List:**
- All streams
- Entry count (XLEN)
- First/last entry ID
- Memory usage

**Stream Details:**
- Consumer groups
- Pending entries
- Stream info

**Consumer Groups:**
- Group list per stream
- Pending count
- Last delivered ID
- Consumers in group

**Operations:**
- XADD (add entries)
- XREAD (read entries)
- XRANGE (range query)
- XTRIM (trim stream)
- XACK (acknowledge)

### Pub/Sub

**Channel Monitoring:**
- Active channels
- Subscriber counts
- Message flow (sampled)

**Pattern Subscriptions:**
- Monitor patterns
- Message preview

### Key Browser

**Browse All Keys:**
- Key list with types
- TTL display
- Memory usage
- Pattern filter

**Key Operations:**
- GET/SET for strings
- Inspect lists, sets, hashes
- Delete keys

### Memory Analysis

- Memory usage by key pattern
- Big keys detection
- Memory stats

## Data Model

```rust
pub struct Stream {
    pub name: String,
    pub length: u64,
    pub first_entry_id: String,
    pub last_entry_id: String,
    pub groups: Vec<ConsumerGroup>,
}

pub struct ConsumerGroup {
    pub name: String,
    pub pending: u64,
    pub last_delivered_id: String,
    pub consumers: Vec<Consumer>,
}

pub struct Consumer {
    pub name: String,
    pub pending: u64,
    pub idle: Duration,
}

pub struct StreamEntry {
    pub id: String,
    pub fields: HashMap<String, String>,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch views |
| `j/k` | Navigate |
| `enter` | View details |
| `s` | Streams view |
| `p` | Pub/Sub view |
| `k` | Keys view |
| `a` | XADD entry |
| `g` | Consumer groups |
| `A` | Acknowledge pending |
| `t` | Trim stream |
| `r` | Refresh |
| `R` | Auto-refresh |
| `/` | Search |
| `q` | Quit |

## Configuration

```toml
# ~/.config/queue-monitor-redis/config.toml
[redis]
url = "redis://localhost:6379"
# password = ""
# database = 0

# For cluster
# cluster = true
# nodes = ["redis://node1:6379", "redis://node2:6379"]

[display]
refresh_secs = 5
max_keys_display = 1000
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
tokio = { workspace = true }
redis = { version = "0.27", features = ["tokio-comp", "cluster"] }
```
