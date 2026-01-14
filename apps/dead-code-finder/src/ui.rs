use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, DeadCodeType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_table(frame, app, chunks[1]);
    render_details(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let high_conf = app.high_confidence_count();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "DEAD CODE FINDER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} issues", app.total_items()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} high confidence", high_conf),
            Style::default().fg(Color::Red),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Code Analysis "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Type", "Name", "File", "Line", "Conf"]
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
            let item = app.items.get(idx)?;
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_color = match item.code_type {
                DeadCodeType::UnusedFunction => Color::Magenta,
                DeadCodeType::UnusedVariable => Color::Blue,
                DeadCodeType::UnusedImport => Color::Yellow,
                DeadCodeType::UnusedStruct => Color::Cyan,
                DeadCodeType::UnusedEnum => Color::Green,
                DeadCodeType::UnusedConst => Color::Red,
            };

            let conf_color = if item.confidence >= 90 {
                Color::Green
            } else if item.confidence >= 70 {
                Color::Yellow
            } else {
                Color::Red
            };

            Some(Row::new(vec![
                Cell::from(item.code_type.icon()).style(Style::default().fg(type_color)),
                Cell::from(item.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(item.file.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(item.line.to_string()),
                Cell::from(format!("{}%", item.confidence)).style(Style::default().fg(conf_color)),
            ]).style(style))
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Percentage(30),
        Constraint::Percentage(35),
        Constraint::Length(6),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Dead Code ({}/{}) ", app.selected + 1, app.filtered_indices.len())),
        );

    frame.render_widget(table, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(&idx) = app.filtered_indices.get(app.selected) {
        let item = &app.items[idx];
        Line::from(vec![
            Span::styled("Suggestion: ", Style::default().fg(Color::Gray)),
            Span::styled(&item.suggestion, Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(Span::styled("No items", Style::default().fg(Color::DarkGray)))
    };

    let details = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Details "));

    frame.render_widget(details, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
