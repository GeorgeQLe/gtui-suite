# CLAUDE.md - TUI Suite Project Guide

## Project Overview

This is **TUI Suite**, a Rust monorepo containing reusable TUI (Terminal User Interface) components and 49 complete application examples. The project demonstrates production-quality Rust TUI application development with emphasis on code reuse, accessibility, and comprehensive testing.

**Repository**: https://github.com/georgeqle/tui-suite

## Current Status

| Component | Status |
|-----------|--------|
| **Shared Crates** | All 6 crates have implementations but are **disabled in workspace** (serde fixes needed) |
| **Tier 1 Apps** (4) | Fully implemented |
| **Tier 2 Apps** (10) | Fully implemented |
| **Tier 3-5 Apps** (35) | Specs complete, implementation pending |

**Known Blockers:**
- Shared crates commented out in `Cargo.toml` due to serde compatibility issues
- Once crates are fixed, enable them in workspace members

## Quick Commands

```bash
# Build the entire workspace
cargo build

# Build a specific app
cargo build -p git-client

# Run an app
cargo run -p habit-tracker

# Run tests for entire workspace
cargo test

# Run tests for a specific crate
cargo test -p tui-widgets

# Run with snapshot updates
UPDATE_SNAPSHOTS=1 cargo test

# Check all packages
cargo check --workspace

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace
```

## Project Structure

```
TUI/
├── Cargo.toml              # Workspace root with shared dependencies
├── crates/                 # 6 shared foundation libraries
│   ├── tui-widgets/        # Reusable UI components (DataTable, TreeView, FormBuilder, CommandPalette)
│   ├── tui-theme/          # Theming engine with presets and accessibility themes
│   ├── tui-keybinds/       # Keybinding configuration and management
│   ├── tui-shell/          # Multi-app orchestration shell
│   ├── tui-testing/        # Testing utilities (snapshot, input simulation, property-based)
│   └── tui-plugins/        # Multi-backend plugin system (Lua, WASM, Native)
├── apps/                   # 49 applications organized by complexity tier
│   ├── habit-tracker/      # Tier 1: Simple apps
│   ├── task-manager/       # Tier 2: Moderate apps
│   ├── docker-manager/     # Tier 3: Complex apps
│   ├── git-client/         # Tier 4: Advanced apps
│   ├── chat-client/        # Tier 5: Expert apps
│   └── tui-shell-*/        # Shell variants (tiled, floating, tabbed, fullscreen)
└── docs/book/              # Documentation structure
```

## Core Technologies

| Category | Library | Version |
|----------|---------|---------|
| TUI Framework | ratatui | 0.29 |
| Terminal Backend | crossterm | 0.28 |
| Async Runtime | tokio / async-std / smol | 1.x / 1.x / 2.x |
| Database | rusqlite | 0.32 (bundled) |
| Serialization | serde, toml, serde_json | 1.x, 0.8, 1.x |
| Error Handling | thiserror, anyhow | 2.x, 1.x |
| Git | git2 | 0.19 |
| Fuzzy Finding | nucleo-matcher, fuzzy-matcher | 0.3 |
| Testing | insta (snapshot), proptest | 1.x |
| Data Processing | polars | 0.45 |

**Rust Edition**: 2021
**License**: MIT

## Shared Crates Reference

### tui-widgets
Reusable UI components with consistent UX. See `crates/tui-widgets/SPEC.md`.

- **DataTable**: Sortable, filterable table with virtual scrolling (10k+ rows)
- **TreeView**: Expandable hierarchical view with lazy loading
- **FormBuilder**: Declarative form construction with validation
- **CommandPalette**: Fuzzy-search command launcher (nucleo-matcher)

### tui-theme
Full-featured theming engine. See `crates/tui-theme/SPEC.md`.

- Semantic colors (bg_primary, accent, error, etc.)
- Built-in presets: Base16, Catppuccin, Nord, Solarized, Gruvbox, Dracula, One Dark
- High contrast accessibility themes
- Runtime theme switching
- User themes via TOML config

### tui-keybinds
Keybinding configuration and management. See `crates/tui-keybinds/SPEC.md`.

