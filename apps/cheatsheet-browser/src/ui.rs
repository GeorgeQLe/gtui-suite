//! UI rendering for cheatsheet browser.

use crate::app::{App, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    match app.view {
        View::List => draw_list(f, app),
        View::Detail => draw_detail(f, app),
    }

    if app.show_help {
        draw_help(f);
    }
}

fn draw_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Search bar
    let search_title = if app.searching {
        format!(" Search: {}_ ", app.search)
    } else if !app.search.is_empty() {
        format!(" Filter: {} ", app.search)
    } else {
        " Cheatsheet Browser ".to_string()
    };

    let search_block = Block::default()
        .borders(Borders::ALL)
        .title(search_title)
        .border_style(if app.searching {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    f.render_widget(search_block, chunks[0]);

    // Cheatsheet list grouped by category
    let filtered = app.filtered_cheatsheets();
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_category = String::new();

    for (i, sheet) in filtered.iter().enumerate() {
        // Add category header if changed
        if sheet.category != current_category {
            if !current_category.is_empty() {
                items.push(ListItem::new(Line::from("")));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  {} ", sheet.category),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))));
            current_category = sheet.category.clone();
        }

        let style = if i == app.selected_index {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let source_indicator = match &sheet.source {
            crate::cheatsheets::Source::Bundled => "",
            crate::cheatsheets::Source::User { .. } => " [user]",
        };

        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(&sheet.topic, style),
            Span::styled(source_indicator, Style::default().fg(Color::DarkGray)),
        ])));
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} cheatsheets ", filtered.len())),
    );
    f.render_widget(list, chunks[1]);

    // Status bar
    let status = Paragraph::new("j/k: Navigate  Enter: View  /: Search  ?: Help  q: Quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[2]);
}

fn draw_detail(f: &mut Frame, app: &App) {
    let Some(sheet) = app.selected_cheatsheet() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(f.area());

    // Content area
    let lines: Vec<Line> = sheet
        .content
        .lines()
        .skip(app.scroll_offset)
        .map(|line| {
            // Simple syntax highlighting
            if line.starts_with("# ") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("## ") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("### ") {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Green),
                ))
            } else if line.starts_with("```") {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Magenta),
                ))
            } else if line.starts_with("- ") || line.starts_with("* ") {
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(line, Style::default()),
                ])
            } else {
                Line::from(line)
            }
        })
        .collect();

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", sheet.topic)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(content, chunks[0]);

    // Status bar
    let total_lines = sheet.content.lines().count();
    let status = Paragraph::new(format!(
        "j/k: Scroll  d/u: Page  n/p: Section  g/G: Top/Bottom  q: Back  ({}/{})",
        app.scroll_offset + 1,
        total_lines
    ))
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[1]);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "Cheatsheet Browser Help",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("List View:", Style::default().fg(Color::Cyan))),
        Line::from("  j/k, ↑/↓     Navigate up/down"),
        Line::from("  g/G          Go to top/bottom"),
        Line::from("  Enter        View cheatsheet"),
        Line::from("  /            Search"),
        Line::from("  Esc          Clear search"),
        Line::from("  q            Quit"),
        Line::from(""),
        Line::from(Span::styled("Detail View:", Style::default().fg(Color::Cyan))),
        Line::from("  j/k          Scroll up/down"),
        Line::from("  d/u          Page down/up"),
        Line::from("  n/p          Next/previous section"),
        Line::from("  g/G          Go to top/bottom"),
        Line::from("  q/Esc        Back to list"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Help "));

    f.render_widget(help, area);
}

fn centered_rect(px: u16, py: u16, area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - py) / 2),
            Constraint::Percentage(py),
            Constraint::Percentage((100 - py) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - px) / 2),
            Constraint::Percentage(px),
            Constraint::Percentage((100 - px) / 2),
        ])
        .split(v[1])[1]
}
