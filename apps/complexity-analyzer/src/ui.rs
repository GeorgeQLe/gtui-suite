use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, ComplexityLevel};

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
    render_table(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let high = app.high_complexity_count();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "COMPLEXITY ANALYZER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} functions", app.functions.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} high complexity", high),
            Style::default().fg(if high > 0 { Color::Red } else { Color::Green }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Code Metrics "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Function", "File", "Cyclomatic", "Cognitive", "Lines", "Level"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .filtered_indices
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .filter_map(|(i, &idx)| {
            let f = app.functions.get(idx)?;
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let level = f.complexity_level();
            let level_color = match level {
                ComplexityLevel::High => Color::Red,
                ComplexityLevel::Medium => Color::Yellow,
                ComplexityLevel::Low => Color::Green,
            };

            Some(Row::new(vec![
                Cell::from(f.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("{}:{}", f.file, f.line)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(f.cyclomatic.to_string()),
                Cell::from(f.cognitive.to_string()),
                Cell::from(f.lines.to_string()),
                Cell::from(level.name()).style(Style::default().fg(level_color)),
            ]).style(style))
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(12),
        Constraint::Percentage(12),
        Constraint::Percentage(10),
        Constraint::Percentage(16),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Functions ({}/{}) ", app.selected + 1, app.filtered_indices.len())),
        );

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
