#![allow(dead_code)]

mod app;
mod config;
mod log_entry;
mod parser;
mod ui;
mod watcher;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

use app::App;

fn main() -> Result<()> {
    // Get file path from args
    let args: Vec<String> = std::env::args().collect();
    let file_path = args.get(1).map(|s| s.as_str());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(file_path)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Poll for events with timeout to allow file updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::NONE) |
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    _ => app.handle_key(key),
                }
            }
        }

        // Check for file updates
        app.check_updates()?;
    }
}
