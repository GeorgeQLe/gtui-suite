#![allow(dead_code)]
//! Config Editor - Configuration file editor TUI.
//!
//! Features:
//! - Support for TOML, JSON, YAML, INI
//! - Syntax highlighting
//! - Schema validation
//! - Tree view navigation

mod app;
mod config;
mod formats;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, time::Duration};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = args.get(1).cloned();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(file_path)?;
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

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.can_quit() && key.code == crossterm::event::KeyCode::Char('q') {
                    if app.modified {
                        app.show_quit_confirm = true;
                    } else {
                        return Ok(());
                    }
                } else if app.show_quit_confirm {
                    match key.code {
                        crossterm::event::KeyCode::Char('y') => return Ok(()),
                        crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Esc => {
                            app.show_quit_confirm = false;
                        }
                        _ => {}
                    }
                } else {
                    app.handle_key(key);
                }
            }
        }
    }
}
