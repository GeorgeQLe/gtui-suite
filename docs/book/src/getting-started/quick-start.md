# Quick Start

This guide walks you through running your first TUI Suite app and understanding the basics.

## Run Your First App

```bash
# Navigate to the project directory
cd gtui-suite

# Run the habit tracker app
cargo run -p habit-tracker
```

## Basic Navigation

All TUI Suite apps share common keybindings:

| Key | Action |
|-----|--------|
| `q` or `Ctrl+Q` | Quit the application |
| `?` | Show help |
| `Ctrl+P` | Open command palette |
| `j` / `k` or `Arrow keys` | Navigate up/down |
| `Enter` | Select/confirm |
| `Esc` | Cancel/back |
| `Tab` | Next field/widget |

## Available Apps

### Tier 1 - Simple Apps

```bash
cargo run -p habit-tracker      # Track daily habits
cargo run -p flashcard-trainer  # Spaced repetition flashcards
cargo run -p time-tracker       # Track time on projects
cargo run -p cheatsheet-browser # Browse command cheatsheets
```

### Tier 2 - Moderate Apps

```bash
cargo run -p task-manager           # GTD-style task management
cargo run -p personal-wiki          # Wiki-style notes
cargo run -p note-manager-daily     # Daily journal entries
cargo run -p note-manager-folder    # Folder-based notes
cargo run -p config-editor          # Edit TOML/JSON/YAML configs
```

## Configuration

Apps store configuration in `~/.config/<app-name>/`:

```
~/.config/habit-tracker/
├── config.toml      # App settings
├── keybinds.toml    # Custom keybindings
└── data.db          # SQLite database
```

## Next Steps

- [Running Apps](./running-apps.md) - Detailed app usage
- [Building an App](../guides/building-an-app.md) - Create your own app
- [Theming](../guides/theming.md) - Customize appearance
