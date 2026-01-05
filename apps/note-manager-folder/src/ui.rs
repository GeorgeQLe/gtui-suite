//! UI rendering for note manager.

use crate::app::{App, InputMode, Mode, Pane};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
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
    let title = format!(
        " Notes: {} | Folders: {} ",
        app.storage.note_count(),
        app.storage.folder_count()
    );
    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(header, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    draw_tree(f, app, chunks[0]);
    draw_editor(f, app, chunks[1]);
}

fn draw_tree(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .tree_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let indent = "  ".repeat(item.depth.saturating_sub(1));
            let icon = if item.is_folder {
                if item.expanded { "📂" } else { "📁" }
            } else {
                "📄"
            };

            let mut style = Style::default();
            if i == app.selected_index {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }

            if item.is_folder {
                style = style.fg(Color::Yellow);
            }

            ListItem::new(format!("{}{} {}", indent, icon, item.name)).style(style)
        })
        .collect();

    let border_style = if app.pane == Pane::Tree && app.mode == Mode::Normal {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Files ")
            .border_style(border_style));

    f.render_widget(list, area);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let title = app.current_note_title()
        .map(|t| format!(" {} ", t))
        .unwrap_or_else(|| " Editor ".to_string());

    let border_style = if app.pane == Pane::Editor || app.mode == Mode::Editing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let mode_indicator = match app.mode {
        Mode::Editing => " [EDIT] ",
        Mode::Normal => "",
        Mode::Search => " [SEARCH] ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{}{}", title, mode_indicator))
        .border_style(border_style);

    if app.editor_content.is_empty() || (app.editor_content.len() == 1 && app.editor_content[0].is_empty()) {
        let placeholder = Paragraph::new("Press 'n' to create a new note or select one from the tree")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(placeholder, area);
    } else {
        let inner = block.inner(area);
        f.render_widget(block, area);

        // Simple text rendering with cursor
        let text: Vec<Line> = app
            .editor_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if app.mode == Mode::Editing && i == app.editor_cursor.0 {
                    // Show cursor in editing mode
                    let mut spans = Vec::new();
                    let col = app.editor_cursor.1;

                    if col < line.len() {
                        spans.push(Span::raw(&line[..col]));
                        spans.push(Span::styled(
                            &line[col..col+1],
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));
                        spans.push(Span::raw(&line[col+1..]));
                    } else {
                        spans.push(Span::raw(line.as_str()));
                        spans.push(Span::styled(
                            " ",
                            Style::default().bg(Color::White),
                        ));
                    }
                    Line::from(spans)
                } else {
                    Line::from(line.as_str())
                }
            })
            .collect();

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Mode/path info
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Editing => "EDITING",
        Mode::Search => "SEARCH",
    };

    let info = format!(
        " {} | {} ",
        mode_str,
        app.selected_item()
            .map(|i| i.name.as_str())
            .unwrap_or("-")
    );
    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(info_widget, chunks[0]);

    // Message or keybinds
    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | n new | e edit | / search".to_string()
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
        InputMode::NewFolder => " New Folder ",
        InputMode::Rename => " Rename ",
        InputMode::Search => " Search ",
        InputMode::None => " Input ",
    };

    if app.input_mode == InputMode::Search && !app.search_results.is_empty() {
        // Show search results
        let search_area = Rect {
            height: area.height.min(10),
            ..area
        };

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
        Line::from("  h/l          Collapse/expand or parent/child"),
        Line::from("  Enter        Open note or toggle folder"),
        Line::from("  Tab          Switch pane"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  n            New note"),
        Line::from("  N            New folder"),
        Line::from("  r            Rename"),
        Line::from("  d            Delete"),
        Line::from("  e            Edit mode"),
        Line::from("  Ctrl+S       Save"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /            Search"),
        Line::from("  R            Refresh"),
        Line::from("  Esc          Back to normal"),
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
