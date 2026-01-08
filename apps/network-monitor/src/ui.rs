use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, SortBy, View};
use crate::connections::{ConnectionState, Protocol};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    match app.view {
        View::Connections => render_connections(frame, app, chunks[0]),
        View::Help => render_help(frame, chunks[0]),
    }

    render_status_bar(frame, app, chunks[1]);
}

fn render_connections(frame: &mut Frame, app: &App, area: Rect) {
    let filter_info = format!(
        " {} | {} ",
        match app.filter_protocol {
            None => "All".to_string(),
            Some(Protocol::Tcp) => "TCP".to_string(),
            Some(Protocol::Udp) => "UDP".to_string(),
        },
        if app.show_listening { "All states" } else { "No LISTEN" }
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Network Connections ")
        .title_bottom(Line::from(filter_info).right_aligned());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.connections.is_empty() {
        let empty = Paragraph::new("No connections found. Try running with sudo for full info.");
        frame.render_widget(empty, inner);
        return;
    }

    // Header
    let header_cells = ["Proto", "Local Address", "Remote Address", "State", "PID", "Process"]
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let is_sorted = matches!(
                (i, app.sort_by),
                (0, SortBy::Protocol) |
                (1, SortBy::LocalAddr) |
                (2, SortBy::RemoteAddr) |
                (3, SortBy::State) |
                (4, SortBy::Pid)
            );
            let label = if is_sorted {
                format!("{} {}", h, if app.sort_ascending { "▲" } else { "▼" })
            } else {
                h.to_string()
            };
            Cell::from(label).style(Style::default().add_modifier(Modifier::BOLD))
        });

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // Calculate visible rows
    let visible_height = inner.height.saturating_sub(3) as usize;
    let scroll = if app.selected >= app.scroll + visible_height {
        app.selected - visible_height + 1
    } else if app.selected < app.scroll {
        app.selected
    } else {
        app.scroll
    };

    let rows: Vec<Row> = app.connections
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, conn)| {
            let is_selected = i == app.selected;

            let state_style = match conn.state {
                ConnectionState::Established => Style::default().fg(Color::Green),
                ConnectionState::Listen => Style::default().fg(Color::Yellow),
                ConnectionState::TimeWait | ConnectionState::CloseWait => Style::default().fg(Color::Red),
                _ => Style::default(),
            };

            let cells = vec![
                Cell::from(conn.protocol.as_str()),
                Cell::from(conn.local_addr.clone()),
                Cell::from(conn.remote_addr.clone()),
                Cell::from(conn.state.as_str()).style(state_style),
                Cell::from(conn.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string())),
                Cell::from(conn.process_name.clone().unwrap_or_else(|| "-".to_string())),
            ];

            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(cells).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Network Monitor Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, arrows    Move up/down"),
        Line::from("  PgUp/PgDn      Page up/down"),
        Line::from("  g/G            First/last"),
        Line::from(""),
        Line::from(Span::styled("Filters", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  t              Cycle protocol (All/TCP/UDP)"),
        Line::from("  e              Cycle state filter"),
        Line::from("  l              Toggle LISTEN connections"),
        Line::from(""),
        Line::from(Span::styled("Sorting", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  s              Cycle sort column"),
        Line::from("  S              Toggle sort direction"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  r, F5          Refresh"),
        Line::from("  a              Toggle auto-refresh"),
        Line::from("  ?              Show help"),
        Line::from("  q              Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let count = format!("{} connections", app.filtered_count());
    let auto = if app.auto_refresh { "Auto-refresh: ON" } else { "Auto-refresh: OFF" };

    let message = app.message.as_deref()
        .unwrap_or("? Help | t Protocol | e State | s Sort | r Refresh | q Quit");

    let status = Paragraph::new(format!(" {} | {} | {} ", count, auto, message))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
