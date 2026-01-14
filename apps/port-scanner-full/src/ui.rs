use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, InputMode, View};
use crate::models::PortState;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " Port Scanner - {} | {} | {} ",
        app.scan_type.as_str(),
        app.profile.as_str(),
        app.timing.as_str()
    );

    let header = Paragraph::new(Line::from(vec![
        Span::styled("PORT SCANNER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(
            if app.scanning { "SCANNING" } else { "READY" },
            if app.scanning {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            },
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(header, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Targets => render_targets(frame, app, area),
        View::Scanning => render_scanning(frame, app, area),
        View::Results => render_results(frame, app, area),
        View::Details => render_details(frame, app, area),
        View::Profiles => render_profiles(frame, app, area),
    }
}

fn render_targets(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Target list
    let items: Vec<ListItem> = app
        .targets
        .iter()
        .enumerate()
        .map(|(i, target)| {
            let style = if i == app.selected_target {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled("🎯 ", Style::default().fg(Color::Yellow)),
                Span::raw(&target.host),
                Span::styled(
                    format!(" ({} ports)", target.ports.len()),
                    Style::default().fg(Color::Gray),
                ),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Targets ({}) ", app.targets.len())),
    );

    frame.render_widget(list, chunks[0]);

    // Scan options
    let options = vec![
        format!("Scan Type: {}", app.scan_type.as_str()),
        format!("Profile: {} - {}", app.profile.as_str(), app.profile.description()),
        format!("Timing: {}", app.timing.as_str()),
        format!("Version Detection: {}", if app.version_detection { "On" } else { "Off" }),
        format!("OS Detection: {}", if app.os_detection { "On" } else { "Off" }),
        String::new(),
        format!("Ports to scan: {}", app.profile.port_count()),
    ];

    let options_widget = Paragraph::new(options.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Scan Options "));

    frame.render_widget(options_widget, chunks[1]);

    // Input overlay
    if app.input_mode == InputMode::Input {
        let input_area = centered_rect(50, 20, area);
        frame.render_widget(Clear, input_area);

        let input = Paragraph::new(app.target_input.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Add Target (hostname or IP) "),
            )
            .style(Style::default().fg(Color::Yellow));

        frame.render_widget(input, input_area);

        frame.set_cursor_position(Position::new(
            input_area.x + 1 + app.target_input.len() as u16,
            input_area.y + 1,
        ));
    }
}

fn render_scanning(frame: &mut Frame, app: &App, area: Rect) {
    let progress_text = if let Some(ref progress) = app.progress {
        vec![
            format!("Progress: {:.1}%", progress.percent_complete()),
            format!("Scanned: {} / {}", progress.scanned_ports, progress.total_ports),
            format!("Open ports found: {}", progress.open_found),
            format!("Rate: {:.0} packets/sec", progress.packets_per_second),
            String::new(),
            format!(
                "Current port: {}",
                progress.current_port.map(|p| p.to_string()).unwrap_or_default()
            ),
        ]
    } else {
        vec!["Initializing scan...".to_string()]
    };

    let content = Paragraph::new(progress_text.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Scanning "))
        .alignment(Alignment::Center);

    frame.render_widget(content, area);
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
    if app.scan_history.is_empty() {
        let empty = Paragraph::new("No scan results yet. Add targets and press 's' to scan.")
            .block(Block::default().borders(Borders::ALL).title(" Results "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Target").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Open").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Time").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Duration").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = app
        .scan_history
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let style = if i == app.selected_result {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let duration = result
                .duration()
                .map(|d| format!("{}s", d.num_seconds()))
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(result.target.clone()),
                Cell::from(result.scan_type.as_str()),
                Cell::from(format!("{}", result.open_ports().len()))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(result.started_at.format("%H:%M:%S").to_string()),
                Cell::from(duration),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Scan History ({}) ", app.scan_history.len())),
    );

    frame.render_widget(table, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(result) = app.current_result() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(10)])
        .split(area);

    // Host info
    let host_info = vec![
        format!("Target: {} ({})", result.target, result.ip.map(|ip| ip.to_string()).unwrap_or_default()),
        format!("Hostname: {}", result.hostname.as_deref().unwrap_or("-")),
        format!("OS: {}", result.os_detection.as_ref().map(|os| format!("{} ({}%)", os.name, os.accuracy)).unwrap_or_else(|| "-".to_string())),
    ];

    let info = Paragraph::new(host_info.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Host Information "));
    frame.render_widget(info, chunks[0]);

    // Port table
    let header = Row::new(vec![
        Cell::from("Port").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("State").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Service").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Version").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Response").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = result
        .ports
        .iter()
        .enumerate()
        .map(|(i, port)| {
            let style = if i == app.selected_port {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let state_style = match port.state {
                PortState::Open => Style::default().fg(Color::Green),
                PortState::Closed => Style::default().fg(Color::Red),
                PortState::Filtered => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Gray),
            };

            let service = port
                .service
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();

            let version = port
                .service
                .as_ref()
                .map(|s| s.display())
                .unwrap_or_default();

            let response = port
                .response_time_ms
                .map(|ms| format!("{}ms", ms))
                .unwrap_or_default();

            Row::new(vec![
                Cell::from(format!("{}/{}", port.port, port.protocol.as_str())),
                Cell::from(format!("{} {}", port.state.icon(), port.state.as_str())).style(state_style),
                Cell::from(service),
                Cell::from(version),
                Cell::from(response),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Ports ({} open) ", result.open_ports().len())),
    );

    frame.render_widget(table, chunks[1]);
}

fn render_profiles(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 50, area);
    frame.render_widget(Clear, popup_area);

    let profiles = vec![
        ("1", "Quick", "Top 100 ports, no version detection"),
        ("2", "Standard", "Top 1000 ports, version detection"),
        ("3", "Comprehensive", "All 65535 ports, full detection"),
        ("4", "Stealth", "SYN scan, slow timing, randomized"),
    ];

    let items: Vec<ListItem> = profiles
        .iter()
        .map(|(key, name, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", key), Style::default().fg(Color::Yellow)),
                Span::styled(*name, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Profile "));

    frame.render_widget(list, popup_area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
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
