use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, InputMode, View};

pub fn render(frame: &mut Frame, app: &App) {
    match app.view {
        View::Disclaimer => render_disclaimer(frame, app),
        View::Hosts => render_hosts(frame, app),
        View::HostDetail => render_host_detail(frame, app),
        View::Help => render_help(frame),
    }

    // Render input dialogs
    match app.input_mode {
        InputMode::TargetInput => render_input_dialog(frame, "Target IP", &app.input_buffer),
        InputMode::PortInput => render_input_dialog(frame, "Port Range (e.g., common, 1-1024, 22,80,443)", &app.input_buffer),
        InputMode::Normal => {}
    }
}

fn render_disclaimer(frame: &mut Frame, _app: &App) {
    let area = frame.area();

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("AUTHORIZATION DISCLAIMER", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))),
        Line::from(""),
        Line::from("This tool performs network port scanning operations."),
        Line::from(""),
        Line::from("Before using this tool, ensure you have:"),
        Line::from(""),
        Line::from("  1. Written authorization to scan the target network"),
        Line::from("  2. Permission from network owners/administrators"),
        Line::from("  3. Understanding of your organization's security policies"),
        Line::from(""),
        Line::from(Span::styled("Unauthorized network scanning may be illegal.", Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from("Only scan networks and systems you own or have explicit"),
        Line::from("permission to test."),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("Do you accept these terms and have authorization? (Y/N)", Style::default().fg(Color::Cyan))),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Port Scanner - Authorization Required "))
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_hosts(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Header
    let header_text = format!(
        " Target: {} | Ports: {} | {} ",
        app.target_ip,
        app.port_range,
        if app.scanning {
            format!("Scanning... {}/{}", app.scan_progress.0, app.scan_progress.1)
        } else {
            "Ready".to_string()
        }
    );

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" Port Scanner "));
    frame.render_widget(header, chunks[0]);

    // Hosts list
    if app.hosts.is_empty() {
        let empty = Paragraph::new("No hosts found. Press 's' to start a scan.")
            .block(Block::default().borders(Borders::ALL).title(" Hosts "));
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app.hosts
            .iter()
            .enumerate()
            .map(|(i, host)| {
                let open_ports = host.ports.len();
                let style = if i == app.selected_host {
                    Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let text = format!(
                    "{} - {} open port{}",
                    host.ip,
                    open_ports,
                    if open_ports == 1 { "" } else { "s" }
                );
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Hosts "));
        frame.render_widget(list, chunks[1]);
    }

    // Status bar
    let status = render_status_bar(app);
    frame.render_widget(status, chunks[2]);
}

fn render_host_detail(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Header with host info
    let host = app.current_host();
    let header_text = if let Some(host) = host {
        format!(" Host: {} | {} open ports | Last seen: {} ",
            host.ip,
            host.ports.len(),
            host.last_seen.format("%H:%M:%S")
        )
    } else {
        " No host selected ".to_string()
    };

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" Host Details "));
    frame.render_widget(header, chunks[0]);

    // Ports table
    if let Some(host) = host {
        if host.ports.is_empty() {
            let empty = Paragraph::new("No open ports found")
                .block(Block::default().borders(Borders::ALL).title(" Ports "));
            frame.render_widget(empty, chunks[1]);
        } else {
            let header_cells = ["Port", "State", "Service"].iter().map(|h| {
                Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD))
            });
            let header_row = Row::new(header_cells).height(1).bottom_margin(1);

            let rows: Vec<Row> = host.ports
                .iter()
                .enumerate()
                .map(|(i, port)| {
                    let style = if i == app.selected_port {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    let state_style = match port.state {
                        crate::scanner::PortState::Open => Style::default().fg(Color::Green),
                        crate::scanner::PortState::Closed => Style::default().fg(Color::Red),
                        crate::scanner::PortState::Filtered => Style::default().fg(Color::Yellow),
                    };

                    Row::new(vec![
                        Cell::from(port.number.to_string()),
                        Cell::from(format!("{:?}", port.state)).style(state_style),
                        Cell::from(port.service.as_deref().unwrap_or("-")),
                    ]).style(style)
                })
                .collect();

            let widths = [
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(50),
            ];

            let table = Table::new(rows, widths)
                .header(header_row)
                .block(Block::default().borders(Borders::ALL).title(" Open Ports "));
            frame.render_widget(table, chunks[1]);
        }
    }

    // Status bar
    let status = Paragraph::new(" Esc: Back | r: Rescan | q: Quit ")
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, chunks[2]);
}

fn render_help(frame: &mut Frame) {
    let help_text = vec![
        Line::from(Span::styled("Port Scanner Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, arrows    Move up/down"),
        Line::from("  Enter          View host details"),
        Line::from("  Esc            Go back"),
        Line::from(""),
        Line::from(Span::styled("Scanning", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  s              Start scan (enter target IP)"),
        Line::from("  S              Stop current scan"),
        Line::from("  p              Set port range"),
        Line::from("  r              Rescan selected host"),
        Line::from(""),
        Line::from(Span::styled("Port Range Formats", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  common         Common ports (default)"),
        Line::from("  top100         Top 100 ports"),
        Line::from("  22             Single port"),
        Line::from("  1-1024         Port range"),
        Line::from("  22,80,443      Comma-separated"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  c              Clear results"),
        Line::from("  ?              Toggle help"),
        Line::from("  q              Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, frame.area());
}

fn render_status_bar(app: &App) -> Paragraph<'static> {
    let message = app.error.as_ref()
        .or(app.message.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("s: Scan | p: Ports | Enter: Details | ?: Help | q: Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    Paragraph::new(format!(" {} ", message)).style(style)
}

fn render_input_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input = Paragraph::new(format!("{}|", value));
    frame.render_widget(input, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
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
        .split(popup[1])[1]
}
