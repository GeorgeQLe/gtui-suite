mod app;
mod config;
mod models;
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
    let config = Config::load()?;
    let mut app = App::new(config);

    // Load mailboxes
    app.refresh().await;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut last_check = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Auto-check for new mail
        if last_check.elapsed() >= Duration::from_secs(60) {
            app.check_new_mail().await;
            last_check = std::time::Instant::now();
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.handle_key(key).await {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