- Context-aware bindings (Normal, Insert, Visual, Command, Popup, Dialog)
- User override via `~/.config/app-name/keybinds.toml`
- Conflict detection with severity levels
- Multiple notation styles: `ctrl+s`, `C-s`, `<C-s>`, `Ctrl+S`
- Shell prefix key support (default: Ctrl+Space)

### tui-shell
Multi-app orchestration. See `crates/tui-shell/SPEC.md`.

- App lifecycle management (launch, suspend, resume, kill, focus)
- Unified notification marquee
- Context switching (Alt+Tab, Alt+1-9, fuzzy search, workspaces)
- Inter-process communication
- 4 shell variants: tiled, floating, tabbed, fullscreen

### tui-testing
Testing utilities. See `crates/tui-testing/SPEC.md`.

- Snapshot testing with golden files
- Input simulation (keyboard/mouse sequences)
- Test terminal (virtual headless terminal)
- Property-based testing with proptest
- Async test harness

### tui-plugins
Multi-backend plugin system. See `crates/tui-plugins/SPEC.md`.

- Lua plugins via mlua (primary backend)
- WASM plugins via wasmtime (phase 2)
- Native plugins via .so/.dll (optional)
- Unified `Plugin` trait abstracts backend differences
- Sandboxed execution environment

## Application Tiers

| Tier | Complexity | Examples |
|------|------------|----------|
| 1 | Simple | habit-tracker, flashcard-trainer, time-tracker |
| 2 | Moderate | task-manager, note-manager-*, personal-wiki |
| 3 | Complex | docker-manager, file-manager, log-viewer, ssh-hub |
| 4 | Advanced | git-client, api-tester, k8s-dashboard, db-client |
| 5 | Expert | chat-client (multi-protocol), email-client, log-anomaly-detector |

## Coding Patterns

### App Structure
Each app follows a consistent structure:
```
apps/app-name/
├── Cargo.toml      # Package manifest using workspace dependencies
├── SPEC.md         # Detailed feature specification
└── src/
    └── main.rs     # Entry point
```

### Using Workspace Dependencies
```toml
[dependencies]
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
tokio = { workspace = true }
```

### Standard Keybindings
Common across all apps:
- `q` / `Ctrl+Q` - Quit
- `?` - Help
- `Ctrl+P` - Command palette
- `j/k` or arrows - Navigate
- `Enter` - Select/confirm
- `Esc` - Cancel/back

### Configuration Pattern
Apps store user config in `~/.config/app-name/`:
```toml
# config.toml
[theme]
name = "catppuccin-mocha"
variant = "compact"

[keybinds]
# Override default bindings
```

## Testing

### Snapshot Testing
```rust
#[test]
fn test_table_rendering() {
    let mut test = SnapshotTest::new("table_basic", 80, 24);
    let table = DataTable::new(columns, data);
    test.assert_snapshot(table);
}
```

Update snapshots: `UPDATE_SNAPSHOTS=1 cargo test`

### Input Simulation
```rust
let input = InputSequence::new()
    .down().down()
    .enter()
    .text("value")
    .ctrl('s');
app.process_input(&input);
```

### Property-Based Testing
```rust
proptest! {
    #[test]
    fn navigation_never_panics(events in generators::navigation_sequence(100)) {
        let mut table = DataTable::new(columns, data);
        for event in events {
            table.handle_key(event);
        }
    }
}
```

## Accessibility Features

All widgets support:
- Screen reader hints (where terminal supports)
- High contrast mode compatibility
- Large text mode (reduced density)
- Keyboard-only navigation
- Colorblind-friendly palettes

## Key Files

- **Workspace Cargo.toml**: `/Cargo.toml` - All shared dependencies and workspace members
- **Crate SPECs**: `/crates/*/SPEC.md` - Detailed specifications for each shared crate
- **App SPECs**: `/apps/*/SPEC.md` - Feature specifications for each application

## Adding a New App

1. Create directory: `apps/new-app/`
2. Add `Cargo.toml` using workspace dependencies
3. Add `SPEC.md` documenting features
4. Add to workspace members in root `Cargo.toml`
5. Implement using shared crates (tui-widgets, tui-theme, tui-keybinds)

