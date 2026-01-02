# ssh-hub

SSH connection manager with profiles, snippets, and port forwarding.

## Architecture Decisions

### Connection Failure Handling
- **Auto retry**: Exponential backoff reconnection on failure
- Configurable max attempts and backoff parameters
- Notification on eventual success or final failure

### Snippet Variable Substitution
- **Built-in vars only**: Support {{host}}, {{user}}, {{port}} from connection context
- Automatic substitution without user prompts
- Predictable, safe templating for common use cases

## Features

### Connection Management

**Respect ~/.ssh/config:**
- Parse existing SSH config
- Use defined hosts
- Inherit settings (ProxyJump, IdentityFile, etc.)

**System SSH Agent:**
- No credential storage in app
- Use ssh-agent for keys
- Agent forwarding support

### Host Profiles

```rust
pub struct HostProfile {
    pub name: String,
    pub host: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<PathBuf>,
    pub proxy_jump: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub last_connected: Option<DateTime<Utc>>,
}
```

### Features

**Quick Connect:**
- Fuzzy search hosts
- Recent connections
- Tag filtering

**Connection History:**
- Track connections
- Connection duration
- Quick reconnect

**Command Snippets:**
- Save common commands
- Per-host or global
- Quick execute

**Port Forwarding:**
- Local forwarding (-L)
- Remote forwarding (-R)
- Dynamic forwarding (-D)
- Manage active tunnels

**Multi-hop:**
- ProxyJump support
- Chain connections
- Bastion/jump host management

### Session Management

**Tabs:**
- Multiple sessions
- Switch between
- Named tabs

**Split Panes:**
- Horizontal/vertical splits
- Synchronized input

## Views

**Host List:**
- All configured hosts
- Status (online check optional)
- Quick filters

**Active Sessions:**
- Current connections
- Tunnels
- Quick actions

**Snippets:**
- Command library
- Execute on current host

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate hosts |
| `enter` | Connect |
| `/` | Search hosts |
| `t` | Filter by tag |
| `h` | Connection history |
| `s` | Snippets |
| `f` | Port forwards |
| `a` | Add host |
| `e` | Edit host |
| `d` | Delete host |
| `Tab` | Switch sessions |
| `Ctrl+n` | New tab |
| `Ctrl+w` | Close tab |
| `q` | Quit |

## Configuration

```toml
# ~/.config/ssh-hub/config.toml
[ssh]
parse_config = true
config_path = "~/.ssh/config"
agent_forwarding = false

[display]
show_tags = true
show_last_connected = true

[connection]
timeout_secs = 30
keepalive_secs = 60

[history]
max_entries = 100
```

## Data Storage

```sql
CREATE TABLE hosts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    user TEXT,
    port INTEGER,
    identity_file TEXT,
    proxy_jump TEXT,
    notes TEXT,
    last_connected TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE host_tags (
    host_id TEXT NOT NULL REFERENCES hosts(id),
    tag TEXT NOT NULL,
    PRIMARY KEY (host_id, tag)
);

CREATE TABLE snippets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    host_id TEXT,  -- NULL for global
    description TEXT
);

CREATE TABLE connection_history (
    id TEXT PRIMARY KEY,
    host_id TEXT NOT NULL REFERENCES hosts(id),
    connected_at TEXT NOT NULL,
    disconnected_at TEXT,
    duration_secs INTEGER
);

CREATE TABLE port_forwards (
    id TEXT PRIMARY KEY,
    host_id TEXT NOT NULL REFERENCES hosts(id),
    forward_type TEXT NOT NULL,  -- local, remote, dynamic
    local_port INTEGER,
    remote_host TEXT,
    remote_port INTEGER,
    active INTEGER DEFAULT 0
);
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
ssh2 = "0.9"
tokio = { workspace = true }
fuzzy-matcher = { workspace = true }
```
