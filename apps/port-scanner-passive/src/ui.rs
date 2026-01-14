use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, InputMode, View};

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
    let status = if app.monitoring {
        Span::styled(" MONITORING ", Style::default().bg(Color::Green).fg(Color::Black))
    } else {
        Span::styled(" STOPPED ", Style::default().bg(Color::Red).fg(Color::White))
    };

    let duration = if app.monitoring {
        let d = app.stats.duration();
        format!(" {}h {}m ", d.num_hours(), d.num_minutes() % 60)
    } else {
        String::new()
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("PASSIVE SCANNER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        status,
        Span::styled(&duration, Style::default().fg(Color::Gray)),
        Span::raw(" | "),
        Span::raw(format!(
            "{} devices ({} online)",
            app.stats.total_devices,
            app.stats.online_devices
        )),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Passive Network Discovery "));

    frame.render_widget(header, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Devices => render_devices(frame, app, area),
        View::DeviceDetails => render_device_details(frame, app, area),
        View::Timeline => render_timeline(frame, app, area),
        View::Stats => render_stats(frame, app, area),
    }
}

fn render_devices(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if app.input_mode == InputMode::Search { Constraint::Length(3) } else { Constraint::Length(0) },
            Constraint::Min(5),
        ])
        .split(area);

    // Search bar
    if app.input_mode == InputMode::Search {
        let search = Paragraph::new(format!("/{}", app.search_query))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Search "));
        frame.render_widget(search, chunks[0]);
    }

    let filtered = app.filtered_devices();

    if filtered.is_empty() {
        let empty = Paragraph::new("No devices found. Press 'm' to start monitoring.")
            .block(Block::default().borders(Borders::ALL).title(" Devices "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, chunks[1]);
        return;
    }

    let header = Row::new(vec![
        Cell::from("").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("IP Address").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Hostname").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Vendor").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Method").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, device)| {
            let style = if i == app.selected_device {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status = if device.is_online() {
                Span::styled("● Online", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ Offline", Style::default().fg(Color::Gray))
            };

            let type_icon = device.device_type.map(|t| t.icon()).unwrap_or("❓");

            Row::new(vec![
                Cell::from(type_icon),
                Cell::from(device.ip.to_string()),
                Cell::from(device.hostname.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(device.vendor.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(device.device_type.map(|t| t.as_str()).unwrap_or("Unknown")),
                Cell::from(format!("{} {}", device.discovery_method.icon(), device.discovery_method.as_str())),
                Cell::from(status),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Devices ({}) ", filtered.len())),
    );

    frame.render_widget(table, chunks[1]);
}

fn render_device_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(device) = app.selected_device_data() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(5)])
        .split(area);

    // Device info
    let type_icon = device.device_type.map(|t| t.icon()).unwrap_or("❓");
    let info_lines = vec![
        format!("{} {}", type_icon, device.display_name()),
        format!("IP Address: {}", device.ip),
        format!("MAC Address: {}", device.mac.as_deref().unwrap_or("-")),
        format!("Vendor: {}", device.vendor.as_deref().unwrap_or("-")),
        format!("Type: {}", device.device_type.map(|t| t.as_str()).unwrap_or("Unknown")),
        format!("Discovery: {} {}", device.discovery_method.icon(), device.discovery_method.as_str()),
    ];

    let info = Paragraph::new(info_lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Device Information "));
    frame.render_widget(info, chunks[0]);

    // Services
    let services: Vec<ListItem> = device
        .services
        .iter()
        .map(|service| {
            let port_info = service
                .port
                .map(|p| format!(":{}", p))
                .unwrap_or_default();

            ListItem::new(Line::from(vec![
                Span::styled("  📡 ", Style::default().fg(Color::Yellow)),
                Span::raw(&service.name),
                Span::styled(
                    format!(" ({}{})", service.protocol, port_info),
                    Style::default().fg(Color::Gray),
                ),
            ]))
        })
        .collect();

    let services_list = List::new(services).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Services ({}) ", device.services.len())),
    );

    frame.render_widget(services_list, chunks[1]);
}

fn render_timeline(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .events
        .iter()
        .enumerate()
        .map(|(i, event)| {
            let style = if i == app.selected_event {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let time = event.timestamp.format("%H:%M:%S").to_string();

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", time),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", event.event_type.icon()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(&event.description),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Timeline ({} events) ", app.events.len())),
    );

    frame.render_widget(list, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Overall stats
    let duration = app.stats.duration();
    let stats_lines = vec![
        format!("Total Devices: {}", app.stats.total_devices),
        format!("Online Devices: {}", app.stats.online_devices),
        format!("Total Services: {}", app.stats.total_services),
        format!("Packets Captured: {}", app.stats.packets_captured),
        String::new(),
        format!("Monitoring Duration: {}h {}m", duration.num_hours(), duration.num_minutes() % 60),
    ];

    let stats = Paragraph::new(stats_lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Statistics "));
    frame.render_widget(stats, chunks[0]);

    // Protocol breakdown
    let protocol_lines: Vec<String> = app
        .protocols
        .iter()
        .map(|(method, enabled)| {
            let status = if *enabled { "✓" } else { "✗" };
            format!("{} {} {}: {}", status, method.icon(), method.as_str(), method.description())
        })
        .collect();

    let protocols = Paragraph::new(protocol_lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Protocols "));
    frame.render_widget(protocols, chunks[1]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
}
