use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
};

use crate::app::{App, ConfirmAction, Mode, ServerFormState, View};
use crate::server::ServerStatus;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_content(frame, app, chunks[0]);
    render_status_bar(frame, app, chunks[1]);

    match &app.mode {
        Mode::AddServer(form) => render_server_form(frame, "Add Server", form),
        Mode::EditServer(form) => render_server_form(frame, "Edit Server", form),
        Mode::Confirm(action) => render_confirm_dialog(frame, action),
        Mode::FilterTag(tag) => render_filter_dialog(frame, tag),
        Mode::Normal => {}
    }
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Dashboard => render_dashboard(frame, app, area),
        View::ServerDetail => render_server_detail(frame, app, area),
        View::Help => render_help(frame, area),
    }
}

fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Server", "CPU", "Mem", "Disk", "Load", "Status"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let filtered = app.filtered_servers();
    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, server)| {
        let metrics = app.get_metrics(&server.id);

        let style = if i == app.selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else if !server.enabled {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let (cpu, mem, disk, load, status) = if let Some(m) = metrics {
            let _status_style = match m.status {
                ServerStatus::Ok => Style::default().fg(Color::Green),
                ServerStatus::Warning => Style::default().fg(Color::Yellow),
                ServerStatus::Critical => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::DarkGray),
            };

            (
                format!("{:.0}%", m.cpu_percent),
                m.format_memory(),
                format!("{:.0}%", m.disk_percent()),
                format!("{:.2}", m.load_1),
                format!("{} {}", m.status.icon(), m.status.label()),
            )
        } else {
            ("--".to_string(), "--".to_string(), "--".to_string(), "--".to_string(), "? UNKNOWN".to_string())
        };

        Row::new(vec![
            server.name.clone(),
            cpu,
            mem,
            disk,
            load,
            status,
        ]).style(style)
    }).collect();

    let title = if let Some(tag) = &app.tag_filter {
        format!(" Server Dashboard [tag: {}] ", tag)
    } else {
        " Server Dashboard ".to_string()
    };

    let table = Table::new(rows, [
        Constraint::Percentage(25),
        Constraint::Percentage(12),
        Constraint::Percentage(12),
        Constraint::Percentage(12),
        Constraint::Percentage(12),
        Constraint::Percentage(25),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_server_detail(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(server) = app.selected_server() {
        let metrics = app.get_metrics(&server.id);

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&server.name),
            ]),
            Line::from(vec![
                Span::styled("Host: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(server.connection_string()),
            ]),
            Line::from(vec![
                Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(server.tags_display()),
            ]),
            Line::from(""),
        ];

        if let Some(m) = metrics {
            lines.push(Line::from(vec![
                Span::styled("CPU: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.1}%", m.cpu_percent)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Memory: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} / {:.1}G ({:.1}%)",
                    m.format_memory(),
                    m.memory_total as f64 / 1_073_741_824.0,
                    m.memory_percent()
                )),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Disk: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.1}%", m.disk_percent())),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Load: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.2} {:.2} {:.2}", m.load_1, m.load_5, m.load_15)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} {}", m.status.icon(), m.status.label())),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(m.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]));
        } else {
            lines.push(Line::from("No metrics available"));
        }

        let detail = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(format!(" {} ", server.name)));

        frame.render_widget(detail, area);
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Server Dashboard Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, ↑/↓     Move up/down"),
        Line::from("  g/G          Top/bottom"),
        Line::from("  Enter        View details"),
        Line::from("  Esc          Back"),
        Line::from(""),
        Line::from(Span::styled("Server Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  a            Add server"),
        Line::from("  e            Edit server"),
        Line::from("  d            Delete server"),
        Line::from(""),
        Line::from(Span::styled("Display", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  g            Toggle graphs"),
        Line::from("  t            Filter by tag"),
        Line::from("  Space        Pause updates"),
        Line::from("  r/F5         Refresh"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ?            Show help"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));

    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let paused = if app.paused { " [PAUSED]" } else { "" };
    let overall = app.overall_status();

    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | a Add | r Refresh | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(
        " {} {} | {} servers{} | {} ",
        overall.icon(),
        overall.label(),
        app.servers.len(),
        paused,
        message
    )).style(style);

    frame.render_widget(status, area);
}

fn render_server_form(frame: &mut Frame, title: &str, form: &ServerFormState) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            (0..ServerFormState::field_count())
                .map(|_| Constraint::Length(2))
                .collect::<Vec<_>>()
        )
        .split(inner);

    for i in 0..ServerFormState::field_count() {
        let label = ServerFormState::field_label(i);
        let value = form.field_value(i);
        let is_active = i == form.field;

        let style = if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let display = if is_active {
            format!("{}: {}█", label, value)
        } else {
            format!("{}: {}", label, value)
        };

        frame.render_widget(Paragraph::new(display).style(style), chunks[i]);
    }
}

fn render_confirm_dialog(frame: &mut Frame, action: &ConfirmAction) {
    let area = centered_rect(50, 25, frame.area());
    frame.render_widget(Clear, area);

    let message = match action {
        ConfirmAction::DeleteServer(id) => format!("Delete server {}?", &id[..8.min(id.len())]),
    };

    let text = vec![
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from(Span::styled("(y)es / (n)o", Style::default().fg(Color::Yellow))),
    ];

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_filter_dialog(frame: &mut Frame, tag: &str) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Filter by Tag ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input = Paragraph::new(format!("{}█", tag));
    frame.render_widget(input, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
