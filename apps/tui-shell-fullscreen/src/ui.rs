use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::{App, Mode};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calculate layout
    let chunks = if app.show_status_bar {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)])
            .split(area)
    };

    // Render current app content
    render_app_content(frame, app, chunks[0]);

    // Render status bar if visible
    if app.show_status_bar && chunks.len() > 1 {
        render_status_bar(frame, app, chunks[1]);
    }

    // Render switcher overlay
    if app.mode == Mode::Switcher {
        render_switcher(frame, app);
    }
}

fn render_app_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(current) = app.current_app() {
        // Simulated app content
        let content = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", current.title),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(format!("  App: {}", current.name)),
            Line::from(format!("  Started: {}", current.started_at.format("%H:%M:%S"))),
            Line::from(""),
            Line::from(""),
            Line::from("  This is a fullscreen placeholder for the application."),
            Line::from("  The actual app would render its content here."),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  Press Ctrl+Space to open the app switcher",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Press Ctrl+Space twice quickly to switch to last app",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Press Ctrl+1-9 for quick slots",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, area);
    } else {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  No app running",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  Press Ctrl+Space to open the app switcher"),
        ]);
        frame.render_widget(empty, area);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut parts = Vec::new();

    // App name
    if app.config.status_bar.show_app_name {
        if let Some(current) = app.current_app() {
            parts.push(format!("[{}]", current.name));
        } else {
            parts.push("[none]".to_string());
        }
    }

    // App count
    if app.config.status_bar.show_app_count {
        parts.push(format!("{} apps", app.running_apps.len()));
    }

    // Clock
    if app.config.status_bar.show_clock {
        parts.push(app.current_time());
    }

    let status_text = parts.join(" │ ");
    let status = Paragraph::new(format!(" {} ", status_text))
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status, area);
}

fn render_switcher(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = app.config.switcher.width.min(area.width - 4);
    let height = (app.switcher_results.len() + 5).min(20) as u16;

    let switcher_area = centered_rect_fixed(width, height, area);

    // Clear background
    frame.render_widget(Clear, switcher_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Switch App ");

    let inner = block.inner(switcher_area);
    frame.render_widget(block, switcher_area);

    // Split inner area
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Search input
    let search = Paragraph::new(format!("> {}_", app.switcher_query))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(search, inner_chunks[0]);

    // Results
    if app.switcher_results.is_empty() {
        let no_match = Paragraph::new(format!("No matches for '{}'", app.switcher_query))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(no_match, inner_chunks[2]);
    } else {
        let items: Vec<ListItem> = app
            .switcher_results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == app.switcher_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let indicator = if result.is_running {
                    "[running]"
                } else {
                    "[launch]"
                };

                let text = format!("  {:20} {}", result.title, indicator);
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner_chunks[2]);
    }

    // Help line
    let help = Paragraph::new("↑↓ navigate  Enter select  x close  Esc quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, inner_chunks[3]);
}

fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
