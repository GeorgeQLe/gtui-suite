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

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CLIPBOARD MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::raw(format!("{} items", stats.total_entries)),
        Span::raw(" | "),
        Span::styled(format!("📌 {}", stats.pinned_count), Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(format!("⭐ {}", stats.favorite_count), Style::default().fg(Color::Magenta)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Clipboard History "));

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
        View::List => render_list(frame, app, area),
        View::Preview => render_preview(frame, app, area),
        View::Categories => render_categories(frame, app, area),
        View::Stats => render_stats(frame, app, area),
        View::Search => render_list(frame, app, area),
    }
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_entries();

    if filtered.is_empty() {
        let empty = Paragraph::new("No clipboard entries. Copy something to see it here!")
            .block(Block::default().borders(Borders::ALL).title(" Entries "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let pinned = if entry.pinned { "📌 " } else { "" };
            let favorite = if entry.favorite { "⭐ " } else { "" };
            let type_icon = entry.content_type.icon();

            let age = format_age(entry.created_at);
            let preview = entry.preview(50);

            let line = Line::from(vec![
                Span::styled(pinned, Style::default().fg(Color::Yellow)),
                Span::styled(favorite, Style::default().fg(Color::Magenta)),
                Span::styled(format!("{} ", type_icon), Style::default().fg(Color::Cyan)),
                Span::raw(preview),
                Span::styled(
                    format!(" ({})", age),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Entries ({}) ", filtered.len())),
    );

    frame.render_widget(list, area);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // List on left
    render_list(frame, app, chunks[0]);

    // Preview on right
    if let Some(entry) = app.selected_entry() {
        let info_lines = vec![
            format!("Type: {} {}", entry.content_type.icon(), entry.content_type.as_str()),
            format!("Length: {} chars, {} lines", entry.char_count(), entry.line_count()),
            format!("Created: {}", entry.created_at.format("%Y-%m-%d %H:%M")),
            format!("Used: {} times", entry.use_count),
            format!("Pinned: {}", if entry.pinned { "Yes" } else { "No" }),
            format!(
                "Category: {}",
                entry.category.as_deref().unwrap_or("None")
            ),
            String::new(),
            "─".repeat(40),
            String::new(),
        ];

        let mut content_lines: Vec<Line> = info_lines
            .into_iter()
            .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::Gray))))
            .collect();

        // Add content lines
        for line in entry.content.lines().take(20) {
            content_lines.push(Line::from(line.to_string()));
        }

        if entry.line_count() > 20 {
            content_lines.push(Line::from(Span::styled(
                format!("... ({} more lines)", entry.line_count() - 20),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let preview = Paragraph::new(content_lines)
            .block(Block::default().borders(Borders::ALL).title(" Preview "))
            .wrap(Wrap { trim: false });

        frame.render_widget(preview, chunks[1]);
    }
}

fn render_categories(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let style = if i == app.selected_category {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let color_indicator = Span::styled("●", Style::default().fg(cat.color.to_ratatui_color()));

            ListItem::new(Line::from(vec![
                color_indicator,
                Span::raw(" "),
                Span::raw(&cat.name),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Select Category (Enter to assign, Esc to cancel) "),
    );

    frame.render_widget(list, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let stats = app.stats();

    let lines = vec![
        format!("Total Entries: {}", stats.total_entries),
        format!("Pinned: {}", stats.pinned_count),
        format!("Favorites: {}", stats.favorite_count),
        format!("Total Characters: {}", stats.total_chars),
        String::new(),
        "Categories:".to_string(),
    ];

    let mut all_lines: Vec<Line> = lines.into_iter().map(Line::from).collect();

    for (cat, count) in &stats.categories {
        all_lines.push(Line::from(format!("  {}: {}", cat, count)));
    }

    all_lines.push(Line::from(String::new()));
    all_lines.push(Line::from("Press Esc to go back"));

    let paragraph = Paragraph::new(all_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Statistics "),
    );

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
}

fn format_age(dt: chrono::DateTime<chrono::Utc>) -> String {
    let duration = chrono::Utc::now() - dt;

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else {
        format!("{}d ago", duration.num_days())
    }
}
