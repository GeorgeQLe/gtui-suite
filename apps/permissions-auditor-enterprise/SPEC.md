# permissions-auditor-enterprise

Enterprise-grade permissions auditing with compliance frameworks.

## Architecture Decisions

### Auto-Fix Behavior
- **Preview + batch apply**: Show all proposed changes in preview, apply as single batch with backup
- Backup affected file permissions to restore file before applying
- "Dry run" mode shows exactly what would change without modifying anything
- Single-click to apply all fixes after reviewing preview

### Multi-System Scan Coordination
- **Parallel with limit**: Scan N systems concurrently (default 5), queue others
- Progress view shows queue with estimated completion
- Priority flag to bump critical systems to front of queue
- Balances speed with resource usage and network impact

### Agent Authentication
- **SSH key-based**: Use existing SSH keys for agent authentication
- Standard practice, no new credentials to manage
- Agent runs as user over SSH connection
- Supports SSH agent forwarding for key management

## Features

All features from consumer version, plus:

### Linux Capabilities

**Capability Auditing:**
- Check binary capabilities (getcap)
- Flag unexpected capabilities
- CAP_NET_ADMIN, CAP_SYS_ADMIN, etc.

**Known-Safe Database:**
- Compare against expected capabilities
- Alert on deviations

### Container/Namespace Awareness

**Container Detection:**
- Identify containerized processes
- Namespace boundary analysis
- Escaped mount detection

**Docker/Podman:**
- Container permissions
- Volume mount permissions
- Privileged containers

### Compliance Frameworks

**CIS Benchmarks:**
- Linux CIS benchmarks
- Section mapping
- Pass/fail scoring

**STIG Mapping:**
- DISA STIGs
- Finding IDs
- Remediation guidance

**Custom Rulesets:**
- Define organizational policies
- Import/export rules

### Multi-System Scanning

**SSH-Based:**
- Scan multiple hosts
- Centralized results
- Credential management

**Agent Mode:**
- Deploy lightweight agents
- Push results to collector

### Advanced Features

**Centralized Reporting:**
- Dashboard for all systems
- Aggregate statistics
- Trend analysis

**Trend Tracking:**
- Historical data
- Compliance over time
- Regression detection

**Ticketing Integration:**
- Create tickets for findings
- Track remediation
- Jira, ServiceNow support

### Remediation

**Automated Fixes:**
- Safe auto-remediation
- Preview mode
- Rollback support

**Playbooks:**
- Ansible integration
- Remediation scripts

## Data Model

```rust
pub struct ComplianceCheck {
    pub id: String,
    pub framework: ComplianceFramework,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub check_fn: CheckFunction,
    pub remediation: String,
}

pub enum ComplianceFramework {
    CisBenchmark { version: String, section: String },
    Stig { stig_id: String, rule_id: String },
    Custom { ruleset: String },
}

pub struct SystemScan {
    pub system_id: String,
    pub hostname: String,
    pub scan_time: DateTime<Utc>,
    pub findings: Vec<Finding>,
    pub compliance_score: f64,
}
```

## Views

**Dashboard:**
- Multi-system overview
- Compliance scores
- Critical findings

**System View:**
- Per-system details
- Finding breakdown
- Trend graph

**Compliance View:**
- Framework compliance
- Section breakdown
- Pass/fail details

**Report View:**
- Generate reports
- Export options
- Schedule reports

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate |
| `enter` | View details |
| `s` | Start scan |
| `S` | Scan all systems |
| `c` | Compliance view |
| `d` | Dashboard |
| `t` | Create ticket |
| `f` | Fix finding |
| `r` | Generate report |
| `Tab` | Switch views |
| `q` | Quit |

## Configuration

```toml
# ~/.config/permissions-auditor-enterprise/config.toml
[[systems]]
name = "web-server-01"
host = "192.168.1.10"
user = "auditor"
key_path = "~/.ssh/audit_key"

[[systems]]
name = "db-server-01"
host = "192.168.1.20"

[compliance]
frameworks = ["cis-rhel8", "stig-rhel8"]
custom_rules = "~/.config/permissions-auditor/rules"

[capabilities]
check_binaries = true
known_safe_path = "~/.config/permissions-auditor/capabilities.toml"

[container]
scan_docker = true
scan_podman = true

[reporting]
output_path = "~/.local/share/permissions-auditor/reports"
formats = ["html", "json", "pdf"]

[ticketing]
enabled = false
system = "jira"
url = "https://jira.example.com"
project = "SEC"
```

## CIS Benchmark Coverage

Example checks:
- 1.1.1: Disable unused filesystems
- 1.4.1: Ensure permissions on bootloader config
- 4.1.1: Ensure auditing is enabled
- 5.2.1: Ensure permissions on /etc/ssh/sshd_config
- 6.1.1: Audit system file permissions

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
walkdir = "2"
nix = { version = "0.29", features = ["fs", "user"] }
caps = "0.5"  # Linux capabilities
ssh2 = "0.9"
reqwest = { workspace = true }
handlebars = "6"  # Report templates
```
