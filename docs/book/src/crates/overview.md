# Shared Crates Overview

TUI Suite provides 6 shared crates that form the foundation for all applications. These crates ensure consistent UX, reduce code duplication, and provide battle-tested components.

## Crate Dependency Graph

```
┌─────────────────────────────────────────────────────────┐
│                     Applications                         │
├─────────────────────────────────────────────────────────┤
│  tui-shell (orchestration)                              │
├──────────────┬──────────────┬──────────────────────────┤
│ tui-widgets  │  tui-theme   │  tui-keybinds            │
├──────────────┴──────────────┴──────────────────────────┤
│              tui-testing    │  tui-plugins             │
└─────────────────────────────────────────────────────────┘
```

## Crates Summary

### tui-widgets

Reusable UI components with consistent behavior and accessibility support.

**Components:**
- `DataTable` - Sortable, filterable table with virtual scrolling
- `TreeView` - Expandable hierarchical view with lazy loading
- `FormBuilder` - Declarative form construction with validation
- `CommandPalette` - Fuzzy-search command launcher

### tui-theme

Full-featured theming engine supporting multiple color depths and accessibility.

**Features:**
- Semantic color system (bg_primary, accent, error, etc.)
- Built-in presets (Catppuccin, Nord, Dracula, Gruvbox, etc.)
- High contrast accessibility themes
- Runtime theme switching

### tui-keybinds

Flexible keybinding configuration with vim/emacs presets.

**Features:**
- Context-aware bindings (Normal, Insert, Visual, Command modes)
- User overrides via TOML config
- Conflict detection
- Multiple notation styles (`ctrl+s`, `C-s`, `<C-s>`)

### tui-shell

Multi-app orchestration for running apps together.

**Features:**
- App lifecycle management
- Workspace support
- Inter-process communication
- Multiple layout modes (tiled, floating, tabbed, fullscreen)

### tui-testing

Testing utilities for TUI applications.

**Features:**
- Snapshot testing with golden files
- Input simulation (keyboard/mouse sequences)
- Virtual headless terminal
- Property-based testing

### tui-plugins

Multi-backend plugin system for extensibility.

**Backends:**
- Lua (via mlua) - Primary backend
- WASM (via wasmtime) - Phase 2
- Native (.so/.dll) - Optional

## Using Shared Crates

In your app's `Cargo.toml`:

```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
tui-keybinds = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
```

See [Using Shared Crates](../guides/using-shared-crates.md) for detailed usage examples.
