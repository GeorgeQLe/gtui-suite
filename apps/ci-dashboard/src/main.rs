mod app;
mod config;
mod models;
mod providers;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

use app::App;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load().unwrap_or_default();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(config);
    app.refresh().await;

    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_refresh = std::time::Instant::now();
    let refresh_interval = Duration::from_secs(app.config.display.refresh_secs);

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.handle_key(key).await {
                        break;
                    }
                }
            }
        }

        // Auto-refresh
        if last_refresh.elapsed() >= refresh_interval {
            app.refresh().await;
            last_refresh = std::time::Instant::now();
        }

        // Clear status message after a while
        if app.status_message.is_some() {
            app.status_message = None;
        }
    }

    Ok(())
}
