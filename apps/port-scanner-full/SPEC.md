# port-scanner-full

Comprehensive nmap-style port scanner for authorized testing.

**IMPORTANT:** Only use with explicit authorization on systems you own or have permission to test.

## Architecture Decisions

### Scan Rate Limiting
- **Adaptive rate**: Start aggressive, back off on timeouts/resets, speed up on success
- Show real-time packets-per-second rate in status bar
- Use timing template (T0-T5) as starting point, adaptive adjustment from there
- Maximizes speed while respecting network limits and avoiding detection

### Results Persistence
- **SQLite with comparison**: Store all scans in SQLite, show diffs between scans
- Track new/removed ports, service version changes over time
- Automatic retention policy: keep last N scans per target or scans from last N days
- Essential for tracking security posture changes

## Features

### Scan Techniques

**TCP Scans:**
- SYN scan (stealth) - requires root
- Connect scan
- ACK scan
- FIN scan
- Xmas scan
- NULL scan

**UDP Scan:**
- UDP port detection
- Protocol-specific probes

### Service Detection

**Version Probing:**
- Send protocol-specific probes
- Parse responses for version info
- Confidence scoring

**Protocol Detection:**
- HTTP/HTTPS
- SSH
- FTP
- SMTP
- DNS
- TLS version

### OS Fingerprinting

**TCP/IP Stack Analysis:**
- TTL values
- Window size
- TCP options
- IP ID patterns

**Signature Matching:**
- OS signature database
- Best match with confidence

### Script Scanning

**Custom Lua Scripts:**
- NSE-like scripting
- Vulnerability checks
- Information gathering

**Built-in Scripts:**
- SSL certificate info
- HTTP headers
- DNS zone transfer
- SMB enumeration

### Scan Profiles

**Quick:**
- Top 100 ports
- No version detection
- No OS fingerprinting

**Comprehensive:**
- All 65535 ports
- Version detection
- OS fingerprinting

**Stealth:**
- SYN scan
- Slow timing
- Randomized order

## Views

**Scan Progress:**
- Real-time results
- Progress bar
- Statistics

**Results View:**
- Hosts discovered
- Ports per host
- Service versions

**Detail View:**
- Full host info
- Script output
- Raw responses

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `s` | Start scan |
| `S` | Stop scan |
| `p` | Scan profile |
| `t` | Target selection |
| `o` | Options |
| `x` | Export |
| `r` | Generate report |
| `q` | Quit |

## Configuration

```toml
# ~/.config/port-scanner-full/config.toml
[scan]
default_profile = "quick"
timeout_ms = 3000
max_retries = 2
max_concurrent = 500

[timing]
# Timing templates (like nmap -T)
paranoid = { delay_ms = 300000, concurrent = 1 }
sneaky = { delay_ms = 15000, concurrent = 10 }
polite = { delay_ms = 400, concurrent = 100 }
normal = { delay_ms = 0, concurrent = 500 }
aggressive = { delay_ms = 0, concurrent = 1000 }

[scripts]
enabled = true
path = "~/.config/port-scanner-full/scripts"

[output]
default_format = "xml"
save_path = "~/.local/share/port-scanner-full"
```

## Report Formats

- XML (nmap-compatible)
- JSON
- HTML
- Plain text

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
mlua = "0.9"  # Lua scripting
quick-xml = "0.36"
dns-lookup = "2"
openssl = "0.10"
```

## Permissions

Requires root/CAP_NET_RAW for:
- SYN scan
- OS fingerprinting
- Some service probes

Connect scan works without root.
