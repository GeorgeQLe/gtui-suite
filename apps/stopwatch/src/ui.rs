use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{format_ms, App};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_display(frame, app, chunks[1]);
    render_stats(frame, app, chunks[2]);
    render_laps(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let status = if app.running {
        ("RUNNING", Color::Green)
    } else if app.elapsed_ms > 0 {
        ("PAUSED", Color::Yellow)
    } else {
        ("READY", Color::Gray)
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "STOPWATCH",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(status.0, Style::default().fg(status.1)),
        Span::raw(" | "),
        Span::styled(
            format!("{} laps", app.laps.len()),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Timer "));

    frame.render_widget(header, area);
}

fn render_display(frame: &mut Frame, app: &App, area: Rect) {
    let time_str = app.format_time();
    let color = if app.running { Color::Green } else { Color::White };

    let display = Paragraph::new(Line::from(vec![
        Span::styled(
            time_str,
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Elapsed Time ")
            .border_style(Style::default().fg(if app.running { Color::Green } else { Color::White })),
    );

    frame.render_widget(display, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let best = app.best_lap().map(|l| format_ms(l.split_ms)).unwrap_or("--.--".to_string());
    let worst = app.worst_lap().map(|l| format_ms(l.split_ms)).unwrap_or("--.--".to_string());
    let avg = app.average_lap().map(|ms| format_ms(ms)).unwrap_or("--.--".to_string());

    let stats = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Best: ", Style::default().fg(Color::Gray)),
            Span::styled(best, Style::default().fg(Color::Green)),
            Span::raw("   "),
            Span::styled("Worst: ", Style::default().fg(Color::Gray)),
            Span::styled(worst, Style::default().fg(Color::Red)),
            Span::raw("   "),
            Span::styled("Avg: ", Style::default().fg(Color::Gray)),
            Span::styled(avg, Style::default().fg(Color::Yellow)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Lap Statistics "));

    frame.render_widget(stats, area);
}

fn render_laps(frame: &mut Frame, app: &App, area: Rect) {
    if app.laps.is_empty() {
        let placeholder = Paragraph::new("Press 'l' to record laps")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Laps "));
        frame.render_widget(placeholder, area);
        return;
    }

    let best_split = app.best_lap().map(|l| l.split_ms);
    let worst_split = app.worst_lap().map(|l| l.split_ms);

    let visible_height = area.height.saturating_sub(2) as usize;
    let start = if app.selected_lap >= visible_height {
        app.selected_lap - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .laps
        .iter()
        .enumerate()
        .rev() // Show newest first
        .skip(start)
        .take(visible_height)
        .map(|(i, lap)| {
            let is_selected = i == app.selected_lap;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let split_color = if Some(lap.split_ms) == best_split {
                Color::Green
            } else if Some(lap.split_ms) == worst_split {
                Color::Red
            } else {
                Color::White
            };

            let marker = if Some(lap.split_ms) == best_split {
                "★"
            } else if Some(lap.split_ms) == worst_split {
                "▼"
            } else {
                " "
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", marker),
                    Style::default().fg(split_color),
                ),
                Span::styled(
                    format!("Lap {:>2} ", lap.number),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{} ", format_ms(lap.elapsed_ms)),
                    Style::default().fg(Color::White),
                ),
                Span::styled("(", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("+{}", format_ms(lap.split_ms)),
                    Style::default().fg(split_color),
                ),
                Span::styled(")", Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Laps ({}) ", app.laps.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
