# port-scanner-local

Local network port scanner for device discovery.

## Architecture Decisions

### Scan Rate Limiting
- **Adaptive rate**: Start aggressive, back off on timeouts/resets, speed up on success
- Show real-time packets-per-second rate in status bar
- Maximizes speed while respecting network limits
- Timing template as starting point, adaptive adjustment from there

### Results Persistence
- **SQLite with comparison**: Store all scans in SQLite, show diffs between scans
- Track new/removed ports and services over time
- Automatic retention policy: keep last N scans per target or scans from last N days
- Enables detection of network changes and new services

### Authorization Disclaimer
- **First-run disclaimer**: Show authorization disclaimer on first launch
- Require explicit acknowledgment before proceeding
- Store acknowledgment in config file
- Protects users in corporate environments where scanning may require approval

## Features

### Network Discovery

**ARP Discovery:**
- Scan local subnet
- MAC address collection
- Vendor lookup

**Port Scanning:**
- TCP connect scan
- UDP scan
- Common ports preset

### Scan Types

**TCP Connect:**
- Full TCP handshake
- Reliable detection
- Slower but stealthy

**UDP Scan:**
- Send UDP packets
- Detect open ports by response
- Less reliable than TCP

### Features

**Common Ports:**
- Web: 80, 443, 8080, 8443
- SSH: 22
- Database: 3306, 5432, 27017
- Email: 25, 110, 143, 993, 995

**Custom Ranges:**
- Single port: 22
- Range: 1-1024
- List: 22,80,443

**Service Detection:**
- Banner grabbing
- Protocol identification

**Rate Limiting:**
- Configurable scan rate
- Avoid network flooding

### Views

**Host List:**
- Discovered hosts
- Open ports summary
- MAC/vendor info

**Host Detail:**
- All open ports
- Service banners
- Scan history

**Network Map:**
- ASCII network diagram
- Host visualization

## Data Model

```rust
pub struct Host {
    pub ip: IpAddr,
    pub mac: Option<MacAddr>,
    pub vendor: Option<String>,
    pub hostname: Option<String>,
    pub ports: Vec<Port>,
    pub last_seen: DateTime<Utc>,
}

pub struct Port {
    pub number: u16,
    pub protocol: Protocol,
    pub state: PortState,
    pub service: Option<String>,
    pub banner: Option<String>,
}

pub enum PortState {
    Open,
    Closed,
    Filtered,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate hosts |
| `enter` | View host details |
| `s` | Start scan |
| `S` | Stop scan |
| `p` | Port range dialog |
| `r` | Rescan host |
| `x` | Export results |
| `/` | Search |
| `Tab` | Switch views |
| `q` | Quit |

## Configuration

```toml
# ~/.config/port-scanner-local/config.toml
[scan]
interface = "eth0"
timeout_ms = 1000
max_concurrent = 100
rate_limit_per_sec = 1000

[ports]
default = "common"  # common, top100, top1000, all
common = [22, 80, 443, 8080, 3306, 5432]

[discovery]
arp_enabled = true
ping_enabled = true

[output]
save_path = "~/.local/share/port-scanner-local"
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
pnet = { workspace = true }
socket2 = "0.5"
mac_address = "1"
oui = "0.7"  # MAC vendor lookup
dns-lookup = "2"
```
