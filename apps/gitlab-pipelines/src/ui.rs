use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::app::{App, PipelineStatus, View};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_view {
        View::Pipelines => render_pipelines(f, app),
        View::Stages => render_stages(f, app),
        View::Jobs => render_jobs(f, app),
    }

    if app.show_help {
        render_help(f);
    }
}

fn status_style(status: &PipelineStatus) -> Style {
    match status {
        PipelineStatus::Success => Style::default().fg(Color::Green),
        PipelineStatus::Failed => Style::default().fg(Color::Red),
        PipelineStatus::Running => Style::default().fg(Color::Yellow),
        PipelineStatus::Pending => Style::default().fg(Color::Blue),
        PipelineStatus::Cancelled => Style::default().fg(Color::Gray),
        PipelineStatus::Skipped => Style::default().fg(Color::DarkGray),
    }
}

fn status_icon(status: &PipelineStatus) -> &str {
    match status {
        PipelineStatus::Success => "✓",
        PipelineStatus::Failed => "✗",
        PipelineStatus::Running => "●",
        PipelineStatus::Pending => "○",
        PipelineStatus::Cancelled => "⊘",
        PipelineStatus::Skipped => "⊖",
    }
}

fn render_pipelines(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Cell::from("").style(Style::default().bold()),
        Cell::from("Pipeline").style(Style::default().bold()),
        Cell::from("Branch").style(Style::default().bold()),
        Cell::from("Commit").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Duration").style(Style::default().bold()),
        Cell::from("Author").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .pipelines
        .iter()
        .enumerate()
        .map(|(i, pipeline)| {
            let style = if i == app.selected_pipeline {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(status_icon(&pipeline.status)).style(status_style(&pipeline.status)),
                Cell::from(format!("#{}", pipeline.id)),
                Cell::from(pipeline.branch.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(pipeline.commit.clone()),
                Cell::from(pipeline.status.as_str()).style(status_style(&pipeline.status)),
                Cell::from(pipeline.duration.clone()),
                Cell::from(pipeline.author.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(18),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" GitLab Pipelines "));

    f.render_widget(table, chunks[0]);

    let help = " q:Quit | j/k:Navigate | Enter:Stages | r:Retry | c:Cancel | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[1]);
}

fn render_stages(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Pipeline info
    let pipeline = app.current_pipeline();
    let info = format!(
        " Pipeline #{} | {} | {} | {}",
        pipeline.id, pipeline.branch, pipeline.commit_msg, pipeline.created
    );
    let info_bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Pipeline "));
    f.render_widget(info_bar, chunks[0]);

    // Stages table
    let header = Row::new(vec![
        Cell::from("").style(Style::default().bold()),
        Cell::from("Stage").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Jobs").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = pipeline
        .stages
        .iter()
        .enumerate()
        .map(|(i, stage)| {
            let style = if i == app.selected_stage {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(status_icon(&stage.status)).style(status_style(&stage.status)),
                Cell::from(stage.name.clone()),
                Cell::from(stage.status.as_str()).style(status_style(&stage.status)),
                Cell::from(stage.jobs.len().to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Stages "));

    f.render_widget(table, chunks[1]);

    let help = " Esc:Back | j/k:Navigate | Enter:Jobs | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
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

    // Stage info
    let stage = app.current_stage();
    let info = format!(
        " Stage: {} | Status: {} | Jobs: {}",
        stage.name,
        stage.status.as_str(),
        stage.jobs.len()
    );
    let info_bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Stage "));
    f.render_widget(info_bar, chunks[0]);

    // Jobs table
    let header = Row::new(vec![
        Cell::from("").style(Style::default().bold()),
        Cell::from("Job").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Duration").style(Style::default().bold()),
        Cell::from("Runner").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = stage
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
                Cell::from(job.runner.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Jobs "));

    f.render_widget(table, chunks[1]);

    let help = " Esc:Back | j/k:Navigate | l:Logs | r:Retry | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
GitLab Pipelines - Keyboard Shortcuts

Pipelines View:
  j/k, Up/Down  - Navigate pipelines
  Enter         - View stages
  r             - Retry pipeline
  c             - Cancel pipeline

Stages View:
  j/k, Up/Down  - Navigate stages
  Enter         - View jobs
  Esc           - Back to pipelines

Jobs View:
  j/k, Up/Down  - Navigate jobs
  l             - View logs
  r             - Retry job
  Esc           - Back to stages

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
