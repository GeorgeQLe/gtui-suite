//! UI rendering for time tracker.

use crate::app::{App, InputField, View};
use crate::pomodoro::SessionType;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if app.show_help {
        draw_help(f);
    }

    if app.editing {
        draw_input(f, app);
    }

    if let Some(msg) = &app.message {
        draw_message(f, msg);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let date_str = app.selected_date.format("%A, %B %d, %Y").to_string();

    let tabs = vec![
        styled_tab("1:Timer", app.view == View::Timer),
        Span::raw(" "),
        styled_tab("2:Entries", app.view == View::Entries),
        Span::raw(" "),
        styled_tab("r:Reports", app.view == View::Reports),
        Span::raw(" "),
        styled_tab("P:Projects", app.view == View::Projects),
    ];

    let header = Paragraph::new(Line::from(tabs))
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", date_str)))
        .alignment(Alignment::Center);
    f.render_widget(header, area);
}

fn styled_tab(label: &str, active: bool) -> Span {
    if active {
        Span::styled(format!("[{}]", label), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(format!(" {} ", label), Style::default().fg(Color::Gray))
    }
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Timer => draw_timer_view(f, app, area),
        View::Entries => draw_entries_view(f, app, area),
        View::Reports => draw_reports_view(f, app, area),
        View::Projects => draw_projects_view(f, app, area),
    }
}

