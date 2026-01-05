//! UI rendering for daily notes manager.

use crate::app::{App, Mode, Pane};
use chrono::{Datelike, Weekday};
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
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);

    if app.mode == Mode::Search {
        draw_search_dialog(f, app);
    }

    if app.show_help {
        draw_help(f);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let stats_text = format!(
        " Daily Notes | Entries: {} | Words: {} | Streak: {} days ",
        app.stats.total_entries,
        app.stats.total_words,
        app.stats.current_streak,
    );
    let header = Paragraph::new(stats_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(24),
            Constraint::Min(30),
            Constraint::Length(25),
        ])
        .split(area);

    draw_calendar(f, app, chunks[0]);
    draw_editor(f, app, chunks[1]);
    draw_recent_list(f, app, chunks[2]);
}

fn draw_calendar(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.pane == Pane::Calendar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title = format!(" {} {} ", app.calendar.month_name(), app.calendar.year);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Day headers
    let day_headers = "Su Mo Tu We Th Fr Sa";
    let header = Paragraph::new(day_headers)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(header, Rect { height: 1, ..inner });

    // Calendar grid
    let first_weekday = app.calendar.first_weekday();
    let offset = match first_weekday {
        Weekday::Sun => 0,
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
    };

    let mut lines: Vec<Line> = Vec::new();
    let mut current_line: Vec<Span> = Vec::new();

    // Add leading spaces
    for _ in 0..offset {
        current_line.push(Span::raw("   "));
    }

    for (i, day) in app.calendar.days.iter().enumerate() {
        let day_num = day.date.day();
        let is_selected = i == app.selected_day;

        let mut style = Style::default();

        if day.is_today {
            style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
        }

        if day.has_entry {
            style = style.fg(Color::Green);
        }

        if is_selected {
            style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
        }

        let text = format!("{:2} ", day_num);
        current_line.push(Span::styled(text, style));

        let col = (offset + i + 1) % 7;
        if col == 0 {
            lines.push(Line::from(current_line.clone()));
            current_line.clear();
        }
    }

    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    let calendar_text = Paragraph::new(lines);
    let cal_area = Rect {
        y: inner.y + 1,
        height: inner.height.saturating_sub(1),
        ..inner
    };
    f.render_widget(calendar_text, cal_area);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let title = app.current_entry
        .as_ref()
        .map(|e| format!(" {} ", e.formatted_date()))
        .unwrap_or_else(|| " Editor ".to_string());

    let border_style = if app.pane == Pane::Editor || app.mode == Mode::Editing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let mode_indicator = if app.mode == Mode::Editing { " [EDIT] " } else { "" };
    let word_count = app.current_entry
        .as_ref()
        .map(|e| format!(" {} words ", e.word_count))
        .unwrap_or_default();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{}{}{}", title, mode_indicator, word_count))
        .border_style(border_style);

    if app.editor_content.is_empty() || (app.editor_content.len() == 1 && app.editor_content[0].is_empty()) {
        let placeholder = Paragraph::new("Press 'e' to start writing...")
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
                    // Highlight markdown headers
                    if line.starts_with('#') {
                        Line::from(Span::styled(line.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                    } else if line.starts_with("- ") {
                        Line::from(Span::styled(line.as_str(), Style::default().fg(Color::Yellow)))
                    } else {
                        Line::from(line.as_str())
                    }
                }
            })
            .collect();

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

fn draw_recent_list(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.pane == Pane::List {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .recent_entries
        .iter()
        .map(|entry| {
            let date_str = entry.date.format("%b %d").to_string();
            let preview = entry.preview(1);
            let preview = if preview.len() > 15 {
                format!("{}...", &preview[..15])
            } else {
                preview
            };

            let is_current = app.current_entry.as_ref().map(|e| e.id) == Some(entry.id);
            let style = if is_current {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(vec![
                Line::from(Span::styled(date_str, style)),
                Line::from(Span::styled(preview, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Recent ")
            .border_style(border_style));

    f.render_widget(list, area);
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

    let selected_date = app.selected_date()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let info = format!(" {} | {} ", mode_str, selected_date);
    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(info_widget, chunks[0]);

    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | t today | e edit | / search | [ ] month".to_string()
    });
    let msg_widget = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(msg_widget, chunks[1]);
}

fn draw_search_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    // Search input
    let input = Paragraph::new(app.search_query.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Search "))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input, chunks[0]);

    // Results
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .take(10)
        .map(|entry| {
            let date_str = entry.date.format("%Y-%m-%d").to_string();
            let preview = entry.preview(50);
            ListItem::new(vec![
                Line::from(Span::styled(date_str, Style::default().add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(preview, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Results "));
    f.render_widget(list, chunks[1]);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Calendar", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  h/j/k/l      Navigate days"),
        Line::from("  [ / ]        Previous/next month"),
        Line::from("  t            Go to today"),
        Line::from("  Enter        Select day"),
        Line::from(""),
        Line::from(Span::styled("Editor", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  e            Enter edit mode"),
        Line::from("  Ctrl+S       Save"),
        Line::from("  Esc          Exit edit mode"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Tab          Switch pane"),
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
