use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, LicenseCategory};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_summary(frame, app, chunks[1]);
    render_table(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let filter_text = match app.filter {
        Some(cat) => format!(" | Filter: {}", cat.name()),
        None => String::new(),
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "LICENSE CHECKER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} packages", app.licenses.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(filter_text, Style::default().fg(Color::Magenta)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" License Audit "));

    frame.render_widget(header, area);
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let (permissive, copyleft, proprietary, unknown) = app.category_counts();

    let summary = Paragraph::new(Line::from(vec![
        Span::styled("Permissive(1): ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", permissive), Style::default().fg(Color::Green)),
        Span::raw("  "),
        Span::styled("Copyleft(2): ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", copyleft), Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled("Proprietary(3): ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", proprietary), Style::default().fg(Color::Red)),
        Span::raw("  "),
        Span::styled("Unknown(4): ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", unknown), Style::default().fg(Color::Magenta)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Summary "));

    frame.render_widget(summary, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Package", "Version", "License", "Category"]
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
            let lic = app.licenses.get(idx)?;
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let cat_color = match lic.category {
                LicenseCategory::Permissive => Color::Green,
                LicenseCategory::Copyleft => Color::Yellow,
                LicenseCategory::Proprietary => Color::Red,
                LicenseCategory::Unknown => Color::Magenta,
            };

            Some(Row::new(vec![
                Cell::from(lic.package.clone()),
                Cell::from(lic.version.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(lic.license.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(lic.category.name()).style(Style::default().fg(cat_color)),
            ]).style(style))
        })
        .collect();

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(35),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Licenses ({}/{}) ", app.selected + 1, app.filtered_indices.len())),
        );

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
