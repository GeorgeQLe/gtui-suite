# network-monitor

Network connection monitor with packet capture and protocol analysis.

**Platform:** Linux primary (uses /proc/net, AF_PACKET)

## Architecture Decisions

### Permission Handling
- **Refuse + explain**: Exit with clear message about required permissions
- Explain that root or CAP_NET_RAW is needed for packet capture
- No degraded mode - captures are core functionality
- Suggest running with sudo or setting capabilities

### Pcap Export Scope
- **Filtered only**: Export what's currently visible after filters
- Smaller, more relevant export files
- User can clear filters before export if full dump needed

## Features

### Connection Listing

Like ss/netstat but visual:
- TCP connections (established, listening)
- UDP sockets
- Unix sockets
- Per-process mapping

```rust
pub struct Connection {
    pub protocol: Protocol,
    pub local_addr: SocketAddr,
    pub remote_addr: Option<SocketAddr>,
    pub state: TcpState,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub user: Option<String>,
}
```

### Bandwidth Per-Process

Track network usage by process:
- Bytes in/out
- Packets in/out
- Bandwidth rate
- Requires: /proc polling or eBPF

### Protocol Awareness

Identify application protocols:
- HTTP/HTTPS
- DNS
- SSH
- TLS version
- Custom identification

### Packet Capture Mode

**Requires root or CAP_NET_RAW:**

Using pnet + AF_PACKET:
- Capture all packets on interface
- BPF-style filters
- Live packet display
- Protocol parsing

### Features

**Capture Filters:**
```
host 192.168.1.1
port 80
tcp and dst port 443
```

**Packet Detail View:**
- Ethernet frame
- IP header
- TCP/UDP header
- Payload (hex/ASCII)

**Traffic Graph:**
- Real-time bandwidth graph
- Per-interface
- In/out separately

**Export:**
- pcap format
- For analysis in Wireshark

## Views

**Connections View:**
- Active connections table
- Sort by any column
- Filter by protocol/state

**Bandwidth View:**
- Per-process bandwidth
- Top talkers
- Historical graph

**Capture View:**
- Packet list
- Packet detail
- Hex dump

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `Tab` | Switch views |
| `f` | Set filter |
| `c` | Start/stop capture |
| `s` | Sort menu |
| `/` | Search |
| `g` | Toggle graph |
| `x` | Export to pcap |
| `Space` | Pause updates |
| `q` | Quit |

## Configuration

```toml
# ~/.config/network-monitor/config.toml
[display]
refresh_ms = 1000
resolve_hostnames = false
resolve_services = true

[capture]
default_interface = "eth0"
snap_length = 65535
promiscuous = false

[bandwidth]
interface = "eth0"
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
pnet = { workspace = true }
tokio = { workspace = true }
libc = "0.2"
nix = "0.29"
dns-lookup = "2"
pcap-file = "2"  # For pcap export
```

## Permissions

- Basic connection view: no special permissions
- Per-process mapping: may need root
- Packet capture: requires root or CAP_NET_RAW
