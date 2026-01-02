# queue-monitor-rabbitmq

RabbitMQ monitoring and management.

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
- **Double confirmation + count**: First confirm shows impact ("4,523 messages"), second requires typing resource name
- Quick-delete keybind for resources under threshold (e.g., <10 messages or empty)
- Prevents accidental data loss while allowing fast operations on safe targets

## Features

### Management API

Connect via RabbitMQ Management HTTP API:
- Requires management plugin enabled
- Authentication

### Views

**Overview:**
- Cluster status
- Message rates
- Queue totals
- Connection count

**Exchanges:**
- List exchanges
- Type, durability
- Bindings
- Message rates

**Queues:**
- Queue list with message counts
- Ready/unacked messages
- Consumer count
- Memory usage

**Bindings:**
- Exchange to queue bindings
- Routing keys
- Arguments

**Consumers:**
- Active consumers
- Prefetch count
- Acknowledgement mode

**Connections:**
- Client connections
- Channels per connection
- Flow control status

### Queue Details

**Metrics:**
- Message counts over time
- Publish/deliver rates
- Consumer utilization

**Actions:**
- Purge queue (delete messages)
- Delete queue
- Get messages (peek)

### Operations

**Publish Test:**
- Publish to exchange
- Routing key
- Headers
- Payload

**Get Messages:**
- Peek at messages
- Acknowledge
- Requeue

### Vhosts

- Switch between vhosts
- Vhost-specific views

## Data Model

```rust
pub struct Queue {
    pub name: String,
    pub vhost: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub messages: u64,
    pub messages_ready: u64,
    pub messages_unacked: u64,
    pub consumers: u32,
    pub memory: u64,
    pub state: QueueState,
}

pub struct Exchange {
    pub name: String,
    pub vhost: String,
    pub exchange_type: ExchangeType,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
}

pub enum ExchangeType {
    Direct,
    Fanout,
    Topic,
    Headers,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch views |
| `j/k` | Navigate |
| `enter` | View details |
| `v` | Change vhost |
| `p` | Publish message |
| `g` | Get messages |
| `P` | Purge queue |
| `d` | Delete (confirm) |
| `r` | Refresh |
| `R` | Auto-refresh toggle |
| `/` | Search |
| `q` | Quit |

## Configuration

```toml
# ~/.config/queue-monitor-rabbitmq/config.toml
[rabbitmq]
url = "http://localhost:15672"
user = "guest"
password_source = "keyring"  # keyring, env, file
default_vhost = "/"

[display]
refresh_secs = 5
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
reqwest = { workspace = true }
keyring = "3"
```
