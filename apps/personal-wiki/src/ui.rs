//! UI rendering for personal wiki.

use crate::app::{App, InputMode, Mode, Pane, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);

    if app.input_mode != InputMode::None {
        draw_input_dialog(f, app);
    }

    if app.show_help {
        draw_help(f);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] All", "[2] Recent", "[3] Categories", "[4] Orphans", "[5] Wanted"];
    let selected = match app.view {
        View::AllPages => 0,
        View::RecentChanges => 1,
        View::Categories => 2,
        View::Orphans => 3,
        View::Wanted => 4,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Wiki ({} pages, {} links) ",
            app.stats.total_pages,
            app.stats.total_links,
        )))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    draw_list(f, app, chunks[0]);
    draw_editor(f, app, chunks[1]);
    draw_sidebar(f, app, chunks[2]);
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.pane == Pane::List {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    match app.view {
        View::Categories => {
            let items: Vec<ListItem> = app
                .categories
                .iter()
                .map(|cat| {
                    ListItem::new(format!("{} ({})", cat.name, cat.page_count))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Categories ")
                    .border_style(border_style));
            f.render_widget(list, area);
        }
        View::Wanted => {
            let items: Vec<ListItem> = app
                .wanted_pages
                .iter()
                .map(|(title, count)| {
                    ListItem::new(Line::from(vec![
                        Span::styled(title, Style::default().fg(Color::Red)),
                        Span::raw(format!(" ({} links)", count)),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Wanted Pages ")
                    .border_style(border_style));
            f.render_widget(list, area);
        }
        _ => {
            let items: Vec<ListItem> = app
                .pages
                .iter()
                .enumerate()
                .map(|(i, page)| {
                    let mut style = Style::default();
                    if i == app.selected_index {
                        style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
                    }
                    if page.is_redirect() {
                        style = style.fg(Color::DarkGray);
                    }
                    if app.current_page.as_ref().map(|p| p.id) == Some(page.id) {
                        style = style.fg(Color::Cyan);
                    }

                    let prefix = if page.is_redirect() { "→ " } else { "" };
                    ListItem::new(format!("{}{}", prefix, page.title)).style(style)
                })
                .collect();

            let title = match app.view {
                View::AllPages => " Pages ",
                View::RecentChanges => " Recent ",
                View::Orphans => " Orphans ",
                _ => " Pages ",
            };

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style));
            f.render_widget(list, area);
        }
    }
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let title = app.current_page
        .as_ref()
        .map(|p| format!(" {} ", p.title))
        .unwrap_or_else(|| " Editor ".to_string());

    let border_style = if app.pane == Pane::Editor || app.mode == Mode::Editing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let mode_indicator = if app.mode == Mode::Editing { " [EDIT] " } else { "" };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{}{}", title, mode_indicator))
        .border_style(border_style);

    if app.editor_content.is_empty() || (app.editor_content.len() == 1 && app.editor_content[0].is_empty()) {
        let placeholder = Paragraph::new("Press 'n' to create a new page or 'o' to go to a page")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(placeholder, area);
    } else {
        let inner = block.inner(area);
        f.render_widget(block, area);

        let link_re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

        let text: Vec<Line> = app
            .editor_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                // Highlight [[links]]
                let mut spans = Vec::new();
                let mut last_end = 0;

                for mat in link_re.find_iter(line) {
                    if mat.start() > last_end {
                        spans.push(Span::raw(&line[last_end..mat.start()]));
                    }
                    spans.push(Span::styled(
                        mat.as_str(),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
                    ));
                    last_end = mat.end();
                }

                if last_end < line.len() {
                    spans.push(Span::raw(&line[last_end..]));
                }

                if spans.is_empty() {
                    spans.push(Span::raw(line.as_str()));
                }

                // Show cursor
                if app.mode == Mode::Editing && i == app.editor_cursor.0 {
                    let col = app.editor_cursor.1;
                    let full_line: String = spans.iter().map(|s| s.content.as_ref()).collect();

                    spans.clear();
                    if col < full_line.len() {
                        spans.push(Span::raw(full_line[..col].to_string()));
                        spans.push(Span::styled(
                            full_line[col..col+1].to_string(),
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));
                        spans.push(Span::raw(full_line[col+1..].to_string()));
                    } else {
                        spans.push(Span::raw(full_line));
                        spans.push(Span::styled(" ", Style::default().bg(Color::White)));
                    }
                }

                Line::from(spans)
            })
            .collect();

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let border_style = if app.pane == Pane::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    // Backlinks
    let backlink_items: Vec<ListItem> = app
        .backlinks
        .iter()
        .map(|page| {
            ListItem::new(Span::styled(&page.title, Style::default().fg(Color::Green)))
        })
        .collect();

    let backlinks_list = List::new(backlink_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Backlinks ({}) ", app.backlinks.len()))
            .border_style(border_style));
    f.render_widget(backlinks_list, chunks[0]);

    // Revisions
    let revision_items: Vec<ListItem> = app
        .revisions
        .iter()
        .take(10)
        .map(|rev| {
            let date = rev.created_at.format("%m/%d %H:%M").to_string();
            let summary = if rev.summary.is_empty() { "(no summary)" } else { &rev.summary };
            ListItem::new(vec![
                Line::from(Span::styled(date, Style::default().fg(Color::Yellow))),
                Line::from(Span::styled(summary, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let revisions_list = List::new(revision_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" History ({}) ", app.revisions.len()))
            .border_style(border_style));
    f.render_widget(revisions_list, chunks[1]);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Editing => "EDITING",
        Mode::Search => "SEARCH",
    };

    let info = format!(
        " {} | {} categories | {} orphans ",
        mode_str,
        app.stats.total_categories,
        app.stats.orphan_pages,
    );
    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(info_widget, chunks[0]);

    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | n new | o goto | e edit | b back | r random".to_string()
    });
    let msg_widget = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(msg_widget, chunks[1]);
}

fn draw_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_mode {
        InputMode::NewPage => " New Page ",
        InputMode::GoTo => " Go To Page ",
        InputMode::Search => " Search ",
        InputMode::EditSummary => " Edit Summary ",
        InputMode::None => " Input ",
    };

    if app.input_mode == InputMode::Search && !app.search_results.is_empty() {
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .take(8)
            .map(|page| {
                ListItem::new(vec![
                    Line::from(Span::styled(&page.title, Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(Span::styled(
                        page.preview(50),
                        Style::default().fg(Color::DarkGray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(list, area);
    } else {
        let input = Paragraph::new(app.input_buffer.as_str())
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(input, area);
    }
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 75, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k          Move up/down"),
        Line::from("  Tab          Switch pane"),
        Line::from("  1-5          Switch view"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  n            New page"),
        Line::from("  o            Go to page (creates if missing)"),
        Line::from("  Enter        Open selected"),
        Line::from("  e            Edit mode"),
        Line::from("  d            Delete"),
        Line::from("  b            Go back"),
        Line::from("  r            Random page"),
        Line::from("  Ctrl+S       Save (prompts for summary)"),
        Line::from(""),
        Line::from(Span::styled("Wiki Links", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  [[Page]]           Link to page"),
        Line::from("  [[Page|Text]]      Link with display text"),
        Line::from("  [[Category:Name]]  Add to category"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /            Search"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help, area);
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
