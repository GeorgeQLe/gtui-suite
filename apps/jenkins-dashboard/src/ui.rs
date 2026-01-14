use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::app::{App, BuildStatus, View};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_view {
        View::Jobs => render_jobs(f, app),
        View::Builds => render_builds(f, app),
        View::Console => render_console(f, app),
    }

    if app.show_help {
        render_help(f);
    }
}

fn status_style(status: &BuildStatus) -> Style {
    match status {
        BuildStatus::Success => Style::default().fg(Color::Green),
        BuildStatus::Failure => Style::default().fg(Color::Red),
        BuildStatus::Unstable => Style::default().fg(Color::Yellow),
        BuildStatus::Building => Style::default().fg(Color::Blue),
        BuildStatus::Aborted => Style::default().fg(Color::Gray),
        BuildStatus::NotBuilt => Style::default().fg(Color::DarkGray),
    }
}

fn health_indicator(health: u8) -> (&'static str, Style) {
    match health {
        80..=100 => ("☀", Style::default().fg(Color::Green)),
        60..=79 => ("⛅", Style::default().fg(Color::Yellow)),
        40..=59 => ("☁", Style::default().fg(Color::Yellow)),
        20..=39 => ("🌧", Style::default().fg(Color::Red)),
        _ => ("⛈", Style::default().fg(Color::Red)),
    }
}

fn render_jobs(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Cell::from("W").style(Style::default().bold()),
        Cell::from("Job").style(Style::default().bold()),
        Cell::from("Folder").style(Style::default().bold()),
        Cell::from("Last Build").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Duration").style(Style::default().bold()),
        Cell::from("Last Run").style(Style::default().bold()),
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

            let (health_icon, health_style) = health_indicator(job.health);

            Row::new(vec![
                Cell::from(health_icon).style(health_style),
                Cell::from(job.name.clone()),
                Cell::from(job.folder.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("#{}", job.last_build)),
                Cell::from(job.status.as_str()).style(status_style(&job.status)),
                Cell::from(job.duration.clone()),
                Cell::from(job.timestamp.clone()),
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
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(18),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Jenkins Dashboard "));

    f.render_widget(table, chunks[0]);

    let help = " q:Quit | j/k:Navigate | Enter:Builds | b:Build | r:Refresh | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[1]);
}

fn render_builds(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Job info
    let job = app.current_job();
    let info = format!(
        " Job: {}/{} | Last Build: #{} | Health: {}%",
        job.folder, job.name, job.last_build, job.health
    );
    let info_bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Job Info "));
    f.render_widget(info_bar, chunks[0]);

    // Builds table
    let header = Row::new(vec![
        Cell::from("Build").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Duration").style(Style::default().bold()),
        Cell::from("Timestamp").style(Style::default().bold()),
        Cell::from("Cause").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .builds
        .iter()
        .enumerate()
        .map(|(i, build)| {
            let style = if i == app.selected_build {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("#{}", build.number)),
                Cell::from(build.status.as_str()).style(status_style(&build.status)),
                Cell::from(build.duration.clone()),
                Cell::from(build.timestamp.clone()),
                Cell::from(build.cause.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(18),
            Constraint::Min(25),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Build History "));

    f.render_widget(table, chunks[1]);

    let help = " Esc:Back | j/k:Navigate | Enter/c:Console | a:Abort | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_console(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Build info
    if !app.builds.is_empty() {
        let build = &app.builds[app.selected_build];
        let job = app.current_job();
        let info = format!(
            " {} #{} | {} | {}",
            job.name,
            build.number,
            build.status.as_str(),
            build.timestamp
        );
        let info_bar = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).title(" Build "));
        f.render_widget(info_bar, chunks[0]);
    }

    // Console output
    let output_text = app.console_output.join("\n");
    let console = Paragraph::new(output_text)
        .block(Block::default().borders(Borders::ALL).title(" Console Output "))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .style(Style::default().fg(Color::White).bg(Color::Black));

    f.render_widget(console, chunks[1]);

    let help = " Esc:Back | j/k:Scroll | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Jenkins Dashboard - Keyboard Shortcuts

Jobs View:
  j/k, Up/Down  - Navigate jobs
  Enter         - View builds
  b             - Trigger build
  r             - Refresh

Builds View:
  j/k, Up/Down  - Navigate builds
  Enter, c      - View console
  a             - Abort build
  Esc           - Back to jobs

Console View:
  j/k           - Scroll output
  Esc           - Back to builds

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
