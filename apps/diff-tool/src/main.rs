#![allow(dead_code)]

mod app;
mod diff;
mod ui;

use std::env;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: diff-tool <file1> <file2>");
        eprintln!("  Compare two files side-by-side");
        std::process::exit(1);
    }

    let left = PathBuf::from(&args[1]);
    let right = PathBuf::from(&args[2]);

    if !left.exists() {
        eprintln!("Error: {} does not exist", left.display());
        std::process::exit(1);
    }
    if !right.exists() {
        eprintln!("Error: {} does not exist", right.display());
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    if let Err(e) = app.load_files(left, right) {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        eprintln!("Failed to load files: {}", e);
        return Err(e);
    }

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    return Ok(());
                }

                if app.handle_key(key) {
                    return Ok(());
                }
            }
        }
    }
}
