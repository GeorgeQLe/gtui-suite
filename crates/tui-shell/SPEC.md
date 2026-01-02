# tui-shell

Optional TUI OS wrapper for multi-app orchestration.

## Purpose

Provide a unified shell environment for running multiple TUI apps simultaneously with shared notifications, context switching, and background task coordination.

## Architecture Decisions

### App Launch Model
- **Hybrid approach**: Apps declare their preferred launch mode
  - **In-process plugins**: For tight integration, shared memory, faster IPC
  - **Separate processes**: For isolation, crash safety, resource limits
- Apps specify preference in manifest; shell respects it

### IPC Protocol
- **Unix domain sockets**: Works for both in-process and separate process apps
- Future-proof for non-Rust apps
- JSON message format over socket for interoperability

### Input Routing
- **Focused app owns terminal input**: Active app reads terminal directly
- Shell captures control via **single prefix key** (Ctrl+Space default, configurable)
- After prefix key, next key is interpreted as shell command

### Window Boundaries
- **Sandboxed buffer per app**: Each app renders to its own buffer
- Shell composites buffers with proper clipping
- Apps don't need to know their boundaries

### Session Management
- **Full session restore** by default
- Save layout + app state on exit
- Apps implement `SessionState` trait for state serialization
- Per-app and per-user configurable (can disable)

### Workspace Model
- **Multi-workspace**: Apps can be visible in multiple workspaces
- "Sticky" windows concept - pin apps across all workspaces
- Workspace membership stored per-app-instance

### Crash Handling
- **Crash dialog**: Show modal with crash info
- Options: Restart app, Dismiss, View crash log
- Per-app auto-restart option with exponential backoff

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  TUI Shell                                                   │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Compositor (sandboxed app buffers)                     ││
│  │  ┌─────────────────┐  ┌─────────────────┐               ││
│  │  │  App 1 Buffer   │  │  App 2 Buffer   │               ││
│  │  │  (git-client)   │  │  (log-viewer)   │               ││
│  │  │                 │  │                 │               ││
│  │  └─────────────────┘  └─────────────────┘               ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Notification Panel (priority queue + expandable)        ││
│  │  [!] Error: Build failed ← [i] Push complete ← ...      ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Features

### App Lifecycle Management

```rust
pub struct AppManager {
    apps: HashMap<AppId, AppHandle>,
    running: Vec<AppId>,
    focused: Option<AppId>,
    sessions: SessionStore,
}

pub struct AppHandle {
    id: AppId,
    manifest: AppManifest,
    launch_mode: LaunchMode,
    buffer: AppBuffer,  // Sandboxed render buffer
    ipc: IpcChannel,
    workspaces: HashSet<WorkspaceId>,  // Multi-workspace membership
}

pub enum LaunchMode {
    InProcess { plugin: Box<dyn AppPlugin> },
    Subprocess { process: Child, socket: UnixStream },
}

pub struct AppManifest {
    pub name: String,
    pub preferred_launch: PreferredLaunch,
    pub supports_session: bool,
    pub auto_restart: bool,
    pub restart_backoff_ms: u64,
}

pub enum PreferredLaunch {
    InProcess,    // Load as dynamic library
    Subprocess,   // Spawn as child process
    Either,       // Shell decides based on conditions
}

impl AppManager {
    pub fn launch(&mut self, app: &str, args: &[&str]) -> Result<AppId>;
    pub fn suspend(&mut self, id: AppId) -> Result<()>;
    pub fn resume(&mut self, id: AppId) -> Result<()>;
    pub fn kill(&mut self, id: AppId) -> Result<()>;
    pub fn focus(&mut self, id: AppId) -> Result<()>;
    pub fn list_running(&self) -> &[AppId];

    // Session management
    pub fn save_session(&self) -> Result<Session>;
    pub fn restore_session(&mut self, session: &Session) -> Result<()>;

    // Multi-workspace
    pub fn add_to_workspace(&mut self, app: AppId, workspace: WorkspaceId);
    pub fn remove_from_workspace(&mut self, app: AppId, workspace: WorkspaceId);
    pub fn set_sticky(&mut self, app: AppId, sticky: bool);
}
```

### Unified Notifications

Priority queue with expandable panel:

```rust
pub struct NotificationQueue {
    notifications: VecDeque<Notification>,
    config: NotificationConfig,
    panel_expanded: bool,
}

pub struct Notification {
    pub id: NotificationId,
    pub source: String,       // App name
    pub level: NotificationLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub action: Option<NotificationAction>,
    pub priority: u8,         // Higher = stays visible longer
}

pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    pub fn default_priority(&self) -> u8 {
        match self {
            Self::Info => 1,
            Self::Success => 2,
            Self::Warning => 3,
            Self::Error => 4,  // Highest priority, stays longest
        }
    }
}

pub struct NotificationAction {
    pub label: String,
    pub command: IpcMessage,  // Sent back to source app when triggered
}

pub struct NotificationConfig {
    pub show_level: NotificationLevel, // Minimum level to display
    pub max_visible: usize,            // In collapsed marquee
    pub max_history: usize,            // In expanded panel
    pub auto_dismiss_secs: HashMap<NotificationLevel, u64>,
    pub marquee_speed: MarqueeSpeed,
}

impl NotificationQueue {
    pub fn push(&mut self, notif: Notification);
    pub fn dismiss(&mut self, id: NotificationId);
    pub fn toggle_panel(&mut self);
    pub fn get_visible(&self) -> Vec<&Notification>;  // Priority-sorted
    pub fn get_history(&self) -> Vec<&Notification>;
}
```

### Context Switching

Multiple methods to switch between apps:

