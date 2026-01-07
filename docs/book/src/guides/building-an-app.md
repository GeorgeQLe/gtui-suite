# Building an App

This guide walks you through creating a new TUI Suite application using the shared crates.

## App Structure

Each app follows a consistent structure:

```
apps/my-app/
├── Cargo.toml      # Package manifest using workspace dependencies
├── SPEC.md         # Feature specification
└── src/
    ├── main.rs     # Entry point
    ├── app.rs      # Application state and logic
    ├── ui.rs       # UI rendering
    ├── config.rs   # Configuration management
    ├── db.rs       # Database operations (if needed)
    └── models.rs   # Data structures
```

## Step 1: Create the Directory

```bash
mkdir -p apps/my-app/src
```

## Step 2: Create Cargo.toml

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Shared crates
tui-widgets = { workspace = true }
tui-theme = { workspace = true }
tui-keybinds = { workspace = true }

# TUI framework
ratatui = { workspace = true }
crossterm = { workspace = true }

# Async runtime
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
toml = { workspace = true }

# Database (if needed)
rusqlite = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }
```

## Step 3: Create main.rs

```rust
#![allow(dead_code)]

mod app;
mod config;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use app::App;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Main loop
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
            app.handle_key(key);
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
```

## Step 4: Create app.rs

```rust
use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::config::Config;

pub struct App {
    pub config: Config,
    pub running: bool,
    // Add your app state here
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;

        Ok(Self {
            config,
            running: true,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Handle keyboard input
    }
}
```

## Step 5: Create ui.rs

```rust
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let block = Block::default()
        .title("My App")
        .borders(Borders::ALL);

    let content = Paragraph::new("Hello, TUI Suite!")
        .block(block);

    frame.render_widget(content, area);
}
```

## Step 6: Create config.rs

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    fn config_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "my-app")
            .map(|p| p.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
```

## Step 7: Add to Workspace

Edit the root `Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing members
    "apps/my-app",
]
```

## Step 8: Run Your App

```bash
cargo run -p my-app
```

## Using Shared Crates

### DataTable

```rust
use tui_widgets::{DataTable, Column, TableState};

let columns = vec![
    Column::new("Name", |item: &MyItem| item.name.clone().into()),
    Column::new("Value", |item: &MyItem| item.value.into()),
];

let table = DataTable::new(columns, items);
let mut state = TableState::default();

frame.render_stateful_widget(table, area, &mut state);
```

### Theming

```rust
use tui_theme::{Theme, ThemeManager};

let manager = ThemeManager::default();
let theme = manager.current();
let style = theme.style("widget.selected");
```

### Keybindings

```rust
use tui_keybinds::{KeybindManager, KeymapPreset};

let mut manager = KeybindManager::new();
manager.load_preset(KeymapPreset::Default);
```

## Best Practices

1. **Use shared crates** - Don't reinvent widgets, theming, or keybindings
2. **Write SPEC.md first** - Document features before implementing
3. **Include tests** - Snapshot tests for rendering, property tests for logic
4. **Support theming** - Use semantic colors from tui-theme
5. **Configurable keybinds** - Use tui-keybinds for all keyboard shortcuts
6. **Accessibility first** - Ensure keyboard navigation and high contrast support
