//! UI rendering for service manager.

use crate::app::{App, Filter};
use crate::services::ServiceStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(3)])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_services(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);

    if app.show_help { draw_help(f); }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] All", "[2] Running", "[3] Stopped", "[4] Failed"];
    let selected = match app.filter {
        Filter::All => 0, Filter::Running => 1, Filter::Stopped => 2, Filter::Failed => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Services "))
        .select(selected)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn draw_services(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_services();
    let items: Vec<ListItem> = filtered.iter().enumerate().map(|(i, svc)| {
        let status_color = match svc.status {
            ServiceStatus::Running => Color::Green,
            ServiceStatus::Stopped => Color::Gray,
            ServiceStatus::Failed => Color::Red,
            ServiceStatus::Unknown => Color::Yellow,
        };

        let mut style = Style::default();
        if i == app.selected_index { style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD); }

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(svc.status.symbol(), Style::default().fg(status_color)),
                Span::raw(" "),
                Span::styled(&svc.name, style),
            ]),
            Line::from(Span::styled(
                format!("  {} | {}", svc.status.label(), svc.description),
                Style::default().fg(Color::DarkGray),
            )),
        ])
    }).collect();

    let title = if app.searching {
        format!(" Search: {} ", app.search)
    } else {
        format!(" {} services ", filtered.len())
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(list, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | s start | S stop | r restart | / search".into()
    });
    let status = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
    f.render_widget(status, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(50, 50, f.area());
    f.render_widget(Clear, area);

    let help = Paragraph::new(vec![
        Line::from(Span::styled("Controls", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k   Navigate"),
        Line::from("  1-4   Filter by status"),
        Line::from("  /     Search"),
        Line::from("  s     Start service"),
        Line::from("  S     Stop service"),
        Line::from("  r     Restart service"),
        Line::from("  R     Refresh list"),
        Line::from("  q     Quit"),
    ]).block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help, area);
}

fn centered_rect(px: u16, py: u16, area: Rect) -> Rect {
    let v = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - py) / 2), Constraint::Percentage(py), Constraint::Percentage((100 - py) / 2)])
        .split(area);
    Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - px) / 2), Constraint::Percentage(px), Constraint::Percentage((100 - px) / 2)])
        .split(v[1])[1]
}
