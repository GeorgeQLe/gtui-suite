//! UI rendering for config editor.

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

    if app.show_quit_confirm {
        draw_quit_confirm(f);
    }

    if app.show_help {
        draw_help(f);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let modified_indicator = if app.modified { " [+]" } else { "" };
    let header_text = format!(
        " {} - {} {}",
        app.file_name(),
        app.format.name(),
        modified_indicator
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(header, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    if app.config.editor.tree_view && app.tree.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        draw_tree(f, app, chunks[0]);
        draw_editor(f, app, chunks[1]);
    } else {
        draw_editor(f, app, area);
    }
}

fn draw_tree(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.pane == Pane::Tree {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .tree_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let indent = "  ".repeat(item.depth.saturating_sub(1));
            let icon = if item.is_container {
                if item.expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            let mut style = Style::default();
            if i == app.selected_tree_index {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }

            let key_style = Style::default().fg(Color::Yellow);
            let value_style = if item.is_container {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::raw(indent),
                Span::raw(icon),
                Span::styled(&item.key, key_style),
                Span::raw(": "),
                Span::styled(&item.value_display, value_style),
            ])).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Structure ")
            .border_style(border_style));

    f.render_widget(list, area);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.pane == Pane::Editor || app.mode == Mode::Editing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let mode_indicator = if app.mode == Mode::Editing { " [EDIT] " } else { "" };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Editor{}", mode_indicator))
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Line numbers + content
    let line_num_width = app.content.len().to_string().len().max(3);

    let text: Vec<Line> = app
        .content
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = format!("{:>width$} ", i + 1, width = line_num_width);

            let content_spans = if app.mode == Mode::Editing && i == app.cursor.0 {
                let col = app.cursor.1;
                let mut spans = vec![
                    Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                ];

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
                spans
            } else {
                // Syntax highlighting based on format
                let mut spans = vec![
                    Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                ];

                // Basic highlighting for keys and values
                if let Some(eq_pos) = line.find('=').or_else(|| line.find(':')) {
                    let key_part = &line[..eq_pos];
                    let sep = &line[eq_pos..eq_pos+1];
                    let val_part = &line[eq_pos+1..];

                    spans.push(Span::styled(key_part.trim_start(), Style::default().fg(Color::Yellow)));
                    spans.push(Span::raw(sep));
                    spans.push(Span::styled(val_part, Style::default().fg(Color::White)));
                } else if line.trim().starts_with('#') || line.trim().starts_with("//") {
                    spans.push(Span::styled(line.as_str(), Style::default().fg(Color::DarkGray)));
                } else if line.trim().starts_with('[') {
                    spans.push(Span::styled(line.as_str(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
                } else {
                    spans.push(Span::raw(line.as_str()));
                }

                spans
            };

            Line::from(content_spans)
        })
        .collect();

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, inner);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Validation status
    let validation_text = if let Some(ref err) = app.validation_error {
        Span::styled(format!("Error: {}", err), Style::default().fg(Color::Red))
    } else {
        Span::styled("Valid", Style::default().fg(Color::Green))
    };

    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Editing => "EDITING",
    };

    let info = Line::from(vec![
        Span::raw(format!(" {} | ", mode_str)),
        Span::raw(format!("Ln {}, Col {} | ", app.cursor.0 + 1, app.cursor.1 + 1)),
        validation_text,
    ]);
    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(info_widget, chunks[0]);

    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | e edit | o open | Ctrl+S save".to_string()
    });
    let msg_widget = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(msg_widget, chunks[1]);
}

fn draw_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 15, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_mode {
        InputMode::OpenFile => " Open File ",
        InputMode::SaveAs => " Save As ",
        InputMode::None => " Input ",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input, area);
}

fn draw_quit_confirm(f: &mut Frame) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let text = vec![
        Line::from(""),
        Line::from("Unsaved changes will be lost."),
        Line::from(""),
        Line::from("Quit anyway? (y/n)"),
    ];

    let dialog = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Confirm Quit "))
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: false });
    f.render_widget(dialog, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k          Move up/down"),
        Line::from("  Tab          Switch pane"),
        Line::from("  Enter        Expand/collapse (tree)"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  e            Edit mode"),
        Line::from("  o            Open file"),
        Line::from("  Ctrl+S       Save"),
        Line::from("  Ctrl+Shift+S Save as"),
        Line::from("  r            Refresh/reparse"),
        Line::from("  Esc          Exit edit mode"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
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
