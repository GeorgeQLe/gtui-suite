# task-scheduler

Unified view and management of cron, systemd timers, and at jobs.

## Architecture Decisions

### Cron Schedule Display
- **Both views**: Human-readable by default ("Every Monday at 3pm")
- Expand to show next N actual timestamps
- Calendar overlay view with computed times

### Cron Expression Editor
- **Visual builder**: Dropdown pickers for minute, hour, day, month, weekday
- Real-time preview of resulting schedule
- Validates syntax on the fly

### Time Collision Display
- **Priority icons**: Show in chronological order
- Colored icons indicate source type (cron/systemd/at)
- Clear visual distinction while maintaining time-based sort

## Features

### Unified View

See all scheduled tasks from:
- crontab (user and system)
- systemd timers
- at queue

### Crontab Management

**View:**
- List all cron entries
- Parse and display schedule in human-readable form
- Show next run time

**Edit:**
- Add new cron jobs
- Edit existing entries
- Delete entries
- Validate cron syntax

### Systemd Timers

**View:**
- List all timer units
- Show status (active, inactive, failed)
- Display schedule and next trigger
- View linked service

**Manage:**
- Enable/disable timers
- Start/stop timers
- View timer logs

### At Jobs

**View:**
- Queued at jobs
- Scheduled time
- Job content preview

**Manage:**
- Add new at job
- Remove pending job
- View job details

### Features

**Next Runs:**
- Calendar view of upcoming jobs
- Timeline view
- Conflict detection (overlapping jobs)

**Logs:**
- View execution history
- Exit codes
- Output logs

**Test Run:**
- Execute job immediately
- Capture output
- Don't affect schedule

## Data Model

```rust
pub enum ScheduledTask {
    Cron(CronEntry),
    SystemdTimer(TimerUnit),
    AtJob(AtEntry),
}

pub struct CronEntry {
    pub id: String,
    pub schedule: CronSchedule,
    pub command: String,
    pub user: String,
    pub file: PathBuf,  // Which crontab file
    pub enabled: bool,
}

pub struct TimerUnit {
    pub name: String,
    pub description: String,
    pub on_calendar: Option<String>,
    pub on_boot_sec: Option<Duration>,
    pub service: String,
    pub state: UnitState,
    pub next_trigger: Option<DateTime<Utc>>,
}

pub struct AtEntry {
    pub job_id: i32,
    pub scheduled_time: DateTime<Utc>,
    pub queue: char,
    pub command: String,
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate tasks |
| `Tab` | Cycle sources (cron/systemd/at) |
| `enter` | View details |
| `a` | Add new task |
| `e` | Edit task |
| `d` | Delete task |
| `x` | Execute now (test) |
| `t` | Toggle enable/disable |
| `l` | View logs |
| `c` | Calendar view |
| `n` | Next runs view |
| `r` | Refresh |
| `q` | Quit |

## Configuration

```toml
# ~/.config/task-scheduler/config.toml
[sources]
cron = true
systemd = true
at = true

[display]
time_format = "24h"
show_next_n = 10

[cron]
edit_command = "$EDITOR"
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
cron = "0.12"
zbus = "4"  # For systemd D-Bus
```

## Permissions

- Reading crontabs may require root for system crontab
- systemd operations require appropriate permissions
- at queue access requires atd
