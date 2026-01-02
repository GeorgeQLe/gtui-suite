# service-manager

Full systemd service management with logs and dependency visualization.

## Architecture Decisions

### Dependency Graph Layout
- **Hierarchical tree**: Top-down with WantedBy targets at top
- Dependencies flow downward
- Clear parent-child relationships in ASCII art

### Service Operation Feedback
- **Progress modal**: Show live progress and logs during start/stop/restart
- Modal blocks UI until operation completes
- Displays systemd status messages in real-time

### Log Buffer Management (Follow Mode)
- **Batch flush**: When buffer exceeds max_lines
- Save current buffer to temp file for later review
- Clear buffer and continue with fresh logs
- Temp files persist for session duration

### Privilege Escalation
- **Inline prompt**: Show password prompt within TUI
- Forward authentication to polkit
- Seamless experience without leaving the app

## Features

### Unit Management

**List Units:**
- Services, timers, sockets, mounts, etc.
- Filter by type
- Filter by state (active, inactive, failed)
- Search by name

**Operations:**
- Start/stop/restart/reload
- Enable/disable (boot behavior)
- Mask/unmask
- Reset failed state

### Logs (journalctl)

**View Logs:**
- Unit-specific logs
- Follow mode (live tail)
- Time range filtering
- Priority filtering
- Grep/search within logs

### Unit Details

**Info Display:**
- Description
- Load state, active state, sub state
- Main PID and memory usage
- Started timestamp
- Dependencies (Requires, Wants, After, Before)

**Edit Units:**
- View unit file
- Edit with $EDITOR
- Reload daemon after edit

### Dependency Graph

Visualize unit relationships:
```
nginx.service
├── Requires: network.target
├── After: network.target, nginx.conf.d.mount
└── WantedBy: multi-user.target
```

### Status Dashboard

Overview of system:
- Count by state
- Failed units (highlighted)
- Recent starts/stops

## Views

**List View:**
- All units with status
- Sortable columns
- Quick filters

**Detail View:**
- Single unit info
- Real-time status updates
- Log preview

**Log View:**
- Full log browser
- Advanced filtering
- Export

**Graph View:**
- Dependency visualization
- Navigate dependencies

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate units |
| `enter` | View details |
| `s` | Start unit |
| `S` | Stop unit |
| `r` | Restart unit |
| `R` | Reload unit |
| `e` | Enable unit |
| `E` | Disable unit |
| `m` | Mask unit |
| `M` | Unmask unit |
| `l` | View logs |
| `f` | Follow logs |
| `d` | Dependencies view |
| `Tab` | Filter by type |
| `/` | Search |
| `F` | Show only failed |
| `q` | Quit |

## Configuration

```toml
# ~/.config/service-manager/config.toml
[display]
show_user_units = true
show_system_units = true
default_filter = "service"

[logs]
max_lines = 1000
follow_by_default = false
priority_filter = "info"

[actions]
confirm_dangerous = true  # Confirm stop/disable
```

## Data Model

```rust
pub struct Unit {
    pub name: String,
    pub unit_type: UnitType,
    pub description: String,
    pub load_state: LoadState,
    pub active_state: ActiveState,
    pub sub_state: String,
    pub main_pid: Option<u32>,
    pub memory: Option<u64>,
    pub started_at: Option<DateTime<Utc>>,
}

pub enum UnitType {
    Service,
    Socket,
    Timer,
    Mount,
    Device,
    Swap,
    Target,
    Path,
    Slice,
    Scope,
}

pub enum LoadState {
    Loaded,
    NotFound,
    Error,
    Masked,
}

pub enum ActiveState {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
}
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
zbus = "4"  # D-Bus for systemd
tokio = { workspace = true }
```

## Permissions

- Reading unit status: usually no special permissions
- Starting/stopping: requires appropriate polkit policy or root
- Editing unit files: requires root