```rust
pub enum SwitchMethod {
    Recent,           // Prefix+Tab style (most recent first)
    Numbered(u8),     // Prefix+1 through Prefix+9
    FuzzySearch,      // Command palette style
    Workspace(String), // Named workspace groups
}

pub struct Switcher {
    history: Vec<AppId>,
    workspaces: HashMap<WorkspaceId, Workspace>,
}

pub struct Workspace {
    pub name: String,
    pub apps: Vec<AppId>,  // Apps in this workspace
}
```

### Inter-Process Communication

Unix domain sockets with JSON messages:

```rust
pub struct IpcChannel {
    socket: UnixStream,
    pending: VecDeque<IpcMessage>,
}

#[derive(Serialize, Deserialize)]
pub enum IpcMessage {
    // Shell -> App
    Focus,
    Blur,
    Resize { width: u16, height: u16 },
    SessionSave,
    SessionRestore { state: serde_json::Value },

    // App -> Shell
    Notification(Notification),
    RequestFocus,
    Data { key: String, value: serde_json::Value },
    Command { name: String, args: Vec<String> },

    // Bidirectional
    Ping,
    Pong,
}

impl IpcChannel {
    pub fn send(&mut self, msg: IpcMessage) -> Result<()>;
    pub fn recv(&mut self) -> Result<Option<IpcMessage>>;
    pub fn recv_blocking(&mut self) -> Result<IpcMessage>;
}
```

### Session State

Apps implement trait for session persistence:

```rust
pub trait SessionState {
    fn save_state(&self) -> Result<serde_json::Value>;
    fn restore_state(&mut self, state: serde_json::Value) -> Result<()>;
}

pub struct Session {
    pub layout: LayoutState,
    pub apps: Vec<AppSession>,
    pub focused: Option<AppId>,
    pub workspaces: Vec<Workspace>,
}

pub struct AppSession {
    pub app_name: String,
    pub args: Vec<String>,
    pub state: Option<serde_json::Value>,
    pub workspace_memberships: Vec<WorkspaceId>,
}
```

### Background Task Coordination

```rust
pub struct TaskCoordinator {
    tasks: HashMap<TaskId, TaskInfo>,
}

pub struct TaskInfo {
    pub app: AppId,
    pub name: String,
    pub progress: Option<f32>,
    pub status: TaskStatus,
}

pub enum TaskStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}
```

### App Launcher

Both CommandPalette integration AND dedicated launcher:

```rust
pub struct AppLauncher {
    registry: AppRegistry,
    recent: Vec<String>,
    categories: HashMap<String, Vec<AppMeta>>,
}

pub struct AppMeta {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub icon: Option<String>,  // Nerd Font icon
    pub category: String,
    pub launch_count: u64,
}

impl AppLauncher {
    // Dedicated launcher with browsing
    pub fn show_launcher(&mut self);

    // CommandPalette integration
    pub fn get_commands(&self) -> Vec<Command>;  // For tui-widgets CommandPalette
}
```

## Shell Variants

This crate provides core logic. Separate binaries implement layout:

### tui-shell-tiled
- **Binary split** algorithm (i3/sway-style tree structure)
- Split horizontal (prefix+h) / vertical (prefix+v)
- Resize panes (prefix+arrows)
- Named workspaces (prefix+1-9)
- Container nesting

### tui-shell-floating
- Overlapping windows with borders
- **Focus-follows-mouse** for z-order
- Move windows (prefix+arrows)
- Resize (prefix+shift+arrows)
- Minimize/maximize support

### tui-shell-tabbed
- Tab bar at top
- Each tab can contain splits
- Tab groups
- Reorder tabs (prefix+shift+left/right)
- Pin tabs

### tui-shell-fullscreen
- One app visible at full size
- Quick switcher overlay (prefix+space)
- Minimal chrome
- Notification ticker only

## Prefix Key System

Single configurable prefix key for shell commands:

```rust
pub struct PrefixKeyHandler {
    prefix: KeyBinding,  // Default: Ctrl+Space
    timeout_ms: u64,     // Time to wait for next key
    pending: bool,       // Prefix was pressed, waiting for command
}

// After prefix key, these keys trigger shell actions:
// Tab       - Switch to recent app
// 1-9       - Switch to numbered app
// p         - Open command palette
// l         - Open app launcher
// n         - Toggle notification panel
// w         - Workspace switcher
// h/v       - Split horizontal/vertical (tiled)
// arrows    - Navigate between panes
// q         - Close focused app
// ?         - Shell help
```

## Configuration

```toml
# ~/.config/tui-shell/config.toml
[shell]
variant = "tiled"  # tiled, floating, tabbed, fullscreen
prefix_key = "ctrl+space"
prefix_timeout_ms = 500

[session]
enabled = true
auto_save = true
save_interval_secs = 300  # Auto-save every 5 minutes

[notifications]
show_level = "info"  # info, success, warning, error
max_visible = 3
max_history = 100
marquee_speed = "normal"  # slow, normal, fast

[notifications.auto_dismiss]
info = 5
success = 5
warning = 10
error = 0  # Never auto-dismiss errors

[workspaces]
default = ["git-client", "log-viewer"]
monitoring = ["process-monitor", "network-monitor"]
comms = ["chat-client", "email-client"]

[startup]
apps = ["git-client"]
workspace = "default"
restore_session = true

[crash]
show_dialog = true
auto_restart_default = false
backoff_initial_ms = 1000
backoff_max_ms = 30000
```

## Dependencies

```toml
[dependencies]
ratatui = { workspace = true }
crossterm = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
chrono = { workspace = true }
directories = "5"
```
