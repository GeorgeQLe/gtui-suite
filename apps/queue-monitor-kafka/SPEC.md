# queue-monitor-kafka

Kafka cluster monitoring and management.

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
- **Double confirmation + count**: First confirm shows impact ("12,345 messages"), second requires typing topic name
- Quick-delete keybind for empty topics or topics under threshold
- Prevents accidental data loss while allowing fast operations on safe targets

### Consumer Lag Alerting
- **Time-based threshold**: Alert when lag exceeds N minutes of messages (calculated from publish rate)
- Default: 5 minutes, user configurable per consumer group
- Shows message count alongside time estimate in UI for context
- More meaningful than raw message count regardless of throughput

## Features

### Admin Client

Connect to Kafka cluster:
- Bootstrap servers
- SASL authentication
- TLS support

### Views

**Topics:**
- Topic list
- Partition count
- Replication factor
- Message count (log end offset)
- Retention settings

**Partitions:**
- Per-topic partitions
- Leader broker
- Replica assignment
- ISR (in-sync replicas)
- Offset (high/low watermark)

**Consumer Groups:**
- Group list
- Member count
- State (stable, rebalancing)
- Lag per partition

**Brokers:**
- Broker list
- Host:port
- Rack ID
- Controller status

### Consumer Lag

**Per Consumer Group:**
- Topic subscriptions
- Partition assignment
- Current offset
- Log end offset
- Lag

**Visualization:**
- Lag over time
- Alert on high lag

### Operations

**Topic Management:**
- Create topic
- Delete topic
- Add partitions
- Modify configs

**Consumer Groups:**
- Reset offsets
- Delete group
- Describe members

**Produce Test:**
- Send message to topic
- Key and value
- Headers

**Consume Messages:**
- Peek at messages
- From beginning/end
- Specific offset

## Data Model

```rust
pub struct Topic {
    pub name: String,
    pub partitions: u32,
    pub replication_factor: u16,
    pub configs: HashMap<String, String>,
    pub internal: bool,
}

pub struct Partition {
    pub topic: String,
    pub partition: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
    pub high_watermark: i64,
    pub low_watermark: i64,
}

pub struct ConsumerGroup {
    pub name: String,
    pub state: GroupState,
    pub members: Vec<GroupMember>,
    pub protocol_type: String,
}

pub struct ConsumerLag {
    pub group: String,
    pub topic: String,
    pub partition: i32,
    pub current_offset: i64,
    pub log_end_offset: i64,
    pub lag: i64,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch views |
| `j/k` | Navigate |
| `enter` | View details |
| `c` | Create topic |
| `d` | Delete (confirm) |
| `o` | Reset offsets |
| `p` | Produce message |
| `C` | Consume messages |
| `l` | View lag |
| `r` | Refresh |
| `R` | Auto-refresh |
| `/` | Search |
| `q` | Quit |

## Configuration

```toml
# ~/.config/queue-monitor-kafka/config.toml
[[clusters]]
name = "local"
bootstrap_servers = "localhost:9092"
# sasl_mechanism = "PLAIN"
# sasl_username = ""
# sasl_password = ""
# ssl_enabled = false

[display]
refresh_secs = 5
default_cluster = "local"
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
rdkafka = { version = "0.36", features = ["cmake-build"] }
```
