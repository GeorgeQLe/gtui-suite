use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};

use crate::app::{App, Category};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_categories(frame, app, chunks[1]);
    render_input(frame, app, chunks[2]);
    render_results(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "UNIT CONVERTER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            app.category.name(),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Converter "));

    frame.render_widget(header, area);
}

fn render_categories(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = Category::all().iter().map(|c| c.name()).collect();
    let selected = Category::all().iter().position(|c| *c == app.category).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" Category (Tab) "));

    frame.render_widget(tabs, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let display = if app.input.is_empty() {
        "0".to_string()
    } else {
        app.input.clone()
    };

    let input = Paragraph::new(Line::from(vec![
        Span::styled(
            display,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            app.current_unit(),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(" (u to change unit)", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Input ")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(input, area);
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
    if app.results.is_empty() {
        let placeholder = Paragraph::new("Enter a value to convert")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(placeholder, area);
        return;
    }

    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, (short, long, value))| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let formatted = if value.abs() >= 1000000.0 || (value.abs() < 0.001 && *value != 0.0) {
                format!("{:.6e}", value)
            } else {
                format!("{:.6}", value)
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>6} ", short),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    formatted,
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("({})", long),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Conversions "),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
