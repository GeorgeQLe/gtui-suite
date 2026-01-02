# tui-keybinds

Keybinding configuration and management system.

## Purpose

Provide consistent, configurable keyboard shortcuts across all apps with conflict detection, context-aware bindings, and multiple input schemes.

## Architecture Decisions

### Key Sequence Support
- **Leader key pattern by default**: Single leader key starts a chord, followed by single key
- **User-configurable scheme**: Users can select full chord support (vim 'gg'), emacs style ('C-x C-s'), or simple modifiers only
- **Leader key**: Configurable only (no default) - user must explicitly set their preferred leader

### Macro System
- **Built-in macro recording**: Record key sequences, save named macros, replay
- **TOML persistence**: Macros saved to `~/.config/{app}/macros.toml` for human editing and sharing
- Like vim's 'q' recording but with persistent storage

### Conflict Resolution
- **Error on conflict**: Refuse to load config if user binding conflicts with default
- Force explicit unbinding if user wants to reassign a key

### Context Conditions
- **Expression + context hybrid**:
  - Simple context enum for basic cases (Normal, Insert, Dialog)
  - Boolean expression syntax for advanced conditions

### Keymap Presets
- **Full keymaps**: Ship complete vim, emacs, default presets
- Each preset provides a full vim-like or emacs-like experience
- Users can extend/override any preset

## Features

### Key Sequences

```rust
pub enum KeyScheme {
    /// Only Ctrl+Key, Alt+Key, Shift+Key. No multi-press.
    SimpleModifiers,

    /// Single leader key (configurable) then single key
    LeaderKey { leader: KeyCode, timeout_ms: u64 },

    /// Full chord support like vim's 'gg' or emacs 'C-x C-s'
    FullChords { timeout_ms: u64 },
}

pub struct KeySequence {
    pub keys: Vec<KeyBinding>,
}

// Examples:
// LeaderKey: Space -> g -> s  (git stage)
// FullChords: g -> g (go to top), C-x -> C-s (save)
```

### Default Keymaps

Each app defines default keybindings:

```rust
pub struct Keymap {
    pub bindings: HashMap<KeySequence, Action>,
    pub contexts: HashMap<Context, HashMap<KeySequence, Action>>,
}

pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers, // Ctrl, Alt, Shift
}
```

### User Overrides

Users can override bindings via config:

```toml
# ~/.config/app-name/keybinds.toml
[scheme]
type = "leader"  # "simple", "leader", or "chords"
leader_key = "space"
timeout_ms = 500

[global]
quit = "ctrl+q"
help = "?"
command_palette = "ctrl+p"

[context.table]
next_row = "j"
prev_row = "k"
select = "enter"
delete = "d"

[context.editor]
save = "ctrl+s"
undo = "ctrl+z"
redo = "ctrl+shift+z"

# Unbind a default key to reassign it
[unbind]
keys = ["ctrl+w"]  # Explicitly unbind before reassigning
```

### Context-Aware Bindings

Simple contexts plus expression syntax:

```rust
pub enum Context {
    Normal,
    Insert,
    Visual,
    Command,
    Popup,
    Dialog,
    Custom(String),
}
```

**Canonical Location**: This `Context` enum is defined in tui-keybinds and re-exported to other crates (tui-widgets, tui-shell). All context-aware functionality uses this shared type.

```rust
pub struct ContextCondition {
    pub context: Context,
    pub when: Option<String>,  // Optional expression: "editorFocus && !readOnly"
}

impl Keymap {
    pub fn get_action(&self, key: &KeySequence, condition: &ContextCondition) -> Option<&Action>;
}
```

Expression syntax for 'when' clauses:
```toml
[[bindings]]
key = "ctrl+s"
command = "save"
when = "editorFocus && !readOnly && hasChanges"
```

Supported operators: `&&`, `||`, `!`, parentheses for grouping.

### Conflict Detection

Strict conflict handling:

```rust
pub struct ConflictReport {
    pub conflicts: Vec<Conflict>,
}

pub struct Conflict {
    pub key: KeySequence,
    pub actions: Vec<(String, Action)>, // (source, action)
    pub severity: ConflictSeverity,
}

pub enum ConflictSeverity {
    Warning,  // Same key, different contexts (may be intentional)
    Error,    // Same key, same context (config rejected)
}

// Config loading fails on Error severity conflicts
impl KeybindManager {
    pub fn load_user_config(&mut self, path: &Path) -> Result<(), ConflictError>;
}
```

### Macro Recording

Built-in macro system with TOML persistence:

```rust
pub struct MacroManager {
    macros: HashMap<String, Macro>,
    recording: Option<MacroRecording>,
}

pub struct Macro {
    pub name: String,
    pub keys: Vec<KeyEvent>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl MacroManager {
    pub fn start_recording(&mut self, name: &str);
    pub fn stop_recording(&mut self) -> Option<Macro>;
    pub fn record_key(&mut self, key: KeyEvent);
    pub fn play(&self, name: &str) -> Option<impl Iterator<Item = KeyEvent>>;
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn load(&mut self, path: &Path) -> Result<()>;
}
```

Macro file format:
```toml
# ~/.config/app/macros.toml
[[macros]]
name = "insert-date"
description = "Insert today's date at cursor"
keys = [
    { key = "i" },
    { text = "2024-12-31" },
    { key = "escape" }
]
created_at = "2024-12-31T10:00:00Z"

[[macros]]
name = "wrap-parens"
keys = [
    { key = "i" },
    { char = "(" },
    { key = "escape" },
    { key = "l" },
    { key = "a" },
    { char = ")" },
    { key = "escape" }
]
```

### Action Groups & Presets

