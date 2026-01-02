# tui-plugins

Multi-backend plugin system for extending TUI Suite applications.

## Purpose

Provide a unified plugin architecture that supports multiple extension mechanisms (Lua, WASM, native), allowing users and developers to customize and extend applications without modifying core code.

## Architecture Decisions

### Multi-Backend Support
- **Three backends**: Lua (mlua), WASM (wasmtime), Native dynamic libraries
- Unified `Plugin` trait abstracts backend differences
- Apps see same interface regardless of plugin implementation language
- Backend selected at plugin load time based on file extension

### Implementation Priority
- **Phase 1: Lua** - Most accessible, fastest to implement
- **Phase 2: WASM** - Language-agnostic, secure sandbox
- **Phase 3: Native** - Maximum performance, optional advanced feature

### Plugin Discovery
- Standard locations: `~/.config/{app}/plugins/`, system-wide `/usr/share/{app}/plugins/`
- File extensions determine backend: `.lua`, `.wasm`, `.so`/`.dll`/`.dylib`
- Plugin manifest (`plugin.toml`) describes metadata and capabilities

## Plugin Trait

```rust
pub trait Plugin: Send + Sync {
    /// Unique plugin identifier
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Version string
    fn version(&self) -> &str;

    /// Capabilities this plugin provides
    fn capabilities(&self) -> &[Capability];

    /// Initialize plugin with app context
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// Clean shutdown
    fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Handle events from host application
    fn on_event(&mut self, event: &PluginEvent) -> Result<Option<PluginResponse>, PluginError>;
}

pub enum Capability {
    /// Adds commands to command palette
    Commands,
    /// Adds keybindings
    Keybindings,
    /// Adds theme colors/styles
    Theming,
    /// Adds file type handlers
    FileHandler { extensions: Vec<String> },
    /// Adds data transformers
    Transformer,
    /// Custom capability
    Custom(String),
}
```

## Lua Backend

Uses mlua for Lua 5.4 embedding.

### Sandbox Configuration
```rust
pub struct LuaSandbox {
    pub memory_limit: usize,      // Default: 10MB
    pub instruction_limit: u64,   // Default: 1_000_000
    pub allowed_modules: Vec<String>,
}
```

### Lua Plugin API
```lua
-- plugin.lua
local plugin = {}

plugin.id = "my-plugin"
plugin.name = "My Plugin"
plugin.version = "1.0.0"
plugin.capabilities = { "commands" }

function plugin.init(ctx)
    -- Initialize with app context
    ctx.log("Plugin initialized")
end

function plugin.on_event(event)
    if event.type == "command" and event.name == "my_command" then
        return { action = "notify", message = "Hello from plugin!" }
    end
    return nil
end

function plugin.shutdown()
    -- Cleanup
end

return plugin
```

### Available Lua Modules
- `tui.notify(msg)` - Show notification
- `tui.prompt(title, options)` - Show prompt dialog
- `tui.run_command(name)` - Execute app command
- `tui.get_selection()` - Get current selection
- `tui.set_clipboard(text)` - Set clipboard
- `tui.http.get(url)` - HTTP GET (if enabled)
- `tui.fs.read(path)` - Read file (sandboxed paths)

## WASM Backend

Uses wasmtime for WebAssembly execution.

### WASM Interface (WIT)
```wit
// plugin.wit
package tui:plugin@1.0.0;

interface host {
    notify: func(message: string);
    prompt: func(title: string, options: list<string>) -> option<u32>;
    log: func(level: log-level, message: string);

    enum log-level {
        debug,
        info,
        warn,
        error,
    }
}

interface plugin {
    record plugin-info {
        id: string,
        name: string,
        version: string,
        capabilities: list<string>,
    }

    get-info: func() -> plugin-info;
    init: func() -> result<_, string>;
    shutdown: func() -> result<_, string>;
    on-event: func(event: event-data) -> option<response>;

    record event-data {
        event-type: string,
        payload: string,  // JSON
    }

    record response {
        action: string,
        payload: string,  // JSON
    }
}

world tui-plugin {
    import host;
    export plugin;
}
```

### WASM Sandbox
- Memory limit: 32MB per plugin
- No direct filesystem access
- No network access (unless explicitly granted)
- CPU time limits enforced

## Native Backend

Dynamic library plugins for maximum performance.

### Native Interface
```rust
// Required exports from .so/.dll
#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo;

#[no_mangle]
pub extern "C" fn plugin_init(ctx: *const PluginContext) -> i32;

#[no_mangle]
pub extern "C" fn plugin_shutdown() -> i32;

#[no_mangle]
pub extern "C" fn plugin_on_event(event: *const PluginEvent) -> *mut PluginResponse;
```

### Security Considerations
- Native plugins are **not sandboxed** - full system access
- Only load from trusted sources
- Signature verification recommended for production
- Display clear warnings before loading native plugins

## Plugin Manifest

```toml
# plugin.toml
[plugin]
id = "my-plugin"
name = "My Plugin"
version = "1.0.0"
description = "Example plugin"
author = "Developer Name"
license = "MIT"

[capabilities]
commands = true
keybindings = true
theming = false

[backend]
type = "lua"  # or "wasm" or "native"
entry = "plugin.lua"

[permissions]
network = false
filesystem = ["~/.config/my-plugin/*"]

[dependencies]
tui-plugins = ">=1.0.0"
```

## Plugin Manager

```rust
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    backends: Backends,
}

impl PluginManager {
    /// Load all plugins from standard locations
    pub fn discover_and_load(&mut self) -> Result<Vec<String>, PluginError>;

    /// Load a specific plugin
    pub fn load(&mut self, path: &Path) -> Result<String, PluginError>;

    /// Unload a plugin
    pub fn unload(&mut self, id: &str) -> Result<(), PluginError>;

    /// Get plugin by ID
    pub fn get(&self, id: &str) -> Option<&dyn Plugin>;

    /// Broadcast event to all plugins
    pub fn broadcast(&mut self, event: &PluginEvent) -> Vec<PluginResponse>;

    /// Send event to specific plugin
    pub fn send(&mut self, id: &str, event: &PluginEvent) -> Result<Option<PluginResponse>, PluginError>;
}
```

## Integration with Apps

Apps integrate plugins through the shell or directly:

```rust
// In app initialization
let mut plugins = PluginManager::new(config);
plugins.discover_and_load()?;

// Register plugin commands with CommandPalette
for plugin in plugins.iter() {
    if plugin.capabilities().contains(&Capability::Commands) {
        for cmd in plugin.get_commands() {
            command_palette.register(cmd);
        }
    }
}

// In event loop
match event {
    Event::Key(key) => {
        // Let plugins handle first
        let responses = plugins.broadcast(&PluginEvent::Key(key));
        if responses.iter().any(|r| r.handled) {
            continue;
        }
        // Normal handling...
    }
}
```

## Dependencies

```toml
[dependencies]
mlua = { version = "0.9", features = ["lua54", "vendored"] }
wasmtime = "25"
libloading = "0.8"
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
directories = { workspace = true }
```

## Security Model

| Backend | Isolation | Performance | Ease of Dev |
|---------|-----------|-------------|-------------|
| Lua | Sandbox (memory, CPU) | Good | Excellent |
| WASM | Strong sandbox | Good | Moderate |
| Native | None (full access) | Excellent | Moderate |

### Permission System
- Plugins declare required permissions in manifest
- Host grants/denies at load time
- Undeclared capabilities are denied by default
- User can override permissions in config
