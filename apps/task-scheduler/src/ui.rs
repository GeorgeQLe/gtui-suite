//! UI rendering for task scheduler.

use crate::app::{App, InputMode, Pane};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(10), Constraint::Length(3)])
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
    let enabled = app.tasks.iter().filter(|t| t.enabled).count();
    let header = Paragraph::new(format!(" Task Scheduler | {} tasks ({} enabled) ", app.tasks.len(), enabled))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(header, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_tasks(f, app, chunks[0]);
    draw_history(f, app, chunks[1]);
}

fn draw_tasks(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.pane == Pane::Tasks {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app.tasks.iter().enumerate().map(|(i, task)| {
        let status = if task.enabled { "●" } else { "○" };
        let next = task.next_run
            .map(|dt| dt.format("%m/%d %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());

        let mut style = Style::default();
        if i == app.selected_index {
            style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
        }
        if !task.enabled {
            style = style.fg(Color::DarkGray);
        }

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(status, if task.enabled { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) }),
                Span::raw(" "),
                Span::styled(&task.name, style.add_modifier(Modifier::BOLD)),
            ]),
            Line::from(Span::styled(
                format!("  {} | Next: {}", task.schedule.label(), next),
                Style::default().fg(Color::DarkGray),
            )),
        ])
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Tasks ").border_style(border_style));
    f.render_widget(list, area);
}

fn draw_history(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.pane == Pane::History {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title = app.current_task.as_ref()
        .map(|t| format!(" History: {} ", t.name))
        .unwrap_or_else(|| " History ".to_string());

    let items: Vec<ListItem> = app.task_runs.iter().map(|run| {
        let status_color = match run.status {
            crate::models::RunStatus::Success => Color::Green,
            crate::models::RunStatus::Failed => Color::Red,
            crate::models::RunStatus::Running => Color::Yellow,
            _ => Color::DarkGray,
        };

        ListItem::new(Line::from(vec![
            Span::styled(run.status.symbol(), Style::default().fg(status_color)),
            Span::raw(" "),
            Span::raw(run.started_at.format("%m/%d %H:%M").to_string()),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(border_style));
    f.render_widget(list, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let msg = app.message.clone().unwrap_or_else(|| {
        "? help | n new | e enable/disable | r run now | d delete".to_string()
    });
    let status = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
    f.render_widget(status, area);
}

fn draw_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_mode {
        InputMode::TaskName => " Task Name ",
        InputMode::TaskCommand => " Command ",
        InputMode::ScheduleType => " Schedule Type (1=interval, 2=daily) ",
        InputMode::ScheduleValue => " Interval (minutes) ",
        InputMode::None => " Input ",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(50, 50, f.area());
    f.render_widget(Clear, area);

    let help = Paragraph::new(vec![
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k   Move selection"),
        Line::from("  Tab   Switch pane"),
        Line::from("  Enter View history"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  n     New task"),
        Line::from("  e     Enable/disable"),
        Line::from("  r     Run now"),
        Line::from("  d     Delete"),
        Line::from("  q     Quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let v = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)])
        .split(area);
    Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)])
        .split(v[1])[1]
}
