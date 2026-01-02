# permissions-auditor-consumer

File system permissions auditing for personal use.

## Architecture Decisions

### Auto-Fix Behavior
- **Preview + batch apply**: Show all proposed changes in preview, apply as single batch with backup
- Backup affected file permissions to restore file before applying
- "Dry run" mode shows exactly what would change without modifying anything
- Single-click to apply all fixes after reviewing preview

## Features

### Permission Scanning

**World-Writable Files:**
- Find files/directories writable by anyone
- Exclude expected locations (/tmp)
- Risk scoring

**SUID/SGID Binaries:**
- Detect setuid/setgid programs
- Compare against known-safe list
- Flag unexpected entries

**Ownership Issues:**
- Files not owned by expected user
- System files owned by non-root
- Orphaned files (no valid owner)

**Sensitive File Permissions:**
- ~/.ssh/* permissions
- ~/.gnupg/* permissions
- Config files with credentials

### Scan Paths

Configurable paths:
- Home directory
- /etc (system configs)
- Custom paths

### Ignore Patterns

Skip known-safe locations:
```toml
[ignore]
paths = [
    "/tmp",
    "/var/tmp",
    "*/node_modules/*",
    "*/.git/*",
]
```

### Reports

**Finding Summary:**
- Count by severity
- Count by type
- Trend over time

**Detailed Report:**
- Each finding with path
- Current permissions
- Recommended fix
- Fix command

### Fix Suggestions

Generate fix commands:
```bash
chmod 600 ~/.ssh/id_rsa
chmod 755 ~/.local/bin
chown user:user ~/important-file
```

### Scheduling

- Run on demand
- Schedule periodic scans
- Alert on new findings

## Data Model

```rust
pub struct Finding {
    pub id: Uuid,
    pub path: PathBuf,
    pub finding_type: FindingType,
    pub severity: Severity,
    pub current_permissions: String,
    pub recommended_permissions: Option<String>,
    pub description: String,
    pub fix_command: Option<String>,
    pub found_at: DateTime<Utc>,
    pub resolved: bool,
}

pub enum FindingType {
    WorldWritable,
    SuidBinary,
    SgidBinary,
    WeakSshPermissions,
    WeakGpgPermissions,
    OwnershipIssue,
    SensitiveFileExposed,
}

pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate findings |
| `enter` | View details |
| `s` | Start scan |
| `f` | Fix finding |
| `F` | Fix all (confirm) |
| `i` | Ignore finding |
| `r` | Generate report |
| `x` | Export |
| `Tab` | Filter by type |
| `q` | Quit |

## Configuration

```toml
# ~/.config/permissions-auditor-consumer/config.toml
[scan]
paths = ["~", "/etc"]
follow_symlinks = false
max_depth = 10

[checks]
world_writable = true
suid_sgid = true
ssh_permissions = true
gpg_permissions = true

[ignore]
paths = ["/tmp", "/var/tmp"]
patterns = ["*/node_modules/*", "*/.git/*"]

[schedule]
enabled = false
cron = "0 0 * * 0"  # Weekly

[reports]
path = "~/.local/share/permissions-auditor/reports"
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
walkdir = "2"
nix = { version = "0.29", features = ["fs", "user"] }
glob = "0.3"
```