fn draw_timer_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Timer display
            Constraint::Length(5),  // Pomodoro
            Constraint::Min(0),     // Today's entries
        ])
        .split(area);

    // Timer display
    let timer_block = Block::default().borders(Borders::ALL).title(" Timer ");

    let timer_content = if let Some(entry) = &app.running_entry {
        let duration = entry.format_duration();
        let desc = if entry.description.is_empty() {
            "(no description - press Enter to add)"
        } else {
            &entry.description
        };
        vec![
            Line::from(vec![
                Span::styled(&duration, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(desc),
            Line::from(""),
            Line::from(Span::styled("Press 's' to stop", Style::default().fg(Color::DarkGray))),
        ]
    } else {
        vec![
            Line::from(Span::styled("00:00:00", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from("No timer running"),
            Line::from(""),
            Line::from(Span::styled("Press 's' to start", Style::default().fg(Color::DarkGray))),
        ]
    };

    let timer = Paragraph::new(timer_content)
        .block(timer_block)
        .alignment(Alignment::Center);
    f.render_widget(timer, chunks[0]);

    // Pomodoro section
    let pomo_block = Block::default().borders(Borders::ALL).title(format!(
        " Pomodoro {} ",
        if app.pomodoro_mode { "(active)" } else { "(press 'p')" }
    ));

    if app.pomodoro_mode {
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(chunks[1]);

        let session_name = app.pomodoro.session_type().name();
        let remaining = app.pomodoro.format_remaining();
        let pomodoros = format!("{} pomodoros completed", app.pomodoro.total_pomodoros());

        let info = Paragraph::new(format!("{} - {} | {}", session_name, remaining, pomodoros))
            .alignment(Alignment::Center);
        f.render_widget(pomo_block, chunks[1]);
        f.render_widget(info, inner[0]);

        let gauge = Gauge::default()
            .ratio(app.pomodoro.progress())
            .gauge_style(Style::default().fg(match app.pomodoro.session_type() {
                SessionType::Work => Color::Red,
                SessionType::ShortBreak => Color::Green,
                SessionType::LongBreak => Color::Blue,
            }));
        f.render_widget(gauge, inner[1]);
    } else {
        let pomo = Paragraph::new("Press 'p' to start Pomodoro mode")
            .block(pomo_block)
            .alignment(Alignment::Center);
        f.render_widget(pomo, chunks[1]);
    }

    // Today's summary
    let today_total = app.today_total();
    let title = format!(" Today: {} ", App::format_duration(today_total));

    if app.entries.is_empty() {
        let msg = Paragraph::new("No entries today")
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(Alignment::Center);
        f.render_widget(msg, chunks[2]);
    } else {
        let items: Vec<ListItem> = app.entries.iter().take(5).map(|e| {
            let time = e.start_time.format("%H:%M").to_string();
            let dur = e.format_duration_short();
            let desc = if e.description.is_empty() { "(no description)" } else { &e.description };
            ListItem::new(format!("{} | {} | {}", time, dur, desc))
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(list, chunks[2]);
    }
}

fn draw_entries_view(f: &mut Frame, app: &App, area: Rect) {
    if app.entries.is_empty() {
        let msg = Paragraph::new("No entries for this date")
            .block(Block::default().borders(Borders::ALL).title(" Entries "))
            .alignment(Alignment::Center);
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app.entries.iter().enumerate().map(|(i, e)| {
        let time = format!("{} - {}",
            e.start_time.format("%H:%M"),
            e.end_time.map(|t| t.format("%H:%M").to_string()).unwrap_or_else(|| "running".to_string())
        );
        let dur = e.format_duration_short();
        let desc = if e.description.is_empty() { "(no description)" } else { &e.description };

        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        ListItem::new(format!("{} | {} | {}", time, dur, desc)).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Entries "));
    f.render_widget(list, area);
}

fn draw_reports_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Today summary
    let today_total = app.today_total();
    let summary = format!(
        "Today: {}\nEntries: {}",
        App::format_duration(today_total),
        app.entries.len()
    );

    let today = Paragraph::new(summary)
        .block(Block::default().borders(Borders::ALL).title(" Today "))
        .wrap(Wrap { trim: true });
    f.render_widget(today, chunks[0]);

    // Project breakdown
    if app.projects.is_empty() {
        let msg = Paragraph::new("No projects yet")
            .block(Block::default().borders(Borders::ALL).title(" By Project "))
            .alignment(Alignment::Center);
        f.render_widget(msg, chunks[1]);
    } else {
        let rows: Vec<Row> = app.projects.iter().map(|p| {
            let hours = app.project_hours.get(&p.id).copied().unwrap_or(0.0);
            let budget = p.budget_hours.map(|b| format!("{:.1}h", b)).unwrap_or_else(|| "-".to_string());
            Row::new(vec![
                p.name.clone(),
                format!("{:.1}h", hours),
                budget,
            ])
        }).collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(25), Constraint::Percentage(25)],
        )
        .header(Row::new(vec!["Project", "Hours", "Budget"]).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(Block::default().borders(Borders::ALL).title(" By Project "));

        f.render_widget(table, chunks[1]);
    }
}

fn draw_projects_view(f: &mut Frame, app: &App, area: Rect) {
    if app.projects.is_empty() {
        let msg = Paragraph::new("No projects yet. Press 'a' to add one.")
            .block(Block::default().borders(Borders::ALL).title(" Projects "))
            .alignment(Alignment::Center);
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app.projects.iter().enumerate().map(|(i, p)| {
        let hours = app.project_hours.get(&p.id).copied().unwrap_or(0.0);
        let budget_info = p.budget_hours.map(|b| {
            let percent = (hours / b * 100.0).min(100.0);
            format!(" ({:.0}% of {:.0}h)", percent, b)
        }).unwrap_or_default();

        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        ListItem::new(format!("{} - {:.1}h{}", p.name, hours, budget_info)).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Projects "));
    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.view {
        View::Timer => "s:Start/Stop  p:Pomodoro  Enter:Description  h/l:Date  ?:Help  q:Quit",
        View::Entries => "j/k:Navigate  e:Edit  d:Delete  h/l:Date  ?:Help  q:Quit",
        View::Reports => "h/l:Date  ?:Help  q:Quit",
        View::Projects => "j/k:Navigate  a:Add project  ?:Help  q:Quit",
    };

    let footer = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);

    let help = r#"
Time Tracker Keybindings

Timer:
  s               Start/stop timer
  p               Toggle Pomodoro mode
  Enter           Add/edit description

Navigation:
  j/k, Up/Down    Navigate list
  h/l, Left/Right Change date
  t               Go to today

Views:
  1               Timer view
  2               Entries view
  r               Reports view
  P               Projects view

Actions:
  a               Add project (in Projects view)
  e               Edit entry (in Entries view)
  d               Delete entry (in Entries view)

General:
  ?               Show this help
  q               Quit

Press any key to close
"#;

    let popup = Paragraph::new(help)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });
    f.render_widget(popup, area);
}

fn draw_input(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 15, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_field {
        InputField::Description => "Enter description",
        InputField::ProjectName => "Enter project name",
        InputField::None => "",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)));
    f.render_widget(input, area);

    f.set_cursor_position((area.x + 1 + app.input_buffer.len() as u16, area.y + 1));
}

fn draw_message(f: &mut Frame, msg: &str) {
    let area = Rect::new(
        f.area().x + 2,
        f.area().height.saturating_sub(5),
        f.area().width.saturating_sub(4),
        3,
    );
    f.render_widget(Clear, area);

    let message = Paragraph::new(msg)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(message, area);
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
