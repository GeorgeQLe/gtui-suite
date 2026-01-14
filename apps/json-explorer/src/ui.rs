use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, InputMode, View};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            if app.input_mode == InputMode::Search {
                Constraint::Length(3)
            } else {
                Constraint::Length(0)
            },
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, app, chunks[0]);

    if app.input_mode == InputMode::Search {
        render_search(frame, app, chunks[1]);
    }

    render_main(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let stats = app.stats();

    let info = if let Some(ref s) = stats {
        format!("{} keys, {} depth", s.total_keys, s.max_depth)
    } else {
        "No JSON loaded".to_string()
    };

    let view_indicator = match app.view {
        View::Tree => "🌲 Tree",
        View::Raw => "📄 Raw",
        View::Stats => "📊 Stats",
        View::Query => "🔍 Query",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "JSON EXPLORER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(view_indicator, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::raw(info),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" JSON Viewer "));

    frame.render_widget(header, area);
}

fn render_search(frame: &mut Frame, app: &App, area: Rect) {
    let search = Paragraph::new(format!("🔍 {}", app.search_query))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    frame.render_widget(search, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Tree => render_tree(frame, app, area),
        View::Raw => render_raw(frame, app, area),
        View::Stats => render_stats(frame, app, area),
        View::Query => render_query(frame, app, area),
    }
}

fn render_tree(frame: &mut Frame, app: &App, area: Rect) {
    if app.flat_nodes.is_empty() {
        let empty = Paragraph::new("No JSON loaded")
            .block(Block::default().borders(Borders::ALL).title(" Tree "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .flat_nodes
        .iter()
        .enumerate()
        .map(|(i, flat_node)| {
            let node = &flat_node.node;
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let indent = "  ".repeat(node.depth);

            let expand_icon = if node.is_expandable() {
                if node.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let type_style = match node.value_type() {
                crate::models::ValueType::Null => Style::default().fg(Color::DarkGray),
                crate::models::ValueType::Boolean => Style::default().fg(Color::Magenta),
                crate::models::ValueType::Integer | crate::models::ValueType::Float => {
                    Style::default().fg(Color::Cyan)
                }
                crate::models::ValueType::String => Style::default().fg(Color::Green),
                crate::models::ValueType::Array => Style::default().fg(Color::Yellow),
                crate::models::ValueType::Object => Style::default().fg(Color::Blue),
            };

            let key_display = node.display_key();
            let value_display = node.display_value();

            let line = if node.key.is_some() || node.index.is_some() {
                Line::from(vec![
                    Span::raw(indent),
                    Span::styled(expand_icon, Style::default().fg(Color::DarkGray)),
                    Span::styled(key_display, Style::default().fg(Color::White)),
                    Span::styled(": ", Style::default().fg(Color::DarkGray)),
                    Span::styled(value_display, type_style),
                ])
            } else {
                Line::from(vec![
                    Span::raw(indent),
                    Span::styled(expand_icon, Style::default().fg(Color::DarkGray)),
                    Span::styled(value_display, type_style),
                ])
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tree ({} nodes) ", app.flat_nodes.len())),
    );

    frame.render_widget(list, area);
}

fn render_raw(frame: &mut Frame, app: &App, area: Rect) {
    let raw = app.raw_json();

    let lines: Vec<Line> = raw
        .lines()
        .enumerate()
        .map(|(i, line)| {
            Line::from(vec![
                Span::styled(
                    format!("{:4} │ ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(line),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Raw JSON "))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let Some(stats) = app.stats() else {
        let empty = Paragraph::new("No JSON loaded")
            .block(Block::default().borders(Borders::ALL).title(" Statistics "));
        frame.render_widget(empty, area);
        return;
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Total Keys: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{}", stats.total_keys)),
        ]),
        Line::from(vec![
            Span::styled("Max Depth: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{}", stats.max_depth)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Value Types:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  {} Objects: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{}", stats.object_count)),
        ]),
        Line::from(vec![
            Span::styled("  [] Arrays: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", stats.array_count)),
        ]),
        Line::from(vec![
            Span::styled("  \" Strings: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{}", stats.string_count)),
        ]),
        Line::from(vec![
            Span::styled("  # Numbers: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", stats.number_count)),
        ]),
        Line::from(vec![
            Span::styled("  ◉ Booleans: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{}", stats.boolean_count)),
        ]),
        Line::from(vec![
            Span::styled("  ∅ Nulls: ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{}", stats.null_count)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Statistics "),
    );

    frame.render_widget(paragraph, area);
}

fn render_query(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let query = Paragraph::new(format!(".{}", app.jq_query))
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query (jq-like path, e.g., .config.theme) "),
        );
    frame.render_widget(query, chunks[0]);

    let result = app.query_result.as_deref().unwrap_or("Enter a path query...");

    let result_widget = Paragraph::new(result)
        .block(Block::default().borders(Borders::ALL).title(" Result "))
        .wrap(Wrap { trim: false });
    frame.render_widget(result_widget, chunks[1]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
}
