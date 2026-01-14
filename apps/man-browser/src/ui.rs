use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            if app.view == View::Search {
                Constraint::Length(3)
            } else {
                Constraint::Length(0)
            },
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    if app.view == View::Search {
        render_search(frame, app, chunks[1]);
    }

    match app.view {
        View::List | View::Search => render_list(frame, app, chunks[2]),
        View::Reader => render_reader(frame, app, chunks[2]),
    }

    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let section_info = app
        .section_filter
        .map(|s| {
            let page = app.pages.iter().find(|p| p.section == s);
            let name = page.map(|p| p.section_name()).unwrap_or("Unknown");
            format!(" | Section {}: {}", s, name)
        })
        .unwrap_or_default();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "MAN BROWSER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(&section_info),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Manual Pages "));

    frame.render_widget(header, area);
}

fn render_search(frame: &mut Frame, app: &App, area: Rect) {
    let search = Paragraph::new(format!("🔍 {}_", app.search_query))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    frame.render_widget(search, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_pages();

    if visible.is_empty() {
        let empty = Paragraph::new("No manual pages found")
            .block(Block::default().borders(Borders::ALL).title(" Pages "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, page)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let section_style = match page.section {
                1 => Style::default().fg(Color::Green),
                2 => Style::default().fg(Color::Yellow),
                3 => Style::default().fg(Color::Blue),
                5 => Style::default().fg(Color::Magenta),
                8 => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::DarkGray),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("({}) ", page.section),
                    section_style,
                ),
                Span::styled(&page.name, Style::default().fg(Color::White)),
                Span::raw(" - "),
                Span::styled(&page.description, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Manual Pages ({}) ", visible.len())),
    );

    frame.render_widget(list, area);
}

fn render_reader(frame: &mut Frame, app: &App, area: Rect) {
    let Some(page) = app.selected_page() else {
        return;
    };

    let lines: Vec<Line> = page
        .content
        .lines()
        .skip(app.scroll_offset)
        .map(|line| {
            // Simple syntax highlighting for man page format
            let style = if line.chars().all(|c| c.is_uppercase() || c.is_whitespace()) && !line.is_empty() {
                // Section headers
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if line.starts_with("       -") {
                // Options
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            Line::styled(line, style)
        })
        .collect();

    let title = format!(" {}({}) - {} ", page.name, page.section, page.description);

    let content = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    frame.render_widget(content, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
