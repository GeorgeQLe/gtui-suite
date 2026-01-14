use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Filter, View};

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

    match app.view {
        View::Feeds => render_feeds(frame, app, chunks[1]),
        View::Articles => render_articles(frame, app, chunks[1]),
        View::Reader => render_reader(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.view {
        View::Feeds => "Feeds",
        View::Articles => "Articles",
        View::Reader => "Reader",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "RSS READER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(view_name, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::raw(format!("{} unread", app.total_unread())),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Feed Reader "));

    frame.render_widget(header, area);
}

fn render_feeds(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .feeds
        .iter()
        .enumerate()
        .map(|(i, feed)| {
            let style = if i == app.selected_feed {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let unread_style = if feed.unread_count > 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::raw("📰 "),
                Span::styled(&feed.title, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("({})", feed.unread_count),
                    unread_style,
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Feeds "));

    frame.render_widget(list, area);
}

fn render_articles(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_articles();

    let filter_label = match app.filter {
        Filter::All => "All",
        Filter::Unread => "Unread",
        Filter::Starred => "Starred",
    };

    if filtered.is_empty() {
        let empty = Paragraph::new(format!("No {} articles", filter_label.to_lowercase()))
            .block(Block::default().borders(Borders::ALL).title(format!(" Articles ({}) ", filter_label)))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, article)| {
            let style = if i == app.selected_article {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let read_icon = if article.read { "  " } else { "● " };
            let star_icon = if article.starred { "★ " } else { "  " };

            let title_style = if article.read {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let age = format_age(article.published);

            let line = Line::from(vec![
                Span::styled(read_icon, Style::default().fg(Color::Cyan)),
                Span::styled(star_icon, Style::default().fg(Color::Yellow)),
                Span::styled(&article.title, title_style),
                Span::raw(" "),
                Span::styled(age, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Articles - {} ({}) ",
            filter_label,
            filtered.len()
        )));

    frame.render_widget(list, area);
}

fn render_reader(frame: &mut Frame, app: &App, area: Rect) {
    let Some(article) = app.current_article() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(5)])
        .split(area);

    // Article header
    let star = if article.starred { " ★" } else { "" };
    let header_lines = vec![
        Line::from(Span::styled(
            format!("{}{}", article.title, star),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Published: ", Style::default().fg(Color::Gray)),
            Span::raw(article.published.format("%Y-%m-%d %H:%M").to_string()),
        ]),
    ];

    let header = Paragraph::new(header_lines)
        .block(Block::default().borders(Borders::ALL).title(" Article "));

    frame.render_widget(header, chunks[0]);

    // Article content
    let content_lines: Vec<Line> = article
        .content
        .lines()
        .skip(app.scroll_offset)
        .map(|line| Line::raw(line))
        .collect();

    let content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::ALL).title(" Content "))
        .wrap(Wrap { trim: false });

    frame.render_widget(content, chunks[1]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn format_age(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now - dt;

    if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else {
        format!("{}m", duration.num_minutes())
    }
}
