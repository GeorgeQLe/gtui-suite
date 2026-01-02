# tui-shell-floating

Windows-style floating window manager for TUI applications.

## Architecture Decisions

### Window Placement
- **Cascade placement**: Offset each new window by fixed amount from previous window
- Wraps naturally when reaching screen edge
- All title bars remain visible for easy access

### Session Save Format
- **JSON**: Use JSON for session persistence
- Human-readable, easy to hand-edit for power users
- Stores window positions, sizes, and states

## Features

### Floating Windows

**Independent Windows:**
- Overlapping windows
- Free positioning
- Free resizing
- Z-order management

**Window Chrome:**
- Title bar with app name
- Close button
- Minimize/maximize buttons
- Resize handles (corners/edges)

### Window Operations

**Move:**
- Keyboard: `$mod+arrow`
- Click and drag title bar
- Snap to edges
- Snap to other windows

**Resize:**
- Keyboard: `$mod+Shift+arrow`
- Drag resize handles
- Minimum size constraints
- Maximum size constraints

**Z-Order:**
- Raise on focus
- Lower window
- Always on top
- Send to back

### Window States

**Normal:**
- Free floating position
- User-defined size

**Maximized:**
- Fill entire screen (minus status bar)
- Toggle with `$mod+m`
- Restore previous size/position

**Minimized:**
- Hidden from view
- Visible in taskbar
- Click to restore

**Snapped:**
- Snap to half-screen (left/right)
- Snap to quarter (corners)
- Snap to top/bottom halves

### Taskbar

**Window List:**
- All open windows
- Active window highlighted
- Minimized windows indicator
- Click to focus/restore

**Features:**
- Window previews on hover (optional)
- Window grouping by app type
- Right-click context menu

### Virtual Desktops

**Multiple Desktops:**
- Numbered virtual desktops
- Windows per desktop
- Sticky windows (all desktops)

**Navigation:**
- `$mod+1-4`: Switch desktop
- `$mod+Shift+1-4`: Move window

### Window Cascade

**Auto-Arrange:**
- Cascade: Offset each window
- Tile: Non-overlapping grid
- Side-by-side: Two windows 50/50
- Stack: All same position

## Data Model

```rust
pub struct FloatingShell {
    pub desktops: Vec<Desktop>,
    pub active_desktop: usize,
    pub taskbar: Taskbar,
    pub config: FloatingConfig,
}

pub struct Desktop {
    pub id: u32,
    pub name: String,
    pub windows: Vec<Window>,
    pub focused: Option<WindowId>,
}

pub struct Window {
    pub id: WindowId,
    pub app: AppId,
    pub rect: Rect,
    pub state: WindowState,
    pub z_order: u32,
    pub sticky: bool,
    pub always_on_top: bool,
}

pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub enum WindowState {
    Normal { saved_rect: Rect },
    Maximized { saved_rect: Rect },
    Minimized { saved_rect: Rect },
    Snapped { position: SnapPosition, saved_rect: Rect },
}

pub enum SnapPosition {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
}
```

## Views

**Desktop View:**
- Floating windows with borders
- Title bars
- Window shadows (if space permits)
- Focus ring on active window

**Taskbar:**
- Window buttons
- Desktop switcher
- Notification area
- Clock (optional)

**Window Decorations:**
- Title text (truncated)
- Close [X]
- Maximize [□]
- Minimize [_]

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Space` | Command prefix |
| `$mod+Enter` | Launch app picker |
| `$mod+q` | Close window |
| `$mod+Tab` | Cycle windows |
| `$mod+Shift+Tab` | Cycle reverse |
| `$mod+arrow` | Move window |
| `$mod+Shift+arrow` | Resize window |
| `$mod+m` | Maximize/restore |
| `$mod+n` | Minimize |
| `$mod+r` | Raise window |
| `$mod+l` | Lower window |
| `$mod+t` | Toggle always on top |
| `$mod+Left` | Snap left half |
| `$mod+Right` | Snap right half |
| `$mod+Up` | Maximize |
| `$mod+Down` | Restore/minimize |
| `$mod+1-4` | Switch desktop |
| `$mod+Shift+1-4` | Move to desktop |
| `$mod+c` | Cascade windows |
| `$mod+g` | Tile windows |
| `$mod+Shift+e` | Exit shell |

## Configuration

```toml
# ~/.config/tui-shell-floating/config.toml
[general]
mod_key = "ctrl+space"
focus_follows_mouse = false
raise_on_focus = true
snap_threshold = 10

[window]
min_width = 20
min_height = 5
default_width = 80
default_height = 24
title_bar_height = 1
border_width = 1

[decorations]
show_title = true
show_close = true
show_maximize = true
show_minimize = true
focused_color = "accent"
unfocused_color = "muted"

[taskbar]
position = "bottom"
height = 1
show_desktop_switcher = true
show_clock = true
show_notifications = true

[desktops]
count = 4
names = ["Desktop 1", "Desktop 2", "Desktop 3", "Desktop 4"]

[snap]
enabled = true
show_preview = true
edge_resistance = 5

[startup]
restore_session = true
session_file = "~/.local/state/tui-shell-floating/session.json"

[theme]
name = "nord"
window_shadow = false
```

## Window Placement

**Smart Placement:**
- Cascade from top-left
- Avoid covering focused window
- Center if first window
- Remember per-app positions

**Position Memory:**
```json
{
  "app_positions": {
    "task-manager": { "x": 10, "y": 5, "width": 60, "height": 20 },
    "note-manager-folder": { "x": 25, "y": 8, "width": 70, "height": 25 }
  }
}
```

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

## Familiar Experience

Designed to feel like Windows/macOS/GNOME:
- Overlapping windows
- Title bar controls
- Taskbar with window list
- Virtual desktops
- Window snapping

Users comfortable with traditional desktop environments will find this natural.