## Common Patterns to Follow

1. **Use shared crates** - Don't reinvent widgets, theming, or keybindings
2. **Write SPEC.md first** - Document features before implementing
3. **Include tests** - Snapshot tests for rendering, property tests for logic
4. **Support theming** - Use semantic colors from tui-theme
5. **Configurable keybinds** - Use tui-keybinds for all keyboard shortcuts
6. **Accessibility first** - Ensure keyboard navigation and high contrast support

---

## Cross-Cutting Standards

These patterns apply to ALL apps in the suite.

### Data Integrity

| Pattern | Implementation |
|---------|----------------|
| **Schema Migration** | Version table in SQLite + numbered migration scripts. Check version on startup, run pending migrations. |
| **Concurrent Access** | Advisory file lock (flock) on database. Second instance opens in read-only mode with banner. |
| **Crash Recovery** | SQLite WAL mode + transactions wrap all writes. Automatic recovery on restart. |
| **Config Validation** | Interactive repair wizard for invalid TOML. Log errors, prompt user to fix. |

### Session & State

| Pattern | Implementation |
|---------|----------------|
| **Session Persistence** | Explicit save (Ctrl+S), prompt on quit if unsaved. No auto-save. |
| **Network Reconnection** | Exponential backoff (1s→2s→4s→max 60s), status indicator in UI. |
| **Token Refresh** | Background refresh 5 minutes before expiry. Store expiry timestamp. |

### Configuration

| Pattern | Implementation |
|---------|----------------|
| **Paths** | Full XDG compliance via `directories` crate. Respect XDG_CONFIG_HOME, XDG_DATA_HOME. |
| **Time Format** | Strftime strings (e.g., `%Y-%m-%d %H:%M`). Consistent across all apps. |
| **Notifications** | `notify-rust` for desktop notifications, fall back to TUI notification area. |

### Accessibility (Phase 1)

| Feature | Status |
|---------|--------|
| High contrast themes | Required |
| Full keyboard navigation | Required |
| Screen reader hints | Phase 2 |
| Sound cues | Phase 2 |

### Plugin System

| Component | Implementation |
|-----------|----------------|
| Architecture | Multi-backend: Lua (mlua) + WASM (wasmtime) + Native (.so/.dll) |
| Priority | Lua first → WASM phase 2 → Native optional |
| Interface | Unified `Plugin` trait abstracts backend differences |

### Testing Requirements

All apps must include:
- **Unit tests** for business logic
- **Snapshot tests** for UI rendering
- **Property tests** for navigation and input handling
- **Integration tests** for app lifecycle

### CI/CD

| Component | Configuration |
|-----------|---------------|
| Platforms | Linux (Ubuntu), macOS, Windows (ConPTY) |
| Rust Versions | Stable, Beta, MSRV (stable minus 2) |
| Checks | `cargo build`, `cargo test`, `cargo clippy` |
| Releases | Pre-built binaries + crates.io |

---

## Spec Interview Progress

Detailed specs are being developed through interviews. To resume, ask Claude to "continue the spec interviews".

### Completed Specs (Fully Detailed)

| Crate | Status | Key Decisions |
|-------|--------|---------------|
| **tui-widgets** | Done | Internal state management, Ratatui StatefulWidget trait, full animation system (on-demand tick), rich cell widgets with inline editing, Excel-style multi-select, callback props for events |
| **tui-theme** | Done | Multiple color palettes (24-bit/256/16), layered overrides with inheritance, state-level widget styles, hot reload, compiled Rust presets |
| **tui-keybinds** | Done | Leader key pattern (configurable), full macro recording to TOML, error on conflict, expression+context conditions, full vim/emacs presets |
| **tui-shell** | Done | Hybrid app launch (in-process/subprocess), Unix domain sockets IPC, sandboxed buffers, full session restore, multi-workspace support, binary split tiling |
| **tui-testing** | Done | Structured buffer snapshots, custom implementation, multi-runtime harnesses, checkpoint assertions, configurable timing, structured diffs |
| **tui-plugins** | Done | Multi-backend (Lua/WASM/Native), Lua via mlua primary, unified Plugin trait, sandboxed execution, phase 2 WASM support |

