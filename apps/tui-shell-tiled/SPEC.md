# tui-shell-tiled

i3/sway-style tiling window manager for TUI applications.

## Architecture Decisions

### Empty Container Handling
- **Auto-remove + rebalance**: When all apps in a container are closed, remove the empty container and redistribute space to siblings
- Brief animation shows what happened during rebalance
- Maintains clean layout without empty spaces

### Layout Save Format
- **JSON**: Use JSON for layout persistence files
- Human-readable and easy to hand-edit
- Version field included for future migration

## Features

### Tiling Layout

**Automatic Tiling:**
- New apps tile automatically
- Split horizontal or vertical
- No overlapping windows
- Efficient space usage

**Container System:**
- Horizontal containers
- Vertical containers
- Nested layouts
- Tabbed containers within tiles

### Split Operations

**Split Commands:**
- `$mod+h`: Split horizontal
- `$mod+v`: Split vertical
- `$mod+t`: Tabbed in current tile
- `$mod+s`: Stacked in current tile

**Resize:**
- `$mod+arrow`: Resize current container
- Fine-grained pixel control
- Proportional resizing

### Container Navigation

**Focus Movement:**
- `$mod+h/j/k/l`: Focus left/down/up/right
- `$mod+a`: Focus parent container
- `$mod+d`: Focus child container
- Cycle focus within container

**Window Movement:**
- `$mod+Shift+h/j/k/l`: Move window
- Swap with adjacent windows
- Move to different containers

### Workspaces

**Multiple Workspaces:**
- Numbered workspaces (1-10)
- Named workspaces
- Workspace per monitor (future)

**Workspace Commands:**
- `$mod+1-0`: Switch to workspace
- `$mod+Shift+1-0`: Move window to workspace
- Workspace indicators in status bar

### Scratchpad

**Floating Scratchpad:**
- Hide apps to scratchpad
- Quick toggle visibility
- Multiple scratchpad items
- Cycle through scratchpad

### Layout Features

**Layout Modes:**
- Splith: Horizontal splits
- Splitv: Vertical splits
- Tabbed: Tabs in container
- Stacked: Stacked with titles

**Layout Memory:**
- Remember layout per workspace
- Restore on restart
- Save named layouts

## Data Model

```rust
pub struct TiledShell {
    pub workspaces: Vec<Workspace>,
    pub active_workspace: usize,
    pub scratchpad: Vec<AppId>,
    pub config: TiledConfig,
}

pub struct Workspace {
    pub id: u32,
    pub name: String,
    pub root: Container,
    pub focused: Option<ContainerId>,
}

pub enum Container {
    Split {
        id: ContainerId,
        direction: Direction,
        children: Vec<Container>,
        ratios: Vec<f32>,
    },
    Tabbed {
        id: ContainerId,
        children: Vec<Container>,
        active: usize,
    },
    Stacked {
        id: ContainerId,
        children: Vec<Container>,
        active: usize,
    },
    App {
        id: ContainerId,
        app: AppId,
    },
}

pub enum Direction {
    Horizontal,
    Vertical,
}
```

## Views

**Main View:**
- Tiled app windows
- Container borders
- Focus indicator

**Status Bar:**
- Workspace indicators
- Active window title
- Notification marquee
- System status

**Mode Indicator:**
- Normal mode
- Resize mode
- Move mode

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Space` | Command prefix (configurable) |
| `$mod+Enter` | Launch app picker |
| `$mod+q` | Close focused app |
| `$mod+h/j/k/l` | Focus direction |
| `$mod+Shift+h/j/k/l` | Move window |
| `$mod+v` | Split vertical |
| `$mod+b` | Split horizontal |
| `$mod+w` | Tabbed layout |
| `$mod+s` | Stacked layout |
| `$mod+e` | Toggle split direction |
| `$mod+f` | Toggle fullscreen |
| `$mod+1-0` | Switch workspace |
| `$mod+Shift+1-0` | Move to workspace |
| `$mod+r` | Enter resize mode |
| `$mod+minus` | Scratchpad hide |
| `$mod+Shift+minus` | Scratchpad show |
| `$mod+Shift+r` | Reload config |
| `$mod+Shift+e` | Exit shell |

## Configuration

```toml
# ~/.config/tui-shell-tiled/config.toml
[general]
mod_key = "ctrl+space"
default_layout = "splith"
focus_follows_mouse = false
mouse_warping = true

[gaps]
inner = 1
outer = 0

[borders]
width = 1
focused_color = "accent"
unfocused_color = "muted"

[workspaces]
names = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]

[status_bar]
position = "bottom"
height = 1
show_workspaces = true
show_title = true
show_notifications = true

[startup]
apps = ["habit-tracker", "task-manager"]
layout_file = "~/.config/tui-shell-tiled/layout.json"

[theme]
name = "catppuccin-mocha"
```

## Layout Persistence

**Save Layout:**
```json
{
  "workspaces": [
    {
      "id": 1,
      "name": "main",
      "layout": {
        "type": "split",
        "direction": "horizontal",
        "ratios": [0.5, 0.5],
        "children": [
          { "type": "app", "name": "task-manager" },
          {
            "type": "split",
            "direction": "vertical",
            "ratios": [0.6, 0.4],
            "children": [
              { "type": "app", "name": "note-manager-folder" },
              { "type": "app", "name": "time-tracker" }
            ]
          }
        ]
      }
    }
  ]
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

## i3 Compatibility

Where possible, commands mirror i3:
- Same split/focus semantics
- Similar keybinding style
- Workspace numbering
- Container nesting model

Users familiar with i3/sway will feel at home.
