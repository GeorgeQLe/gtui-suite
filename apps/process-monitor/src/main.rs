#![allow(dead_code)]
//! Process Monitor - Linux process monitor TUI with cgroup/namespace awareness.

mod app;
mod config;
mod process;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = Duration::from_millis(app.config.refresh_ms());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.can_quit() && key.code == crossterm::event::KeyCode::Char('q') {
                    return Ok(());
                }
                app.handle_key(key);
            }
        }

        if !app.paused {
            app.refresh_processes();
        }
    }
}
