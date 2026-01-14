use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, FdType};

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

    if app.show_details {
        render_fd_table(frame, app, chunks[1]);
    } else {
        render_process_table(frame, app, chunks[1]);
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "FD INSPECTOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} processes", app.processes.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} total FDs", app.total_fd_count()),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} sockets", app.socket_count()),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" File Descriptor Inspector "));

    frame.render_widget(header, area);
}

fn render_process_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["PID", "Process", "User", "FD Count"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected_process >= visible_height {
        app.selected_process - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .processes
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, proc)| {
            let style = if i == app.selected_process {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let fd_color = if proc.fd_count > 100 {
                Color::Red
            } else if proc.fd_count > 50 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Cell::from(proc.pid.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(proc.name.clone()),
                Cell::from(proc.user.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(proc.fd_count.to_string()).style(Style::default().fg(fd_color)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Processes ({}/{}) - Press Enter to view FDs ",
                    app.selected_process + 1, app.processes.len())),
        );

    frame.render_widget(table, area);
}

fn render_fd_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["FD", "Type", "Path", "Mode", "Size", "Flags"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let filtered = app.filtered_fds();
    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected_fd >= visible_height {
        app.selected_fd - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, fd)| {
            let style = if i == app.selected_fd {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_color = match fd.fd_type {
                FdType::Regular => Color::White,
                FdType::Socket => Color::Cyan,
                FdType::Pipe => Color::Yellow,
                FdType::Directory => Color::Blue,
                FdType::Device => Color::Magenta,
                FdType::Unknown => Color::DarkGray,
            };

            let size_str = fd.size
                .map(|s| format_size(s))
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(fd.fd.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(fd.fd_type.name()).style(Style::default().fg(type_color)),
                Cell::from(fd.path.clone()),
                Cell::from(fd.mode.clone()).style(Style::default().fg(Color::Green)),
                Cell::from(size_str).style(Style::default().fg(Color::DarkGray)),
                Cell::from(fd.flags.clone()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Percentage(40),
        Constraint::Length(5),
        Constraint::Length(10),
        Constraint::Percentage(25),
    ];

    let proc_name = app.current_process()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" FDs for {} ({}/{}) - Backspace to go back ",
                    proc_name, app.selected_fd + 1, filtered.len())),
        );

    frame.render_widget(table, area);
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1}G", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}K", bytes as f64 / 1_000.0)
    } else {
        format!("{}B", bytes)
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
