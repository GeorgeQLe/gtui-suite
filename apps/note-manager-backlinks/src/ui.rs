//! UI rendering for backlinks note manager.

use crate::app::{App, InputMode, Mode, Pane};
use crate::models::ViewMode;
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

    draw_tabs(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);

    if app.input_mode != InputMode::None {
        draw_input_dialog(f, app);
    }

    if app.show_help {
        draw_help(f);
    }
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] List", "[2] Backlinks", "[3] Graph"];
    let selected = match app.view {
        ViewMode::List => 0,
        ViewMode::Backlinks => 1,
        ViewMode::Graph => 2,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Notes ({}) | Links ({}) ",
            app.note_count(),
            app.link_count()
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

    draw_note_list(f, app, chunks[0]);
    draw_editor(f, app, chunks[1]);
    draw_links_panel(f, app, chunks[2]);
}

fn draw_note_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .notes
        .iter()
        .enumerate()
        .map(|(i, note)| {
            let mut style = Style::default();
            if i == app.selected_index {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }
            if app.current_note.as_ref().map(|n| n.id) == Some(note.id) {
                style = style.fg(Color::Cyan);
            }

            let preview = note.preview(30);
            ListItem::new(vec![
                Line::from(Span::styled(&note.title, style.add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(preview, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let border_style = if app.pane == Pane::List {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Notes ")
            .border_style(border_style));

    f.render_widget(list, area);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let title = app.current_note
        .as_ref()
        .map(|n| format!(" {} ", n.title))
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
        let placeholder = Paragraph::new("Press 'n' to create a new note")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(placeholder, area);
    } else {
        let inner = block.inner(area);
        f.render_widget(block, area);

        let text: Vec<Line> = app
            .editor_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                // Highlight [[links]]
                let mut spans = Vec::new();
                let mut last_end = 0;

                for mat in regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap().find_iter(line) {
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
                    // Simplified cursor display
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

fn draw_links_panel(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Backlinks
    let backlink_items: Vec<ListItem> = app
        .backlinks
        .iter()
        .map(|note| {
            ListItem::new(vec![
                Line::from(Span::styled(&note.title, Style::default().fg(Color::Green))),
            ])
        })
        .collect();

    let backlinks_list = List::new(backlink_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Backlinks ({}) ", app.backlinks.len())));
    f.render_widget(backlinks_list, chunks[0]);

    // Forward links
    let forward_items: Vec<ListItem> = app
        .forward_links
        .iter()
        .map(|(title, note)| {
            let style = if note.is_some() {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Red) // Broken link
            };
            ListItem::new(Line::from(Span::styled(title, style)))
        })
        .collect();

    let forward_list = List::new(forward_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Links ({}) ", app.forward_links.len())));
    f.render_widget(forward_list, chunks[1]);
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
        " {} | {} ",
        mode_str,
        app.current_note.as_ref().map(|n| n.title.as_str()).unwrap_or("-")
    );
    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(info_widget, chunks[0]);

    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | n new | e edit | / search | [[link]]".to_string()
    });
    let msg_widget = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(msg_widget, chunks[1]);
}

fn draw_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_mode {
        InputMode::NewNote => " New Note ",
        InputMode::Rename => " Rename ",
        InputMode::Search => " Search ",
        InputMode::InsertLink => " Insert Link [[]] ",
        InputMode::None => " Input ",
    };

    if app.input_mode == InputMode::Search && !app.search_results.is_empty() {
        let search_area = Rect { height: area.height.min(12), ..area };
        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .take(8)
            .map(|r| {
                ListItem::new(vec![
                    Line::from(Span::styled(&r.title, Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(Span::styled(&r.snippet, Style::default().fg(Color::DarkGray))),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(list, search_area);
    } else {
        let input = Paragraph::new(app.input_buffer.as_str())
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(input, area);
    }
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k          Move up/down"),
        Line::from("  Tab          Switch pane"),
        Line::from("  1-3          Switch view"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  n            New note"),
        Line::from("  Enter        Open note"),
        Line::from("  e            Edit mode"),
        Line::from("  r            Rename"),
        Line::from("  d            Delete"),
        Line::from("  Ctrl+S       Save"),
        Line::from("  Ctrl+[       Insert link"),
        Line::from(""),
        Line::from(Span::styled("Links", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  [[title]]    Create link to note"),
        Line::from("               (creates note if missing)"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /            Search"),
        Line::from("  Esc          Back"),
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
