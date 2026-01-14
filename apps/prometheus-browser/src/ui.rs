use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, Tab, TargetHealth};

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

    match app.current_tab {
        Tab::Query => render_query(f, app, chunks[1]),
        Tab::Metrics => render_metrics(f, app, chunks[1]),
        Tab::Targets => render_targets(f, app, chunks[1]),
    }

    render_status_bar(f, app, chunks[2]);

    if app.show_help {
        render_help(f);
    }
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] Query", "[2] Metrics", "[3] Targets"];
    let selected = match app.current_tab {
        Tab::Query => 0,
        Tab::Metrics => 1,
        Tab::Targets => 2,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Prometheus Browser "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bold());

    f.render_widget(tabs, area);
}

fn render_query(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Query input
    let input = Paragraph::new(app.query_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(" PromQL Query (Enter to execute) "))
        .style(Style::default().fg(Color::White));
    f.render_widget(input, chunks[0]);

    // Results
    if app.query_results.is_empty() {
        let help_text = r#"
Enter a PromQL query to execute. Examples:

  up
  rate(http_requests_total[5m])
  node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes * 100
  sum(rate(http_requests_total[5m])) by (status)
  histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

Use Up/Down to navigate query history.
"#;
        let paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        f.render_widget(paragraph, chunks[1]);
    } else {
        let header = Row::new(vec![
            Cell::from("Metric").style(Style::default().bold()),
            Cell::from("Labels").style(Style::default().bold()),
            Cell::from("Value").style(Style::default().bold()),
        ])
        .height(1);

        let rows: Vec<Row> = app
            .query_results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == app.selected_result {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let labels = result
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");

                Row::new(vec![
                    Cell::from(result.metric.clone()),
                    Cell::from(format!("{{{}}}", labels)).style(Style::default().fg(Color::Cyan)),
                    Cell::from(format!("{:.4}", result.value)),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(30),
                Constraint::Min(40),
                Constraint::Length(15),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Results "));

        f.render_widget(table, chunks[1]);
    }
}

fn render_metrics(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Metric").style(Style::default().bold()),
        Cell::from("Type").style(Style::default().bold()),
        Cell::from("Help").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .metrics
        .iter()
        .enumerate()
        .map(|(i, metric)| {
            let style = if i == app.selected_metric {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_style = match metric.metric_type {
                crate::app::MetricType::Counter => Style::default().fg(Color::Green),
                crate::app::MetricType::Gauge => Style::default().fg(Color::Blue),
                crate::app::MetricType::Histogram => Style::default().fg(Color::Yellow),
                crate::app::MetricType::Summary => Style::default().fg(Color::Magenta),
                crate::app::MetricType::Unknown => Style::default().fg(Color::Gray),
            };

            Row::new(vec![
                Cell::from(metric.name.clone()),
                Cell::from(metric.metric_type.as_str()).style(type_style),
                Cell::from(metric.help.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(35),
            Constraint::Length(12),
            Constraint::Min(40),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Available Metrics "));

    f.render_widget(table, area);
}

fn render_targets(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Job").style(Style::default().bold()),
        Cell::from("Instance").style(Style::default().bold()),
        Cell::from("Health").style(Style::default().bold()),
        Cell::from("Last Scrape").style(Style::default().bold()),
        Cell::from("Duration").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .targets
        .iter()
        .enumerate()
        .map(|(i, target)| {
            let style = if i == app.selected_target {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let health_style = match target.health {
                TargetHealth::Up => Style::default().fg(Color::Green),
                TargetHealth::Down => Style::default().fg(Color::Red),
                TargetHealth::Unknown => Style::default().fg(Color::Yellow),
            };

            Row::new(vec![
                Cell::from(target.job.clone()),
                Cell::from(target.instance.clone()),
                Cell::from(target.health.as_str()).style(health_style),
                Cell::from(target.last_scrape.clone()),
                Cell::from(target.scrape_duration.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Scrape Targets "));

    f.render_widget(table, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.current_tab {
        Tab::Query => " Ctrl+Q:Quit | Tab:Switch | Enter:Execute | Up/Down:History/Results | Esc:Clear | ?:Help ",
        Tab::Metrics => " q:Quit | Tab:Switch | j/k:Navigate | Enter:Use in query | ?:Help ",
        Tab::Targets => " q:Quit | Tab:Switch | j/k:Navigate | ?:Help ",
    };
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(65, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Prometheus Browser - Keyboard Shortcuts

Navigation:
  Tab           - Switch tabs
  1/2/3         - Jump to tab

Query Tab:
  Type          - Enter query
  Enter         - Execute query
  Up/Down       - Navigate history/results
  Esc           - Clear query/results
  Ctrl+Q        - Quit

Metrics Tab:
  j/k, Up/Down  - Navigate metrics
  Enter         - Use metric in query
  q             - Quit

Targets Tab:
  j/k, Up/Down  - Navigate targets
  q             - Quit

General:
  ?             - Toggle help
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
