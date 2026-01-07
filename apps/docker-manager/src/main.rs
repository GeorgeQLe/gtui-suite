#![allow(dead_code)]

mod app;
mod config;
mod container;
mod docker_client;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Try to connect to Docker/Podman
    if let Err(e) = app.connect().await {
        app.error = Some(format!("Failed to connect: {}", e));
    }

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Poll for events
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Handle quit
                if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    return Ok(());
                }

                // Handle refresh
                if key.code == KeyCode::F(5) {
                    if let Err(e) = app.refresh_all().await {
                        app.error = Some(format!("Refresh failed: {}", e));
                    }
                    continue;
                }

                // Handle other keys
                if app.handle_key(key) {
                    return Ok(());
                }
            }
        }

        // Check for log updates
        if let Some(ref mut rx) = app.log_rx {
            while let Ok(line) = rx.try_recv() {
                app.logs.push(line);
                // Auto-scroll to bottom if near bottom
                if app.logs_scroll >= app.logs.len().saturating_sub(10) {
                    app.logs_scroll = app.logs.len().saturating_sub(1);
                }
            }
        }
    }
}
