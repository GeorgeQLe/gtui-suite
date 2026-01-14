use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{AclType, App};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_file_info(frame, app, chunks[1]);
    render_acl_table(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mod_indicator = if app.modified {
        Span::styled(" [MODIFIED]", Style::default().fg(Color::Red))
    } else {
        Span::raw("")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "ACL EDITOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        mod_indicator,
        Span::raw(" | "),
        Span::styled(
            format!("{} entries", app.entries.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            if app.has_extended_acl() { "Extended ACL" } else { "Basic Permissions" },
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Access Control List Editor "));

    frame.render_widget(header, area);
}

fn render_file_info(frame: &mut Frame, app: &App, area: Rect) {
    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("File: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.file.path, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Owner: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.file.owner, Style::default().fg(Color::Yellow)),
            Span::styled("  Group: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.file.group, Style::default().fg(Color::Yellow)),
            Span::styled("  Mode: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.file.mode, Style::default().fg(Color::Green)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" File "));

    frame.render_widget(info, area);
}

fn render_acl_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Type", "Qualifier", "Read", "Write", "Execute", "ACL String"]
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
        .entries
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, entry)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_color = match entry.acl_type {
                AclType::User => Color::Cyan,
                AclType::Group => Color::Yellow,
                AclType::Mask => Color::Magenta,
                AclType::Other => Color::Green,
                AclType::Default => Color::Blue,
            };

            let perm_style = |enabled: bool| {
                if enabled {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                }
            };

            Row::new(vec![
                Cell::from(entry.acl_type.name()).style(Style::default().fg(type_color)),
                Cell::from(entry.qualifier.clone().unwrap_or_else(|| "(owner)".to_string())),
                Cell::from(if entry.read { "r" } else { "-" }).style(perm_style(entry.read)),
                Cell::from(if entry.write { "w" } else { "-" }).style(perm_style(entry.write)),
                Cell::from(if entry.execute { "x" } else { "-" }).style(perm_style(entry.execute)),
                Cell::from(entry.full_string()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Percentage(20),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Percentage(35),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" ACL Entries ({}/{}) ", app.selected + 1, app.entries.len())),
        );

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
