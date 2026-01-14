use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_timers(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "COUNTDOWN TIMER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} timers", app.timers.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} active", app.active_count()),
            Style::default().fg(if app.active_count() > 0 { Color::Green } else { Color::DarkGray }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Timers "));

    frame.render_widget(header, area);
}

fn render_timers(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate how many rows each timer needs (2 rows: info + progress bar)
    let timer_height = 3;
    let available_height = area.height.saturating_sub(2) as usize;
    let visible_timers = available_height / timer_height;

    let start = if app.selected >= visible_timers {
        app.selected - visible_timers + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .timers
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_timers)
        .map(|(i, timer)| {
            let is_selected = i == app.selected;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_icon = if timer.running {
                "▶"
            } else if timer.remaining_secs == 0 {
                "✓"
            } else if timer.remaining_secs < timer.duration_secs {
                "⏸"
            } else {
                "○"
            };

            let status_color = if timer.running {
                Color::Green
            } else if timer.remaining_secs == 0 {
                Color::Cyan
            } else {
                Color::Yellow
            };

            let time_color = if timer.remaining_secs == 0 {
                Color::Cyan
            } else if timer.running {
                Color::Green
            } else {
                Color::White
            };

            let progress = timer.progress();
            let bar_width = 20;
            let filled = (progress * bar_width as f64) as usize;
            let bar: String = format!(
                "[{}{}]",
                "█".repeat(filled),
                "░".repeat(bar_width - filled)
            );

            let lines = vec![
                Line::from(vec![
                    Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
                    Span::styled(
                        format!("{:<15}", timer.name),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!(" {} ", timer.format_time()),
                        Style::default().fg(time_color).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(bar, Style::default().fg(status_color)),
                    Span::styled(
                        format!(" {:.0}%", progress * 100.0),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ];

            ListItem::new(lines).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Countdown ({}/{}) ", app.selected + 1, app.timers.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
