# backup-manager

Multi-backend backup manager with scheduling and restore browser.

## Architecture Decisions

### Hook Script Failure
- **Prompt user**: Show dialog when pre/post hook fails
- User decides whether to continue or abort backup
- Provides control without silent failures

### Restore File Conflicts
- **Prompt each**: Ask for each conflicting file
- Options: overwrite, skip, rename, compare
- Full user control over restore behavior

### Schedule Overlap
- **Skip + log**: If previous backup still running, skip scheduled run
- Log that run was skipped due to overlap
- Prevents resource contention and corruption

## Features

### Backup Backends

**Rsync:**
- Local or remote destinations
- Incremental backups
- Bandwidth limiting
- SSH transport

**Restic:**
- Encrypted backups
- Deduplication
- Multiple storage backends (local, S3, SFTP)
- Snapshot management

**Borg:**
- Compression + deduplication
- Encryption
- Mount snapshots as filesystem

### Backup Profiles

```rust
pub struct BackupProfile {
    pub id: Uuid,
    pub name: String,
    pub backend: BackendType,
    pub source_paths: Vec<PathBuf>,
    pub destination: String,
    pub excludes: Vec<String>,
    pub schedule: Option<Schedule>,
    pub retention: RetentionPolicy,
    pub pre_hooks: Vec<String>,
    pub post_hooks: Vec<String>,
    pub enabled: bool,
}

pub struct RetentionPolicy {
    pub keep_last: Option<u32>,
    pub keep_hourly: Option<u32>,
    pub keep_daily: Option<u32>,
    pub keep_weekly: Option<u32>,
    pub keep_monthly: Option<u32>,
    pub keep_yearly: Option<u32>,
}
```

### Features

**Scheduling:**
- Cron-style schedules
- Run on connect (USB, network)
- Manual trigger

**Pre/Post Hooks:**
- Run scripts before backup (e.g., dump database)
- Run scripts after (e.g., notification)

**Integrity Verification:**
- Verify backup integrity
- Check for corruption
- Test restore

**Restore Browser:**
- Browse snapshots
- Navigate file tree
- Preview files
- Restore selection

### Views

**Dashboard:**
- All profiles
- Last backup status
- Next scheduled
- Storage usage

**Profile Detail:**
- Configuration
- Recent runs
- Snapshot list

**Restore View:**
- Snapshot browser
- File tree
- Restore wizard

**Logs:**
- Backup output
- Error details
- History

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate profiles |
| `enter` | View details |
| `b` | Run backup now |
| `v` | Verify backup |
| `r` | Restore browser |
| `a` | Add profile |
| `e` | Edit profile |
| `d` | Delete profile |
| `l` | View logs |
| `s` | Snapshots list |
| `t` | Toggle enabled |
| `q` | Quit |

## Configuration

```toml
# ~/.config/backup-manager/config.toml
[general]
log_path = "~/.local/share/backup-manager/logs"
notification_cmd = "notify-send"

[rsync]
default_options = ["-avz", "--delete"]

[restic]
cache_dir = "~/.cache/restic"

[borg]
# Borg-specific settings
```

## Notifications

- Email on completion/failure
- Desktop notification
- Custom command

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
cron = "0.12"
```

## External Tool Integration

Shells out to:
- `rsync`
- `restic`
- `borg`

Parses their output for progress and status.
