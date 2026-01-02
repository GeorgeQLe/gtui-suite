# tui-shell-tabbed

Tab-based window manager with optional splits.

## Architecture Decisions

### Tab Overflow Handling
- **Scroll arrows + active visible**: When tabs exceed bar width, show scroll arrows
- Active tab is always kept visible in the tab bar
- Standard browser-like behavior users expect

### Session Save Format
- **JSON**: Use JSON for session persistence
- Human-readable, stores tab groups, splits, and pinned tabs
- Easy to back up and share configurations

## Features

### Tab Bar

**Tab Navigation:**
- Horizontal tab bar
- Active tab highlighted
- Tab close buttons
- Tab reordering

**Tab Display:**
- App icon/indicator
- App name (truncated)
- Modified indicator
- Close button [x]

### Tab Operations

**Create/Close:**
- `$mod+t`: New tab (app picker)
- `$mod+w`: Close current tab
- Middle-click tab to close
- Confirm close for unsaved

**Navigate:**
- `$mod+Tab`: Next tab
- `$mod+Shift+Tab`: Previous tab
- `$mod+1-9`: Jump to tab N
- Click tab to focus

**Reorder:**
- `$mod+Shift+Left/Right`: Move tab
- Drag and drop tabs
- Pin tabs to start

### Splits Within Tabs

**Split Current Tab:**
- `$mod+v`: Vertical split
- `$mod+h`: Horizontal split
- New app in split

**Split Navigation:**
- `$mod+arrow`: Focus split pane
- `$mod+Shift+arrow`: Resize split
- `$mod+z`: Toggle zoom (maximize pane)

**Split Layout:**
- Equal splits
- Adjustable ratios
- Nested splits allowed

### Tab Groups

**Group Management:**
- Create tab groups
- Color-coded groups
- Collapse/expand groups
- Name groups

**Group Operations:**
- Move tab to group
- Move entire group
- Close all in group

### Pinned Tabs

**Pin Behavior:**
- Pinned tabs stay left
- Show only icon (compact)
- Cannot be closed accidentally
- Always present on restart

### Session Management

**Save/Restore:**
- Save tab arrangement
- Remember splits per tab
- Restore on startup
- Named sessions

## Data Model

```rust
pub struct TabbedShell {
    pub tab_groups: Vec<TabGroup>,
    pub pinned_tabs: Vec<Tab>,
    pub active_tab: TabId,
    pub config: TabbedConfig,
}

pub struct TabGroup {
    pub id: GroupId,
    pub name: String,
    pub color: Color,
    pub tabs: Vec<Tab>,
    pub collapsed: bool,
}

pub struct Tab {
    pub id: TabId,
    pub layout: TabLayout,
    pub modified: bool,
    pub pinned: bool,
}

pub enum TabLayout {
    Single { app: AppId },
    Split {
        direction: Direction,
        ratios: Vec<f32>,
        children: Vec<TabLayout>,
    },
}

pub enum Direction {
    Horizontal,
    Vertical,
}
```

## Views

**Tab Bar:**
- Pinned tabs (compact)
- Tab groups (collapsible)
- Regular tabs
- New tab button [+]

**Content Area:**
- Full screen single app
- Or split panes
- Focused pane indicator

**Status Bar:**
- Active app info
- Notification marquee
- Quick actions

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Space` | Command prefix |
| `$mod+t` | New tab |
| `$mod+w` | Close tab |
| `$mod+Tab` | Next tab |
| `$mod+Shift+Tab` | Previous tab |
| `$mod+1-9` | Go to tab N |
| `$mod+0` | Go to last tab |
| `$mod+Shift+Left` | Move tab left |
| `$mod+Shift+Right` | Move tab right |
| `$mod+v` | Split vertical |
| `$mod+h` | Split horizontal |
| `$mod+arrow` | Focus pane |
| `$mod+Shift+arrow` | Resize pane |
| `$mod+z` | Zoom pane |
| `$mod+x` | Close pane |
| `$mod+p` | Pin/unpin tab |
| `$mod+g` | Create group |
| `$mod+Shift+g` | Add to group |
| `$mod+s` | Save session |
| `$mod+Shift+e` | Exit shell |

## Configuration

```toml
# ~/.config/tui-shell-tabbed/config.toml
[general]
mod_key = "ctrl+space"
confirm_close_modified = true
confirm_close_last = true

[tabs]
position = "top"
height = 1
show_index = true
max_title_length = 20
show_close = true
new_tab_button = true

[tabs.pinned]
show_name = false
width = 3

[splits]
default_ratio = 0.5
min_pane_size = 5
show_borders = true
focused_border = "accent"

[groups]
enabled = true
default_collapsed = false
colors = ["red", "green", "blue", "yellow", "magenta", "cyan"]

[sessions]
auto_save = true
save_interval_sec = 300
restore_on_start = true
session_file = "~/.local/state/tui-shell-tabbed/session.json"

[status_bar]
position = "bottom"
height = 1
show_notifications = true

[startup]
default_apps = ["task-manager"]

[theme]
name = "catppuccin-frappe"
```

## Session Format

```json
{
  "active_tab": "tab-3",
  "pinned": [
    { "id": "tab-1", "app": "habit-tracker" }
  ],
  "groups": [
    {
      "name": "Work",
      "color": "blue",
      "collapsed": false,
      "tabs": [
        {
          "id": "tab-2",
          "layout": {
            "type": "single",
            "app": "task-manager"
          }
        },
        {
          "id": "tab-3",
          "layout": {
            "type": "split",
            "direction": "horizontal",
            "ratios": [0.6, 0.4],
            "children": [
              { "type": "single", "app": "note-manager-folder" },
              { "type": "single", "app": "time-tracker" }
            ]
          }
        }
      ]
    }
  ],
  "ungrouped": [
    { "id": "tab-4", "app": "cheatsheet-browser" }
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

## Browser-Like Experience

Inspired by modern browsers and IDEs:
- Familiar tab paradigm
- Tab groups like Chrome
- Splits like VS Code
- Pinned tabs for always-on apps
- Session restore like Firefox

Users familiar with tabbed interfaces will feel comfortable immediately.
