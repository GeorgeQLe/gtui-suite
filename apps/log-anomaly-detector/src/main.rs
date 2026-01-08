mod app;
mod config;
mod detector;
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

fn main() -> Result<()> {
    let config = Config::load()?;
    let mut app = App::new(config);

    // Initial scan
    app.scan_logs();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let mut last_scan = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Auto-scan
        if app.auto_scan && last_scan.elapsed() >= Duration::from_secs(app.config.baseline.scan_interval_secs) {
            app.scan_logs();
            last_scan = std::time::Instant::now();
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.handle_key(key) {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
