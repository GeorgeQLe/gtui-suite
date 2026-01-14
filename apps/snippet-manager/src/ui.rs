use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, EditField, InputMode, View};
use crate::models::Language;

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

    let filters: Vec<&str> = [
        app.language_filter.map(|_| "lang"),
        app.tag_filter.as_ref().map(|_| "tag"),
        app.show_favorites_only.then_some("fav"),
    ]
    .into_iter()
    .flatten()
    .collect();

    let filter_text = if filters.is_empty() {
        String::new()
    } else {
        format!(" [{}]", filters.join(", "))
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SNIPPET MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::raw(format!("{} snippets", stats.total)),
        Span::raw(" | "),
        Span::styled(format!("⭐ {}", stats.favorites), Style::default().fg(Color::Yellow)),
        Span::styled(filter_text, Style::default().fg(Color::Magenta)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Code Snippets "));

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
        View::Create | View::Edit => render_editor(frame, app, area),
        View::SelectLanguage => render_language_select(frame, app, area),
        View::Tags => render_tags(frame, app, area),
        View::Stats => render_stats(frame, app, area),
    }
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_snippets();

    if filtered.is_empty() {
        let empty = Paragraph::new("No snippets found. Press 'n' to create one!")
            .block(Block::default().borders(Borders::ALL).title(" Snippets "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, snippet)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let fav = if snippet.favorite { "⭐ " } else { "" };
            let lang_icon = snippet.language.icon();

            let tags = if snippet.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", snippet.tags.join(", "))
            };

            let line = Line::from(vec![
                Span::styled(fav, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} ", lang_icon), Style::default().fg(Color::Cyan)),
                Span::raw(&snippet.title),
                Span::styled(tags, Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" ({} lines)", snippet.line_count()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Snippets ({}) ", filtered.len())),
    );

    frame.render_widget(list, area);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    render_list(frame, app, chunks[0]);

    if let Some(snippet) = app.selected_snippet() {
        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(Color::Gray)),
                Span::styled(&snippet.title, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Language: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} {}", snippet.language.icon(), snippet.language.as_str()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Tags: ", Style::default().fg(Color::Gray)),
                Span::raw(if snippet.tags.is_empty() {
                    "None".to_string()
                } else {
                    snippet.tags.join(", ")
                }),
            ]),
            Line::from(vec![
                Span::styled("Used: ", Style::default().fg(Color::Gray)),
                Span::raw(format!("{} times", snippet.use_count)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "─".repeat(50),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
        ];

        for (i, code_line) in snippet.code.lines().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:3} │ ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(code_line),
            ]));
        }

        let preview = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Preview "))
            .wrap(Wrap { trim: false });

        frame.render_widget(preview, chunks[1]);
    }
}

fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let Some(snippet) = &app.edit_snippet else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    let title_style = if app.edit_field == EditField::Title {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let title_content = if app.edit_field == EditField::Title
        && matches!(app.input_mode, InputMode::EditTitle)
    {
        &app.edit_buffer
    } else {
        &snippet.title
    };

    let title = Paragraph::new(title_content.as_str())
        .style(title_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Title {} ", if app.edit_field == EditField::Title { "▶" } else { "" })),
        );
    frame.render_widget(title, chunks[0]);

    let lang = Paragraph::new(format!("{} {}", snippet.language.icon(), snippet.language.as_str()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Language (press 'l' to change) "),
        );
    frame.render_widget(lang, chunks[1]);

    let code_style = if app.edit_field == EditField::Code {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let code_content = if app.edit_field == EditField::Code
        && matches!(app.input_mode, InputMode::EditCode)
    {
        &app.edit_buffer
    } else {
        &snippet.code
    };

    let code = Paragraph::new(code_content.as_str())
        .style(code_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Code {} ", if app.edit_field == EditField::Code { "▶" } else { "" })),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(code, chunks[2]);

    let tags_style = if app.edit_field == EditField::Tags {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let tags_content = if app.edit_field == EditField::Tags
        && matches!(app.input_mode, InputMode::EditTags)
    {
        app.edit_buffer.clone()
    } else {
        snippet.tags.join(", ")
    };

    let tags = Paragraph::new(tags_content)
        .style(tags_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Tags (comma-separated) {} ", if app.edit_field == EditField::Tags { "▶" } else { "" })),
        );
    frame.render_widget(tags, chunks[3]);
}

fn render_language_select(frame: &mut Frame, app: &App, area: Rect) {
    let languages = Language::all();

    let items: Vec<ListItem> = languages
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            let style = if i == app.selected_language {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", lang.icon()), Style::default().fg(Color::Cyan)),
                Span::raw(lang.as_str()),
                Span::styled(
                    format!(" ({})", lang.extension()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Select Language "),
    );

    frame.render_widget(list, area);
}

fn render_tags(frame: &mut Frame, app: &App, area: Rect) {
    if app.all_tags.is_empty() {
        let empty = Paragraph::new("No tags yet")
            .block(Block::default().borders(Borders::ALL).title(" Tags "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .all_tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let style = if i == app.selected_tag {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled("🏷️ ", Style::default().fg(Color::Yellow)),
                Span::raw(tag),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Filter by Tag "),
    );

    frame.render_widget(list, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let stats = app.stats();

    let mut lines = vec![
        Line::from(format!("Total Snippets: {}", stats.total)),
        Line::from(format!("Favorites: {}", stats.favorites)),
        Line::from(""),
        Line::from("By Language:"),
    ];

    for (lang, count) in &stats.by_language {
        lines.push(Line::from(format!("  {} {}: {}", lang.icon(), lang.as_str(), count)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Popular Tags:"));

    for (tag, count) in &stats.popular_tags {
        lines.push(Line::from(format!("  🏷️ {}: {}", tag, count)));
    }

    let paragraph = Paragraph::new(lines).block(
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