Named groups with preset configurations:

```rust
pub struct ActionGroup {
    pub name: String,
    pub actions: Vec<String>,
    pub presets: HashMap<String, HashMap<String, KeySequence>>,
}
```

Configuration:
```toml
[groups.navigation]
actions = ["move_up", "move_down", "move_left", "move_right", "page_up", "page_down"]

[groups.navigation.presets.vim]
move_up = "k"
move_down = "j"
move_left = "h"
move_right = "l"
page_up = "ctrl+u"
page_down = "ctrl+d"

[groups.navigation.presets.arrows]
move_up = "up"
move_down = "down"
move_left = "left"
move_right = "right"
page_up = "pageup"
page_down = "pagedown"

# User selects preset:
[keybinds]
navigation = "vim"  # Apply vim preset to all navigation actions
```

### Keymap Presets

Full built-in presets:

```rust
pub enum KeymapPreset {
    Default,   // Modern, intuitive keys
    Vim,       // Full vim-like experience
    Emacs,     // Emacs key conventions
}

pub fn load_preset(preset: KeymapPreset) -> Keymap;
```

Presets include complete mappings for:
- Navigation (movement, scrolling)
- Editing (insert, delete, undo/redo)
- Selection (visual mode for vim)
- Search (find, replace)
- File operations
- Window/pane management

### Special Key Support

```rust
pub enum SpecialKeyHandling {
    /// Only support standard 104-key layout
    Standard,

    /// Support extended keys where crossterm supports
    Extended,  // F13-F24, media keys

    /// Unknown keys passed as raw scancodes
    Passthrough { handler: Box<dyn Fn(u32) -> Option<Action>> },
}

// Default: Passthrough mode - apps can handle special keys if desired
```

### Key Notation

Support multiple notation styles with configurable display:

```rust
// All equivalent input formats:
"ctrl+s"
"C-s"
"<C-s>"
"Ctrl+S"

// Special keys:
"enter", "escape", "tab", "space"
"up", "down", "left", "right"
"home", "end", "pageup", "pagedown"
"f1" through "f12"
"backspace", "delete"
```

Display format configuration:
```rust
pub enum KeyDisplayFormat {
    /// Unicode symbols: ⌘S, ⌃P, ⇧Tab
    Symbolic,

    /// Text labels: Ctrl+S, Alt+P, Shift+Tab
    Text,
}

pub struct KeyDisplayConfig {
    pub format: KeyDisplayFormat,
    pub fallback_to_text: bool,  // If symbolic fails
}
```

### Command Mapping

Map keybindings to named actions:

```rust
pub trait Action: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    fn description(&self) -> &str;  // For help screen
    fn group(&self) -> Option<&str>;  // For help screen grouping
    fn execute(&self, ctx: &mut AppContext) -> Result<()>;
}

// Registration
keymap.register("save", SaveAction::new());
keymap.bind(KeyBinding::ctrl('s'), "save");
```

## API

```rust
pub struct KeybindManager {
    scheme: KeyScheme,
    default_keymap: Keymap,
    preset: Option<KeymapPreset>,
    user_keymap: Keymap,
    groups: Vec<ActionGroup>,
    macros: MacroManager,
    current_context: Context,
    display_config: KeyDisplayConfig,
}

impl KeybindManager {
    pub fn new(scheme: KeyScheme, preset: Option<KeymapPreset>) -> Self;
    pub fn load_user_config(&mut self, path: &Path) -> Result<(), ConflictError>;
    pub fn set_context(&mut self, ctx: Context);
    pub fn handle_key(&self, key: KeyEvent) -> Option<Action>;
    pub fn get_binding_for(&self, action_id: &str) -> Option<KeySequence>;
    pub fn check_conflicts(&self) -> ConflictReport;

    // Macro operations
    pub fn start_macro(&mut self, name: &str);
    pub fn stop_macro(&mut self);
    pub fn play_macro(&self, name: &str) -> Option<Vec<KeyEvent>>;
}
```

## Help Display

**Hybrid generation**: Auto-generated list with app-customizable descriptions and grouping:

```rust
pub fn generate_help(&self, context: &Context) -> HelpScreen;

pub struct HelpScreen {
    pub groups: Vec<HelpGroup>,
}

pub struct HelpGroup {
    pub name: String,
    pub description: Option<String>,  // App can provide
    pub entries: Vec<HelpEntry>,
}

pub struct HelpEntry {
    pub key: String,           // "Ctrl+S" or "⌃S" based on config
    pub action: String,        // "save"
    pub description: String,   // "Save the current file"
}

// App can customize:
help_generator.set_group_description("navigation", "Move around the document");
help_generator.set_action_description("save", "Save changes to disk");
```

## Configuration

Full keybinds config example:

```toml
# ~/.config/app-name/keybinds.toml

[scheme]
type = "leader"
leader_key = "space"
timeout_ms = 500

[display]
format = "symbolic"  # or "text"
fallback_to_text = true

[preset]
base = "vim"  # Start with vim preset

[groups]
navigation = "vim"  # Use vim navigation preset

[unbind]
# Explicitly unbind keys before reassigning
keys = ["ctrl+w", "ctrl+e"]

[global]
quit = "ctrl+q"
help = "?"
command_palette = "ctrl+p"

[context.normal]
# Normal mode bindings
gg = "go_to_top"
G = "go_to_bottom"

[context.insert]
escape = "exit_insert"

[context.dialog]
enter = "confirm"
escape = "cancel"

# Advanced when expressions
[[bindings]]
key = "ctrl+s"
command = "save"
when = "editorFocus && !readOnly"
```

## Dependencies

```toml
[dependencies]
crossterm = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
```
