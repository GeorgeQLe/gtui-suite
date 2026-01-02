//! UI rendering for habit tracker.

use crate::app::{App, EditField, MessageType, View};
use crate::models::HabitStats;
use chrono::{Datelike, Duration, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

/// Draw the application.
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer/status
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    // Draw popups
    if app.show_help {
        draw_help_popup(f);
    }

    if let Some(dialog) = &app.confirm_dialog {
        draw_confirm_dialog(f, dialog);
    }

    if app.editing {
        draw_edit_dialog(f, app);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let date_str = app.selected_date.format("%A, %B %d, %Y").to_string();
    let title = format!(" {} - {} ", app.view_title(), date_str);

    // View tabs
    let tabs: Vec<Span> = vec![
        styled_tab("1:Daily", app.view == View::Daily),
        Span::raw(" "),
        styled_tab("c:Calendar", app.view == View::Calendar),
        Span::raw(" "),
        styled_tab("r:Streaks", app.view == View::Streaks),
        Span::raw(" "),
        styled_tab("s:Stats", app.view == View::Stats),
    ];

    let header = Paragraph::new(Line::from(tabs))
        .block(Block::default().borders(Borders::ALL).title(title))
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

fn styled_tab(label: &str, active: bool) -> Span {
    if active {
        Span::styled(
            format!("[{}]", label),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(format!(" {} ", label), Style::default().fg(Color::Gray))
    }
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Daily => draw_daily_view(f, app, area),
        View::Calendar => draw_calendar_view(f, app, area),
        View::Streaks => draw_streaks_view(f, app, area),
        View::Stats => draw_stats_view(f, app, area),
    }
}

fn draw_daily_view(f: &mut Frame, app: &App, area: Rect) {
    if app.habits.is_empty() {
        let msg = Paragraph::new("No habits due today. Press 'a' to add a habit.")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .habits
        .iter()
        .enumerate()
        .map(|(i, habit)| {
            let entry = app.entries.get(&habit.id);
            let completed = entry.map_or(false, |e| e.completed);

            let checkbox = if completed { "[x]" } else { "[ ]" };

            let mut spans = vec![
                Span::styled(
                    checkbox,
                    Style::default().fg(if completed {
                        Color::Green
                    } else {
                        Color::Gray
                    }),
                ),
                Span::raw(" "),
            ];

            // Habit name
            let name_style = if completed {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            spans.push(Span::styled(&habit.name, name_style));

            // Quantitative progress
            if let Some(goal) = habit.goal() {
                let value = entry.and_then(|e| e.value).unwrap_or(0.0);
                let unit = habit.unit().unwrap_or("");
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("[{:.0}/{:.0} {}]", value, goal, unit),
                    Style::default().fg(Color::Cyan),
                ));
            }

            // Streak info
            if let Some(stats) = app.stats_cache.get(&habit.id) {
                if stats.current_streak > 0 {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        format!("{}d streak", stats.current_streak),
                        Style::default().fg(Color::Magenta),
                    ));
                }
            }

            // Note indicator
            if entry.and_then(|e| e.notes.as_ref()).is_some() {
                spans.push(Span::raw(" "));
                spans.push(Span::styled("*", Style::default().fg(Color::Blue)));
            }

            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    let completion = app.today_completion_rate() * 100.0;
    let title = format!(" Today's Habits ({:.0}% complete) ", completion);

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, area);
}

fn draw_calendar_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(area);

    // Heatmap
    let today = Utc::now().date_naive();
    let weeks = 52;
    let week_width = 3;
    let total_width = weeks * week_width;

    let mut lines: Vec<Line> = Vec::new();

    // Day labels
    let day_labels = vec!["Mon", "   ", "Wed", "   ", "Fri", "   ", "Sun"];

    for (day_idx, label) in day_labels.iter().enumerate() {
        let mut spans = vec![Span::raw(format!("{} ", label))];

        for week in 0..weeks {
            let date = today - Duration::days((weeks - 1 - week) as i64 * 7 + (6 - day_idx as i64));

            if date <= today {
                let rate = app
                    .heatmap
                    .iter()
                    .find(|(d, _)| *d == date)
                    .map(|(_, r)| *r)
                    .unwrap_or(0.0);

                let color = rate_to_color(rate);
                spans.push(Span::styled("  ", Style::default().bg(color)));
                spans.push(Span::raw(" "));
            } else {
                spans.push(Span::raw("   "));
            }
        }

        lines.push(Line::from(spans));
    }

    let heatmap = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Activity "));

    f.render_widget(heatmap, chunks[0]);

    // Legend
    let legend = Paragraph::new(Line::from(vec![
        Span::raw("Less "),
        Span::styled("  ", Style::default().bg(Color::Rgb(22, 27, 34))),
        Span::raw(" "),
        Span::styled("  ", Style::default().bg(Color::Rgb(14, 68, 41))),
        Span::raw(" "),
        Span::styled("  ", Style::default().bg(Color::Rgb(0, 109, 50))),
        Span::raw(" "),
        Span::styled("  ", Style::default().bg(Color::Rgb(38, 166, 65))),
        Span::raw(" "),
        Span::styled("  ", Style::default().bg(Color::Rgb(57, 211, 83))),
        Span::raw(" More"),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    f.render_widget(legend, chunks[1]);
}

fn rate_to_color(rate: f32) -> Color {
    if rate == 0.0 {
        Color::Rgb(22, 27, 34)
    } else if rate < 0.25 {
        Color::Rgb(14, 68, 41)
    } else if rate < 0.5 {
        Color::Rgb(0, 109, 50)
    } else if rate < 0.75 {
        Color::Rgb(38, 166, 65)
    } else {
        Color::Rgb(57, 211, 83)
    }
}

fn draw_streaks_view(f: &mut Frame, app: &App, area: Rect) {
    if app.habits.is_empty() {
        let msg = Paragraph::new("No habits to show streaks for.")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec!["Habit", "Current", "Best", "Rate"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .habits
        .iter()
        .map(|habit| {
            let stats = app
                .stats_cache
                .get(&habit.id)
                .cloned()
                .unwrap_or_default();

            Row::new(vec![
                habit.name.clone(),
                format!("{} days", stats.current_streak),
                format!("{} days", stats.best_streak),
                format!("{:.0}%", stats.completion_percent()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Streaks "));

    f.render_widget(table, area);
}

fn draw_stats_view(f: &mut Frame, app: &App, area: Rect) {
    if app.habits.is_empty() {
        let msg = Paragraph::new("No habits to show statistics for.")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(msg, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    // Overall stats
    let total_habits = app.habits.len();
    let total_completed: u32 = app
        .stats_cache
        .values()
        .map(|s| s.completed_entries)
        .sum();
    let total_entries: u32 = app.stats_cache.values().map(|s| s.total_entries).sum();
    let overall_rate = if total_entries > 0 {
        total_completed as f32 / total_entries as f32 * 100.0
    } else {
        0.0
    };

    let overview = format!(
        "Total Habits: {}\nTotal Entries: {}\nCompleted: {}\nOverall Rate: {:.1}%",
        total_habits, total_entries, total_completed, overall_rate
    );

    let overview_widget = Paragraph::new(overview)
        .block(Block::default().borders(Borders::ALL).title(" Overview "))
        .wrap(Wrap { trim: true });

    f.render_widget(overview_widget, chunks[0]);

    // Per-habit stats
    let items: Vec<ListItem> = app
        .habits
        .iter()
        .enumerate()
        .map(|(i, habit)| {
            let stats = app
                .stats_cache
                .get(&habit.id)
                .cloned()
                .unwrap_or_default();

            let mut spans = vec![
                Span::styled(
                    &habit.name,
                    if i == app.selected_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
                Span::raw(" - "),
                Span::styled(
                    format!("{:.0}%", stats.completion_percent()),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(format!(" ({}/{})", stats.completed_entries, stats.total_entries)),
            ];

            if let Some(avg) = stats.average_value {
                if let Some(unit) = habit.unit() {
                    spans.push(Span::raw(format!(" | Avg: {:.1} {}", avg, unit)));
                }
            }

            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Per-Habit "));

    f.render_widget(list, chunks[1]);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let (msg, style) = if let Some((ref message, msg_type)) = app.message {
        let color = match msg_type {
            MessageType::Info => Color::Blue,
            MessageType::Success => Color::Green,
            MessageType::Warning => Color::Yellow,
            MessageType::Error => Color::Red,
        };
        (message.clone(), Style::default().fg(color))
    } else {
        let help = match app.view {
            View::Daily => "j/k:Navigate  Space:Toggle  a:Add  e:Edit  d:Delete  n:Note  h/l:Date  ?:Help  q:Quit",
            View::Calendar => "h/l:Navigate weeks  t:Today  1:Daily view  ?:Help  q:Quit",
            View::Streaks | View::Stats => "j/k:Navigate  1:Daily view  ?:Help  q:Quit",
        };
        (help.to_string(), Style::default().fg(Color::DarkGray))
    };

    let footer = Paragraph::new(msg)
        .style(style)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Habit Tracker Keybindings

Navigation:
  j/k, Up/Down    Move selection
  h/l, Left/Right Change date
  t               Jump to today
  g/G             Jump to first/last

Views:
  1               Daily view
  c               Calendar view
  r               Streaks view
  s               Statistics view

Actions:
  Space, Enter    Toggle completion
  a               Add new habit
  e               Edit habit
  d               Delete habit
  n               Add note

General:
  ?               Show this help
  q               Quit

Press any key to close
"#;

    let popup = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });

    f.render_widget(popup, area);
}

fn draw_confirm_dialog(f: &mut Frame, dialog: &crate::app::ConfirmDialog) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let text = Paragraph::new(dialog.message.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", dialog.title)),
        )
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}

fn draw_edit_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let title = match app.editing_field {
        EditField::HabitName => "Enter habit name",
        EditField::HabitDescription => "Enter description",
        EditField::Value => "Enter value",
        EditField::Notes => "Enter note",
        EditField::None => "",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", title)),
        )
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(input, area);

    // Show cursor
    f.set_cursor_position((
        area.x + 1 + app.input_buffer.len() as u16,
        area.y + 1,
    ));
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
