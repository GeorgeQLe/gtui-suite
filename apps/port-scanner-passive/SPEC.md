# port-scanner-passive

Passive network discovery without active probing.

## Architecture Decisions

### Results Persistence
- **SQLite with comparison**: Store all discoveries in SQLite, show device changes over time
- Track when devices first seen, last seen, service announcements
- Automatic retention policy: configurable retention period (default 90 days)
- Essential for building complete network inventory over time

## Features

### Passive Monitoring

No packets sent - only listen:
- No detection by target systems
- Safe for production networks
- Builds inventory over time

### Discovery Methods

**ARP Monitoring:**
- Listen for ARP requests/replies
- Build MAC-to-IP mapping
- Detect new devices

**mDNS/Bonjour:**
- Apple devices
- Printers
- Smart TVs
- IoT devices

**SSDP/UPnP:**
- Routers
- Media servers
- Smart home devices

**NetBIOS:**
- Windows shares
- Hostnames
- Workgroups

**DHCP:**
- New device requests
- IP assignments
- Hostname info

**DNS:**
- Monitor DNS queries
- Reverse lookups
- Discover internal names

### Device Inventory

Build device database over time:
```rust
pub struct Device {
    pub ip: IpAddr,
    pub mac: MacAddr,
    pub vendor: Option<String>,
    pub hostname: Option<String>,
    pub discovery_method: DiscoveryMethod,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub services: Vec<Service>,
}

pub struct Service {
    pub name: String,
    pub port: Option<u16>,
    pub metadata: HashMap<String, String>,
}
```

### Features

**Vendor Lookup:**
- MAC address OUI database
- Device type identification

**Export:**
- CSV export
- JSON export
- Network diagram

**Alerts:**
- New device detected
- Device disappeared
- Unusual activity

## Views

**Device List:**
- All discovered devices
- Vendor info
- Last seen

**Timeline:**
- Device appearances
- Service announcements

**Network Map:**
- Visual layout
- Connection indicators

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate devices |
| `enter` | View device details |
| `m` | Start monitoring |
| `M` | Stop monitoring |
| `x` | Export |
| `/` | Search |
| `s` | Sort |
| `f` | Filter |
| `Tab` | Switch views |
| `q` | Quit |

## Configuration

```toml
# ~/.config/port-scanner-passive/config.toml
[monitoring]
interface = "eth0"
protocols = ["arp", "mdns", "ssdp", "netbios", "dhcp"]

[storage]
path = "~/.local/share/port-scanner-passive"
retention_days = 90

[alerts]
new_device = true
notification_cmd = "notify-send"
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
tokio = { workspace = true }
pnet = { workspace = true }
mdns = "3"
oui = "0.7"
dns-lookup = "2"
```

## Permissions

Most protocols work without root:
- mDNS, SSDP: UDP multicast (no special permissions)
- NetBIOS: UDP broadcast (no special permissions)

ARP monitoring may need raw socket access depending on method.
