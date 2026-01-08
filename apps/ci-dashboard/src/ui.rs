use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, ConfirmAction, InputMode, View};
use crate::models::{Conclusion, RunStatus};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_main(frame, app, chunks[0]);
    render_status(frame, app, chunks[1]);

    // Render overlays
    match app.input_mode {
        InputMode::Search => render_search(frame, app),
        InputMode::Filter => render_filter(frame, app),
        InputMode::Confirm => render_confirm(frame, app),
        _ => {}
    }
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Overview => {
            if app.show_sidebar {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(20), Constraint::Min(50)])
                    .split(area);

                render_sidebar(frame, app, chunks[0]);
                render_runs_list(frame, app, chunks[1]);
            } else {
                render_runs_list(frame, app, area);
            }
        }
        View::RunDetails => {
            render_run_details(frame, app, area);
        }
        View::Logs => {
            render_logs(frame, app, area);
        }
    }
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .system_status
        .iter()
        .map(|s| {
            let status_icon = if s.connected { "●" } else { "○" };
            let color = if s.connected {
                Color::Green
            } else {
                Color::Red
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(color)),
                Span::raw(s.system.as_str()),
            ]))
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Systems "));

    frame.render_widget(list, area);
}

fn render_runs_list(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Repository", "Workflow", "Branch", "Status", "Duration"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let filtered = app.filtered_runs();

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, run)| {
            let style = if i == app.selected_run {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_style = match run.status {
                RunStatus::Completed => match run.conclusion {
                    Some(Conclusion::Success) => Style::default().fg(Color::Green),
                    Some(Conclusion::Failure) => Style::default().fg(Color::Red),
                    Some(Conclusion::Cancelled) => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                },
                RunStatus::InProgress => Style::default().fg(Color::Cyan),
                RunStatus::Queued => Style::default().fg(Color::Gray),
            };

            Row::new(vec![
                Cell::from(run.repo.clone()),
                Cell::from(run.workflow_name.clone()),
                Cell::from(run.branch.clone()),
                Cell::from(run.display_status()).style(status_style),
                Cell::from(run.duration_display()),
            ])
            .style(style)
        })
        .collect();

    let title = if app.filter_status.is_some() || !app.search_query.is_empty() {
        format!(" CI Runs ({} filtered) ", filtered.len())
    } else {
        format!(" CI Runs ({}) ", app.runs.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_run_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(run) = &app.current_run else {
        let empty = Paragraph::new("No run selected")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(5)])
        .split(area);

    // Run info
    let info = vec![
        Line::from(vec![
            Span::styled("Repository: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&run.repo),
        ]),
        Line::from(vec![
            Span::styled("Workflow: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&run.workflow_name),
        ]),
        Line::from(vec![
            Span::styled("Branch: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&run.branch),
        ]),
        Line::from(vec![
            Span::styled("Commit: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&run.commit_sha),
            Span::raw(" - "),
            Span::raw(&run.commit_message),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(run.display_status()),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(run.duration_display()),
        ]),
    ];

    let info_widget =
        Paragraph::new(info).block(Block::default().borders(Borders::ALL).title(" Run Details "));
    frame.render_widget(info_widget, chunks[0]);

    // Jobs list
    let job_items: Vec<ListItem> = run
        .jobs
        .iter()
        .map(|job| {
            let status_icon = match job.status {
                RunStatus::Completed => match job.conclusion {
                    Some(Conclusion::Success) => "✓",
                    Some(Conclusion::Failure) => "✗",
                    _ => "●",
                },
                RunStatus::InProgress => "⟳",
                RunStatus::Queued => "○",
            };

            let color = match job.status {
                RunStatus::Completed => match job.conclusion {
                    Some(Conclusion::Success) => Color::Green,
                    Some(Conclusion::Failure) => Color::Red,
                    _ => Color::White,
                },
                RunStatus::InProgress => Color::Cyan,
                RunStatus::Queued => Color::Gray,
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(color)),
                Span::raw(&job.name),
            ]))
        })
        .collect();

    let jobs_list =
        List::new(job_items).block(Block::default().borders(Borders::ALL).title(" Jobs "));
    frame.render_widget(jobs_list, chunks[1]);
}

fn render_logs(frame: &mut Frame, app: &App, area: Rect) {
    let content = app.logs.as_deref().unwrap_or("No logs available");

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Logs "))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);

    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);

    frame.render_widget(paragraph, area);
}

fn render_search(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(" Search "));

    frame.render_widget(search, area);
}

fn render_filter(frame: &mut Frame, _app: &App) {
    let area = centered_rect(40, 8, frame.area());
    frame.render_widget(Clear, area);

    let options = vec![
        ListItem::new("(s) Success"),
        ListItem::new("(f) Failed"),
        ListItem::new("(c) Cancelled"),
        ListItem::new("(a) All"),
    ];

    let list = List::new(options)
        .block(Block::default().borders(Borders::ALL).title(" Filter by Status "));

    frame.render_widget(list, area);
}

fn render_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 5, frame.area());
    frame.render_widget(Clear, area);

    let message = match &app.confirm_action {
        Some(ConfirmAction::RetryRun(_)) => "Retry this run?",
        Some(ConfirmAction::CancelRun(_)) => "Cancel this run?",
        None => "",
    };

    let content = format!("{}\n\n(y) Yes  (n) No", message);

    let dialog = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Confirm "))
        .alignment(Alignment::Center);

    frame.render_widget(dialog, area);
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
