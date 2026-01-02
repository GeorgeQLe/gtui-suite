# process-monitor

Linux process monitor with cgroup/namespace awareness.

**Platform:** Linux only (uses /proc)

## Architecture Decisions

### CPU Usage History
- **On-demand**: Only track history when user expands process detail view
- Keeps main list view lightweight (instant percentages only)
- Full sparkline graphs available in detail view
- No memory overhead for background processes

### Signal Confirmation
- **Dangerous only**: Confirm SIGKILL, allow SIGTERM without confirmation
- SIGTERM is graceful shutdown (recoverable), SIGKILL is not
- Reduces friction for normal process management

## Features

### Process Information

From /proc filesystem:
- PID, PPID, user, group
- Command line and executable path
- State (running, sleeping, zombie, etc.)
- CPU usage (calculated from /proc/[pid]/stat)
- Memory usage (RSS, VMS from /proc/[pid]/statm)
- Thread count
- Start time
- Nice value

### I/O Statistics

From /proc/[pid]/io:
- Read bytes
- Write bytes
- Cancelled write bytes
- Syscall counts

### Network Per-Process

From /proc/net and /proc/[pid]/fd:
- Open sockets
- Connection counts
- Port usage

### Container Awareness

**Cgroups (v1 & v2):**
- Cgroup path
- Resource limits
- Current usage vs limits

**Namespaces:**
- Network namespace
- PID namespace
- Mount namespace
- Identify containers

### Views

**List View:**
- Process table with sortable columns
- Real-time updates
- Color-coded by state

**Tree View:**
- Process hierarchy (parent-child)
- Collapse/expand
- Total resources per subtree

**Detail View:**
- Single process info
- Open files
- Memory maps
- Environment variables

**Graphs:**
- CPU usage over time
- Memory usage over time
- System-wide stats

## Data Model

```rust
pub struct Process {
    pub pid: i32,
    pub ppid: i32,
    pub uid: u32,
    pub gid: u32,
    pub name: String,
    pub cmdline: String,
    pub exe: PathBuf,
    pub state: ProcessState,
    pub cpu_percent: f32,
    pub memory_rss: u64,
    pub memory_vms: u64,
    pub threads: u32,
    pub nice: i8,
    pub start_time: DateTime<Utc>,
    pub io: Option<IoStats>,
    pub cgroup: Option<String>,
    pub namespace: Option<NamespaceInfo>,
}

pub struct IoStats {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_syscalls: u64,
    pub write_syscalls: u64,
}

pub struct NamespaceInfo {
    pub pid_ns: u64,
    pub net_ns: u64,
    pub mnt_ns: u64,
    pub uts_ns: u64,
}

pub enum ProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Zombie,
    Stopped,
    TracingStop,
    Dead,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate processes |
| `J/K` | Page down/up |
| `enter` | View details |
| `t` | Toggle tree view |
| `g` | Toggle graphs |
| `s` | Sort menu |
| `/` | Search/filter |
| `9` | Send SIGKILL (confirm) |
| `T` | Send SIGTERM |
| `r` | Renice process |
| `u` | Filter by user |
| `c` | Filter by container |
| `n` | Show namespaces |
| `i` | Show I/O stats |
| `m` | Toggle memory units (KB/MB/GB) |
| `p` | Toggle CPU % / absolute |
| `Space` | Pause updates |
| `q` | Quit |

## Configuration

```toml
# ~/.config/process-monitor/config.toml
[display]
refresh_ms = 1000
default_view = "list"  # list, tree
default_sort = "cpu"   # cpu, mem, pid, name
show_threads = false
show_kernel_threads = false

[columns]
visible = ["pid", "user", "cpu", "mem", "state", "command"]

[colors]
running = "green"
sleeping = "default"
zombie = "red"
stopped = "yellow"

[units]
memory = "auto"  # auto, KB, MB, GB
```

## Performance

Optimized for low overhead:
- Efficient /proc parsing
- Differential updates
- Configurable refresh rate
- Memory-mapped file reading

## Dependencies

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
procfs = "0.17"
nix = { version = "0.29", features = ["signal", "user"] }
libc = "0.2"
```
