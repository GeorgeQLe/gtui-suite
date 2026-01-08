use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, InputMode, View};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    render_tabs(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);

    // Render overlays
    match app.input_mode {
        InputMode::Search => render_search(frame, app),
        InputMode::StreamDetail => render_stream_detail(frame, app),
        InputMode::KeyDetail => render_key_detail(frame, app),
        InputMode::AddEntry => render_add_entry(frame, app),
        InputMode::Normal => {}
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = View::all().iter().map(|v| v.name()).collect();
    let selected = View::all().iter().position(|v| *v == app.view).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Redis Monitor "))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(selected);

    frame.render_widget(tabs, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Overview => render_overview(frame, app, area),
        View::Streams => render_streams(frame, app, area),
        View::Keys => render_keys(frame, app, area),
        View::PubSub => render_pubsub(frame, app, area),
    }
}

fn render_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(5)])
        .split(area);

    // Server info
    let info = &app.info;
    let info_lines = vec![
        Line::from(vec![
            Span::styled("Redis Version: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&info.version),
        ]),
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(info.uptime_display()),
        ]),
        Line::from(vec![
            Span::styled("Memory: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&info.used_memory_human),
        ]),
        Line::from(vec![
            Span::styled("Clients: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(info.connected_clients.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Commands: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_number(info.total_commands_processed)),
        ]),
        Line::from(vec![
            Span::styled("Hit Ratio: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.1}%", info.hit_ratio())),
        ]),
    ];

    let info_widget = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Server Info "));
    frame.render_widget(info_widget, chunks[0]);

    // Quick stats
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);

    let streams_count = app.streams.len();
    let keys_count = app.keys.len();
    let channels_count = app.channels.len();

    let streams_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            streams_count.to_string(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from("Streams"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    let keys_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            keys_count.to_string(),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from("Keys"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    let channels_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            channels_count.to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from("Channels"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    frame.render_widget(streams_widget, stats_chunks[0]);
    frame.render_widget(keys_widget, stats_chunks[1]);
    frame.render_widget(channels_widget, stats_chunks[2]);
}

fn render_streams(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Length", "Groups", "First ID", "Last ID"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.streams)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(s.name.clone()),
                Cell::from(format_number(s.length)),
                Cell::from(s.groups.len().to_string()),
                Cell::from(s.first_entry_id.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(s.last_entry_id.clone().unwrap_or_else(|| "-".to_string())),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Streams "));

    frame.render_widget(table, area);
}

fn render_keys(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Type", "TTL", "Memory"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.keys)
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let ttl_str = match k.ttl {
                Some(t) if t > 0 => format!("{}s", t),
                Some(_) => "expired".to_string(),
                None => "-".to_string(),
            };

            let memory_str = k
                .memory
                .map(|m| format_bytes(m))
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(k.name.clone()),
                Cell::from(k.key_type.as_str()),
                Cell::from(ttl_str),
                Cell::from(memory_str),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(45),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Keys "));

    frame.render_widget(table, area);
}

fn render_pubsub(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Channel/Pattern", "Subscribers", "Type"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.channels)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(c.subscribers.to_string()),
                Cell::from(if c.pattern { "pattern" } else { "channel" }),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Pub/Sub Channels "));

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let help = match app.view {
        View::Overview => "Tab:switch  s:streams  p:pubsub  r:refresh  q:quit",
        View::Streams => "Tab:switch  Enter:details  a:add  /:search  r:refresh  q:quit",
        View::Keys => "Tab:switch  Enter:details  /:search  r:refresh  q:quit",
        View::PubSub => "Tab:switch  /:search  r:refresh  q:quit",
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let status_style = if app.connected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let status_widget = Paragraph::new(status)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));

    let help_widget = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Right);

    frame.render_widget(status_widget, chunks[0]);
    frame.render_widget(help_widget, chunks[1]);
}

fn render_search(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(" Search "));

    frame.render_widget(search, area);
}

fn render_stream_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(80, 70, frame.area());
    frame.render_widget(Clear, area);

    let stream_name = app.selected_stream.as_deref().unwrap_or("Unknown");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(5), Constraint::Min(5)])
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Stream: {} ", stream_name));
    frame.render_widget(block, area);

    // Stream info
    if let Some(stream) = app.streams.iter().find(|s| s.name == stream_name) {
        let info = vec![
            Line::from(format!("Length: {}", format_number(stream.length))),
            Line::from(format!("Groups: {}", stream.groups.len())),
            Line::from(format!(
                "First ID: {}",
                stream.first_entry_id.as_deref().unwrap_or("-")
            )),
        ];
        let info_widget =
            Paragraph::new(info).block(Block::default().borders(Borders::ALL).title(" Info "));
        frame.render_widget(info_widget, chunks[0]);
    }

    // Entries
    let items: Vec<ListItem> = app
        .stream_entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            let fields: String = e
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            ListItem::new(format!("{}: {}", e.id, fields)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Entries (Esc to close) "));
    frame.render_widget(list, chunks[1]);
}

fn render_key_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, area);

    let key_name = app.selected_key.as_deref().unwrap_or("Unknown");
    let key = app.keys.iter().find(|k| k.name == key_name);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Key: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(key_name),
        ]),
    ];

    if let Some(k) = key {
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(k.key_type.as_str()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("TTL: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                k.ttl
                    .map(|t| format!("{}s", t))
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Value:",
        Style::default().add_modifier(Modifier::BOLD),
    )));

    if let Some(value) = &app.key_value {
        lines.push(Line::from(value.clone()));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Esc to close",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Key Details "))
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_add_entry(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    let mut lines = vec![
        Line::from(Span::styled(
            "Stream:",
            Style::default().fg(if app.add_field_idx == 0 {
                Color::Yellow
            } else {
                Color::White
            }),
        )),
        Line::from(format!(
            "> {}{}",
            app.add_stream,
            if app.add_field_idx == 0 { "_" } else { "" }
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Fields:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];

    for (i, (key, value)) in app.add_fields.iter().enumerate() {
        let key_idx = 1 + i * 2;
        let val_idx = 2 + i * 2;

        let key_style = if app.add_field_idx == key_idx {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let val_style = if app.add_field_idx == val_idx {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled("Key: ", key_style),
            Span::raw(format!(
                "{}{}",
                key,
                if app.add_field_idx == key_idx { "_" } else { "" }
            )),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Value: ", val_style),
            Span::raw(format!(
                "{}{}",
                value,
                if app.add_field_idx == val_idx { "_" } else { "" }
            )),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Tab: next field | Enter: add | Esc: cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Add Stream Entry "));

    frame.render_widget(paragraph, area);
}

fn get_filtered_items<'a, T>(app: &App, items: &'a [T]) -> Vec<&'a T> {
    if app.search_query.is_empty() {
        items.iter().collect()
    } else {
        app.filtered_indices
            .iter()
            .filter_map(|&i| items.get(i))
            .collect()
    }
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

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
