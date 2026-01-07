# TUI Suite

A comprehensive Rust monorepo containing reusable TUI (Terminal User Interface) components and 49 complete application examples.

## Overview

TUI Suite provides:

- **6 Shared Crates** - Reusable foundation libraries for building TUI applications
- **49 Applications** - Complete apps organized by complexity tier
- **Consistent UX** - Shared widgets, theming, and keybindings across all apps
- **Production Quality** - Comprehensive testing, accessibility support, and documentation

## Shared Crates

| Crate | Description |
|-------|-------------|
| `tui-widgets` | Reusable UI components (DataTable, TreeView, FormBuilder, CommandPalette) |
| `tui-theme` | Theming engine with presets and accessibility themes |
| `tui-keybinds` | Keybinding configuration and management |
| `tui-shell` | Multi-app orchestration shell |
| `tui-testing` | Testing utilities (snapshot, input simulation, property-based) |
| `tui-plugins` | Multi-backend plugin system (Lua, WASM, Native) |

## Application Tiers

Apps are organized by complexity:

| Tier | Complexity | Example Apps |
|------|------------|--------------|
| 1 | Simple | habit-tracker, flashcard-trainer, time-tracker |
| 2 | Moderate | task-manager, note-manager, personal-wiki |
| 3 | Complex | docker-manager, file-manager, log-viewer |
| 4 | Advanced | git-client, api-tester, k8s-dashboard |
| 5 | Expert | chat-client, email-client, log-anomaly-detector |

## Core Technologies

- **TUI Framework**: ratatui 0.29
- **Terminal Backend**: crossterm 0.28
- **Async Runtime**: tokio
- **Database**: rusqlite (bundled)
- **Serialization**: serde, toml, serde_json

## Quick Start

```bash
# Clone the repository
git clone https://github.com/GeorgeQLe/gtui-suite.git
cd gtui-suite

# Build all apps
cargo build --workspace

# Run an app
cargo run -p habit-tracker
```

## License

MIT License
