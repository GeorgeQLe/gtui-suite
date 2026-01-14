mod app;
mod ui;

use anyhow::Result;
use crossterm::{event::{self, Event, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::prelude::*;
use std::{io, time::Duration};
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app).await;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(k) = event::read()? { if k.kind == KeyEventKind::Press && app.handle_key(k) { break; } }
        }
        app.tick();
    }
    Ok(())
}
