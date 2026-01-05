//! UI rendering for Zettelkasten.

use crate::app::{App, InputMode, Mode, Pane, View};
use crate::models::ZettelType;
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
    let titles = vec!["[1] List", "[2] Tags", "[3] Types"];
    let selected = match app.view {
        View::List => 0,
        View::Tags => 1,
        View::Types => 2,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Zettelkasten ({} zettels, {} links) ",
            app.stats.total_zettels,
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
            Constraint::Percentage(30),
            Constraint::Percentage(45),
            Constraint::Percentage(25),
        ])
        .split(area);

    draw_list(f, app, chunks[0]);
    draw_editor(f, app, chunks[1]);
    draw_links(f, app, chunks[2]);
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .zettels
        .iter()
        .enumerate()
        .map(|(i, z)| {
            let type_symbol = z.zettel_type.symbol();
            let mut style = Style::default();

            if i == app.selected_index {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }

            let color = match z.zettel_type {
                ZettelType::Fleeting => Color::Yellow,
                ZettelType::Literature => Color::Cyan,
                ZettelType::Permanent => Color::Green,
                ZettelType::Hub => Color::Magenta,
            };

            if app.current_zettel.as_ref().map(|c| c.db_id) == Some(z.db_id) {
                style = style.fg(color);
            }

            let id_short = if z.id.len() > 8 { &z.id[..8] } else { &z.id };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{} ", type_symbol), Style::default().fg(color)),
                    Span::styled(&z.title, style.add_modifier(Modifier::BOLD)),
                ]),
                Line::from(Span::styled(
                    format!("  {} | {} words", id_short, z.word_count()),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let border_style = if app.pane == Pane::List {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title = if let Some(t) = app.filter_type {
        format!(" {} ", t.label())
    } else {
        " Zettels ".to_string()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style));

    f.render_widget(list, area);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let title = app.current_zettel
        .as_ref()
        .map(|z| format!(" {} - {} ", z.formatted_id(), z.title))
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

    // Show zettel metadata above content
    if let Some(z) = &app.current_zettel {
        let inner = block.inner(area);
        f.render_widget(block, area);

        let meta_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(5)])
            .split(inner);

        // Metadata line
        let tags_str = if z.tags.is_empty() {
            "no tags".to_string()
        } else {
            z.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")
        };

        let meta = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(z.zettel_type.symbol(), Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(z.zettel_type.label(), Style::default().fg(Color::Yellow)),
                Span::raw(" | "),
                Span::styled(tags_str, Style::default().fg(Color::Cyan)),
            ]),
        ]);
        f.render_widget(meta, meta_chunks[0]);

        // Content
        let text: Vec<Line> = app
            .editor_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if app.mode == Mode::Editing && i == app.editor_cursor.0 {
                    let col = app.editor_cursor.1;
                    let mut spans = Vec::new();
                    if col < line.len() {
                        spans.push(Span::raw(&line[..col]));
                        spans.push(Span::styled(
                            &line[col..col+1],
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));
                        spans.push(Span::raw(&line[col+1..]));
                    } else {
                        spans.push(Span::raw(line.as_str()));
                        spans.push(Span::styled(" ", Style::default().bg(Color::White)));
                    }
                    Line::from(spans)
                } else {
                    Line::from(line.as_str())
                }
            })
            .collect();

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, meta_chunks[1]);
    } else {
        let placeholder = Paragraph::new("Press 'n' to create a new zettel")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(placeholder, area);
    }
}

fn draw_links(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let border_style = if app.pane == Pane::Links {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    // Outgoing links
    let outgoing_items: Vec<ListItem> = app
        .outgoing_links
        .iter()
        .map(|(z, link_type)| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(z.zettel_type.symbol(), Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::raw(&z.title),
                ]),
                Line::from(Span::styled(
                    format!("  → {}", link_type.label()),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let outgoing_list = List::new(outgoing_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Links ({}) ", app.outgoing_links.len()))
            .border_style(border_style));
    f.render_widget(outgoing_list, chunks[0]);

    // Incoming links (backlinks)
    let incoming_items: Vec<ListItem> = app
        .incoming_links
        .iter()
        .map(|(z, link_type)| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(z.zettel_type.symbol(), Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::raw(&z.title),
                ]),
                Line::from(Span::styled(
                    format!("  ← {}", link_type.label()),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let incoming_list = List::new(incoming_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Backlinks ({}) ", app.incoming_links.len()))
            .border_style(border_style));
    f.render_widget(incoming_list, chunks[1]);
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
        " {} | {} fleeting | {} lit | {} perm | {} hub ",
        mode_str,
        app.stats.fleeting,
        app.stats.literature,
        app.stats.permanent,
        app.stats.hubs,
    );
    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(info_widget, chunks[0]);

    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | n new | e edit | t tag | l link | T type".to_string()
    });
    let msg_widget = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(msg_widget, chunks[1]);
}

fn draw_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_mode {
        InputMode::NewZettel => " New Zettel ",
        InputMode::AddTag => " Add Tag ",
        InputMode::AddLink => " Link to Zettel ID ",
        InputMode::Search => " Search ",
        InputMode::None => " Input ",
    };

    if app.input_mode == InputMode::Search && !app.search_results.is_empty() {
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .take(8)
            .map(|z| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(z.zettel_type.symbol(), Style::default().fg(Color::Yellow)),
                        Span::raw(" "),
                        Span::styled(&z.title, Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(Span::styled(
                        z.preview(50),
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
        Line::from("  1-3          Switch view"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  n            New zettel"),
        Line::from("  e            Edit mode"),
        Line::from("  d            Delete"),
        Line::from("  t            Add tag"),
        Line::from("  l            Add link"),
        Line::from("  T            Cycle type"),
        Line::from("  Ctrl+S       Save"),
        Line::from(""),
        Line::from(Span::styled("Zettel Types", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ✎ Fleeting    Quick capture"),
        Line::from("  📖 Literature  From sources"),
        Line::from("  ◆ Permanent   Refined ideas"),
        Line::from("  ◎ Hub         Index notes"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  f            Filter by type"),
        Line::from("  F            Clear filters"),
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