### Completed App Specs

All 49 apps have complete SPEC.md files. Implementation status varies by tier.

| Tier | Spec Status | Implementation | Apps |
|------|-------------|----------------|------|
| **Tier 1** (4) | Done | **Implemented** | habit-tracker, flashcard-trainer, time-tracker, cheatsheet-browser |
| **Tier 2** (10) | Done | **Implemented** | task-manager, note-manager-*, personal-wiki, config-editor, task-scheduler, service-manager, process-monitor |
| **Tier 3** (10) | Done | Pending | log-viewer, ssh-hub, backup-manager, server-dashboard-*, network-monitor, docker-manager, file-manager, diff-tool |
| **Tier 4** (9) | Done | Pending | hex-editor, csv-viewer, kanban-standalone, git-client, api-tester, ci-dashboard, k8s-dashboard, db-client, metrics-viewer |
| **Tier 5** (11) | Done | Pending | queue-monitor-*, chat-client, email-client, log-anomaly-detector, port-scanner-*, permissions-auditor-* |
| **Shell variants** (4) | Done | Pending | tui-shell-tiled, tui-shell-floating, tui-shell-tabbed, tui-shell-fullscreen |

### Key Architecture Decisions Made

#### tui-widgets
- **State**: Widgets own internal state (selection, scroll, etc.)
- **Focus**: Visual order Tab navigation, arrows/jk within widgets
- **Animation**: Full system with extended easing (bounce, elastic, cubic-bezier)
- **DataTable**: Virtual scroll, rich cells, inline edit, TSV clipboard default
- **TreeView**: Nerd Fonts, indent cap at 8 levels, breadcrumb mode toggle
- **FormBuilder**: Builder pattern API, cross-field validation, debounced async validation
- **CommandPalette**: Pure nucleo fuzzy matching, SQLite recents, multi-step wizard for params
- **Accessibility**: Screen reader hints + sound cues + text announcements (all three)

#### tui-theme
- **Colors**: Theme author provides 24-bit, 256, and 16-color palettes
- **Inheritance**: Themes can extend other themes, layered user overrides
- **Styles**: State-level granularity (widget.component.state)
- **Detection**: Auto-detect terminal Unicode/color support
- **Hot reload**: File watcher for theme development
- **Presets**: Compiled as Rust structs for performance
- **Accessibility**: Pre-made HC themes + contrast filter for user themes

#### tui-keybinds
- **Sequences**: Leader key default, user can select vim chords or emacs style
- **Macros**: Built-in recording, TOML file persistence, shareable
- **Conflicts**: Error on conflict, require explicit unbind
- **Conditions**: Simple context enum + boolean expression syntax
- **Groups**: Named action groups with preset configurations (vim/arrows navigation)
- **Presets**: Full vim, emacs, default keymaps shipped
- **Display**: Symbolic (Unicode) with text fallback, configurable

#### tui-shell
- **App Launch**: Hybrid - apps declare preference (in-process plugin or subprocess)
- **IPC**: Unix domain sockets with JSON messages
- **Input Routing**: Focused app owns input, shell captures via prefix key
- **Buffers**: Sandboxed buffer per app, shell composites
- **Sessions**: Full session restore by default, per-app/user configurable
- **Workspaces**: Multi-workspace (apps can be visible in multiple)
- **Crash Handling**: Crash dialog with restart option
- **Tiling**: Binary split (i3/sway style)
- **Floating**: Focus-follows-mouse for z-order
- **Launcher**: Both CommandPalette integration AND dedicated launcher

#### tui-testing
- **Snapshots**: Structured buffer format (cells with styles), custom implementation
- **Diffs**: Structured cell-by-cell diff with position and style changes
- **Runtimes**: Multi-runtime (TokioTestHarness, AsyncStdTestHarness, SmolTestHarness)
- **Frames**: Checkpoint assertions - explicit assert_frame() during tests
- **Timing**: Configurable - default ignores delays, opt-in for real timing
- **Fixtures**: Both deterministic AND random generators
- **Generators**: Fixed defaults with configurable builders
