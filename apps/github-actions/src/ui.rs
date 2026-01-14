use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::app::{App, View, WorkflowStatus};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_view {
        View::Runs => render_runs(f, app),
        View::Jobs => render_jobs(f, app),
        View::Logs => render_logs(f, app),
    }

    if app.show_help {
        render_help(f);
    }
}

fn status_style(status: &WorkflowStatus) -> Style {
    match status {
        WorkflowStatus::Success => Style::default().fg(Color::Green),
        WorkflowStatus::Failure => Style::default().fg(Color::Red),
        WorkflowStatus::InProgress => Style::default().fg(Color::Yellow),
        WorkflowStatus::Queued => Style::default().fg(Color::Blue),
        WorkflowStatus::Cancelled => Style::default().fg(Color::Gray),
    }
}

fn status_icon(status: &WorkflowStatus) -> &str {
    match status {
        WorkflowStatus::Success => "✓",
        WorkflowStatus::Failure => "✗",
        WorkflowStatus::InProgress => "●",
        WorkflowStatus::Queued => "○",
        WorkflowStatus::Cancelled => "⊘",
    }
}

fn render_runs(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Cell::from("").style(Style::default().bold()),
        Cell::from("Workflow").style(Style::default().bold()),
        Cell::from("Branch").style(Style::default().bold()),
        Cell::from("Commit").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Duration").style(Style::default().bold()),
        Cell::from("Actor").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .runs
        .iter()
        .enumerate()
        .map(|(i, run)| {
            let style = if i == app.selected_run {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(status_icon(&run.status)).style(status_style(&run.status)),
                Cell::from(run.workflow.clone()),
                Cell::from(run.branch.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(run.commit.clone()),
                Cell::from(run.status.as_str()).style(status_style(&run.status)),
                Cell::from(run.duration.clone()),
                Cell::from(run.actor.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" GitHub Actions - Workflow Runs "));

    f.render_widget(table, chunks[0]);

    let help = " q:Quit | j/k:Navigate | Enter:Jobs | r:Re-run | c:Cancel | f:Filter | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[1]);
}

fn render_jobs(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Run info
    let run = app.current_run();
    let info = format!(
        " {} #{} | {} | {} | {}",
        run.workflow, run.id, run.branch, run.commit, run.started
    );
    let info_bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Workflow Run "));
    f.render_widget(info_bar, chunks[0]);

    // Jobs table
    let header = Row::new(vec![
        Cell::from("").style(Style::default().bold()),
        Cell::from("Job").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Duration").style(Style::default().bold()),
        Cell::from("Steps").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .jobs
        .iter()
        .enumerate()
        .map(|(i, job)| {
            let style = if i == app.selected_job {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(status_icon(&job.status)).style(status_style(&job.status)),
                Cell::from(job.name.clone()),
                Cell::from(job.status.as_str()).style(status_style(&job.status)),
                Cell::from(job.duration.clone()),
                Cell::from(job.steps.len().to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(25),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Jobs "));

    f.render_widget(table, chunks[1]);

    let help = " Esc:Back | j/k:Navigate | Enter:Logs | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_logs(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Job info
    if let Some(job) = app.current_job() {
        let info = format!(
            " Job: {} | Status: {} | Duration: {}",
            job.name,
            job.status.as_str(),
            job.duration
        );
        let info_bar = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).title(" Job Details "));
        f.render_widget(info_bar, chunks[0]);

        // Steps and logs
        let mut log_lines: Vec<Line> = Vec::new();
        for step in &job.steps {
            let icon = status_icon(&step.status);
            let style = status_style(&step.status);
            log_lines.push(Line::from(vec![
                Span::styled(format!("{} ", icon), style),
                Span::styled(format!("{} ", step.name), Style::default().bold()),
                Span::styled(format!("({})", step.duration), Style::default().fg(Color::Gray)),
            ]));

            // Simulated log output
            log_lines.push(Line::from(Span::styled(
                "  > Setting up job...",
                Style::default().fg(Color::DarkGray),
            )));
            log_lines.push(Line::from(Span::styled(
                "  > Running step...",
                Style::default().fg(Color::DarkGray),
            )));
            log_lines.push(Line::from(Span::styled(
                "  > Step completed",
                Style::default().fg(Color::DarkGray),
            )));
            log_lines.push(Line::from(""));
        }

        let logs = Paragraph::new(log_lines)
            .block(Block::default().borders(Borders::ALL).title(" Steps & Logs "))
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(logs, chunks[1]);
    }

    let help = " Esc:Back | j/k:Scroll | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
GitHub Actions - Keyboard Shortcuts

Workflow Runs:
  j/k, Up/Down  - Navigate runs
  Enter         - View jobs
  r             - Re-run workflow
  c             - Cancel workflow
  f             - Filter by branch

Jobs View:
  j/k, Up/Down  - Navigate jobs
  Enter         - View logs
  Esc           - Back to runs

Logs View:
  j/k           - Scroll logs
  Esc           - Back to jobs

General:
  ?             - Toggle help
  q             - Quit
"#;

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
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
