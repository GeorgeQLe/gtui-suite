# tui-shell-fullscreen

Minimal fullscreen shell with fast app switching.

## Architecture Decisions

### App Suspension Model
- **Keep running**: Apps continue running in background when switched away
- Timers, watchers, and background tasks continue working
- Instant resume when switching back - no reload delay

### Switcher No-Match Behavior
- **Continue filtering**: When query matches nothing, show "No matches for xyz"
- User can backspace to correct query
- Consistent behavior, Escape always dismisses

### Session Save Format
- **JSON**: Use JSON for session persistence
- Stores running apps, recent order, quick slot assignments

## Features

### Single App Focus

**Fullscreen Display:**
- One app fills entire screen
- Minimal chrome
- Maximum content area
- Zero distractions

**Status Line:**
- Single line at bottom
- Optional (can be hidden)
- Notification marquee
- Quick status indicators

### Fast Switcher

**Overlay Launcher:**
- `Ctrl+Space` triggers overlay
- Fuzzy search apps
- Recent apps first
- Launch new or switch to running

**Overlay Display:**
- Semi-transparent overlay
- App list with search
- Keyboard-driven selection
- Instant dismiss on select

### App Management

**Running Apps:**
- Background apps continue running
- Quick switch between running
- Close apps from switcher
- View running app count

**Recent Order:**
- Most recent first
- `Ctrl+Space` twice for last app
- Cycle through with arrows
- Jump to specific app

### Notification Queue

**Minimal Notifications:**
- Queued notifications
- Scroll through with keybind
- Dismiss individual
- Clear all

**Priority Levels:**
- Critical: Always show
- Normal: Show briefly
- Low: Queue silently

### Quick Actions

**Global Shortcuts:**
- Launch specific apps directly
- `$mod+1-9`: Quick launch slots
- Configurable per slot

## Data Model

```rust
pub struct FullscreenShell {
    pub apps: Vec<RunningApp>,
    pub active: Option<AppId>,
    pub recent_order: Vec<AppId>,
    pub quick_slots: [Option<String>; 9],
    pub notifications: VecDeque<Notification>,
    pub config: FullscreenConfig,
}

pub struct RunningApp {
    pub id: AppId,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub suspended: bool,
}

pub struct Notification {
    pub id: NotificationId,
    pub source: AppId,
    pub message: String,
    pub priority: Priority,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

pub enum Priority {
    Low,
    Normal,
    Critical,
}
```

## Views

**App View (Primary):**
- Full screen app content
- No decorations
- Single optional status line

**Status Line:**
```
[habit-tracker] 3 apps running │ 2 notifications │ 14:32
```

**Switcher Overlay:**
```
┌─────────────────────────────────────┐
│ > task_                             │
│                                     │
│   task-manager        [running]     │
│   time-tracker        [running]     │
│   task-scheduler      [launch]      │
│                                     │
│   ↑↓ navigate  enter select  esc quit│
└─────────────────────────────────────┘
```

**Notification View:**
```
┌─ Notifications ─────────────────────┐
│ [task-manager] Task "Review code"   │
│   due in 30 minutes                 │
│                                     │
│ [habit-tracker] Don't forget:       │
│   Exercise today!                   │
│                                     │
│   j/k navigate  d dismiss  D clear  │
└─────────────────────────────────────┘
```

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Space` | Open switcher |
| `Ctrl+Space Ctrl+Space` | Switch to last app |
| `$mod+q` | Close current app |
| `$mod+1-9` | Quick slot launch |
| `$mod+n` | Show notifications |
| `$mod+Shift+n` | Clear notifications |
| `$mod+b` | Toggle status bar |
| `$mod+/` | Show all keybindings |
| `$mod+Shift+e` | Exit shell |

**In Switcher:**
| Key | Action |
|-----|--------|
| `j/k` or `↑/↓` | Navigate |
| `Enter` | Select/launch |
| `Esc` | Cancel |
| `x` | Close running app |
| `/` | Focus search |

## Configuration

```toml
# ~/.config/tui-shell-fullscreen/config.toml
[general]
mod_key = "ctrl+space"
show_status_bar = true
double_tap_switch = true

[status_bar]
position = "bottom"
height = 1
show_app_name = true
show_app_count = true
show_notification_count = true
show_clock = true
clock_format = "%H:%M"

[switcher]
width = 50
max_results = 10
show_running_indicator = true
recent_first = true
fuzzy_matching = true

[notifications]
max_queue = 50
auto_dismiss_sec = 0  # 0 = never
show_critical = true

[quick_slots]
1 = "habit-tracker"
2 = "task-manager"
3 = "time-tracker"
4 = "note-manager-folder"
5 = "cheatsheet-browser"
# 6-9 unassigned

[startup]
default_app = "task-manager"
restore_last_app = true

[theme]
name = "base16-ocean"
switcher_opacity = 0.9
```

## App Lifecycle

**Launch:**
1. App not running → start fresh
2. App suspended → resume
3. App running → switch to it

**Suspend:**
- When switching away
- App state preserved
- Background work continues

**Close:**
- From switcher (x key)
- From within app (app's quit)
- Confirm if unsaved changes

## Dependencies

```toml
[dependencies]
tui-shell = { workspace = true }
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
tui-keybinds = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
serde = { workspace = true }
tokio = { workspace = true }
```

## Minimal Design Philosophy

This shell is for users who want:
- Maximum screen real estate
- Zero visual noise
- Fast keyboard-driven switching
- Focus on one task at a time

Inspired by:
- macOS fullscreen mode
- dmenu/rofi launchers
- Minimalist window managers
- Focus/distraction-free modes

The switcher overlay appears only when needed, then disappears completely, leaving the user with an uninterrupted view of their current application.
