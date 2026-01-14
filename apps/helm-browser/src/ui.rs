use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, ReleaseStatus, Tab};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_tabs(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);

    if app.show_help {
        render_help(f);
    }

    if app.show_values {
        render_values(f, app);
    }
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] Releases", "[2] Repositories", "[3] Charts"];
    let selected = match app.current_tab {
        Tab::Releases => 0,
        Tab::Repositories => 1,
        Tab::Charts => 2,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Helm Browser "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bold());

    f.render_widget(tabs, area);
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_tab {
        Tab::Releases => render_releases(f, app, area),
        Tab::Repositories => render_repos(f, app, area),
        Tab::Charts => render_charts(f, app, area),
    }
}

fn render_releases(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().bold()),
        Cell::from("Namespace").style(Style::default().bold()),
        Cell::from("Chart").style(Style::default().bold()),
        Cell::from("Version").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Rev").style(Style::default().bold()),
        Cell::from("Updated").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .releases
        .iter()
        .enumerate()
        .map(|(i, release)| {
            let status_style = match release.status {
                ReleaseStatus::Deployed => Style::default().fg(Color::Green),
                ReleaseStatus::Failed => Style::default().fg(Color::Red),
                ReleaseStatus::Pending => Style::default().fg(Color::Yellow),
                ReleaseStatus::Uninstalling => Style::default().fg(Color::Magenta),
                ReleaseStatus::Superseded => Style::default().fg(Color::Gray),
            };

            let style = if i == app.selected_release {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(release.name.clone()),
                Cell::from(release.namespace.clone()),
                Cell::from(release.chart.clone()),
                Cell::from(release.version.clone()),
                Cell::from(release.status.as_str()).style(status_style),
                Cell::from(release.revision.to_string()),
                Cell::from(release.updated.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Releases "));

    f.render_widget(table, area);
}

fn render_repos(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().bold()),
        Cell::from("URL").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let style = if i == app.selected_repo {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(repo.name.clone()),
                Cell::from(repo.url.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(25), Constraint::Min(50)],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Repositories "));

    f.render_widget(table, area);
}

fn render_charts(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Chart").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .charts
        .iter()
        .enumerate()
        .map(|(i, chart)| {
            let style = if i == app.selected_chart {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![Cell::from(chart.clone())]).style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Min(50)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Available Charts "));

    f.render_widget(table, area);
}

fn render_status_bar(f: &mut Frame, _app: &App, area: Rect) {
    let help = " q:Quit | Tab:Switch | j/k:Navigate | v:Values | u:Upgrade | r:Rollback | d:Delete | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Helm Browser - Keyboard Shortcuts

Navigation:
  j/k, Up/Down  - Navigate list
  Tab           - Switch tabs
  1/2/3         - Jump to tab

Actions:
  v             - View values
  u             - Upgrade release
  r             - Rollback release
  d             - Delete release
  Enter         - Select item

General:
  ?             - Toggle help
  q             - Quit
"#;

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}

fn render_values(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, area);

    let release = &app.releases[app.selected_release];
    let values = format!(
        r#"# Values for {}

replicaCount: 2

image:
  repository: nginx
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 80

resources:
  limits:
    cpu: 100m
    memory: 128Mi
  requests:
    cpu: 50m
    memory: 64Mi

nodeSelector: {{}}

tolerations: []

affinity: {{}}
"#,
        release.name
    );

    let paragraph = Paragraph::new(values)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Values: {} ", release.name)),
        )
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
