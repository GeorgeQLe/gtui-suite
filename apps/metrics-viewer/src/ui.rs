use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, InputMode, View};
use crate::models::{AlertState, PanelType, Sparkline};

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
        InputMode::QueryEdit => render_query_editor(frame, app),
        InputMode::TimeRangeSelect => render_time_range_select(frame, app),
        InputMode::DashboardSelect => render_dashboard_select(frame, app),
        _ => {}
    }
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Query => render_query_view(frame, app, area),
        View::Dashboard => render_dashboard_view(frame, app, area),
        View::Alerts => render_alerts_view(frame, app, area),
        View::MetricBrowser => render_metric_browser(frame, app, area),
    }
}

fn render_query_view(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    // Query input
    let query_display = if app.query.is_empty() {
        "Enter PromQL query (press 'e' to edit)".to_string()
    } else {
        app.query.clone()
    };

    let query_widget = Paragraph::new(query_display)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Query [{}] ",
            app.time_range.display()
        )));
    frame.render_widget(query_widget, chunks[0]);

    // Results
    if app.results.is_empty() {
        let empty = Paragraph::new("No results. Execute a query to see metrics.")
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(empty, chunks[1]);
    } else {
        let result_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Sparkline graph area
        render_sparklines(frame, app, result_chunks[0]);

        // Results table
        render_results_table(frame, app, result_chunks[1]);
    }
}

fn render_sparklines(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Block::default()
        .borders(Borders::ALL)
        .title(" Graph ")
        .inner(area);

    frame.render_widget(Block::default().borders(Borders::ALL).title(" Graph "), area);

    if app.results.is_empty() {
        return;
    }

    // Render sparklines for each result
    let height = inner.height as usize;
    let width = inner.width as usize;

    if height == 0 || width == 0 {
        return;
    }

    let lines: Vec<Line> = app
        .results
        .iter()
        .take(height)
        .map(|result| {
            let sparkline = Sparkline::from_values(&result.values);
            let graph = sparkline.render(width.saturating_sub(20));

            let label = result
                .metric
                .get("instance")
                .cloned()
                .unwrap_or_else(|| "series".to_string());

            let value = result
                .latest_value()
                .map(|v| format!("{:.2}", v))
                .unwrap_or_else(|| "-".to_string());

            Line::from(vec![
                Span::styled(
                    format!("{:>12}: ", label),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(graph),
                Span::styled(format!(" {}", value), Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_results_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Series", "Labels", "Current Value"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let style = if i == app.selected_result {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let name = result
                .metric
                .get("__name__")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let labels: String = result
                .metric
                .iter()
                .filter(|(k, _)| *k != "__name__")
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");

            let value = result
                .latest_value()
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(name),
                Cell::from(labels),
                Cell::from(value),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Results ({}) ", app.results.len())),
    );

    frame.render_widget(table, area);
}

fn render_dashboard_view(frame: &mut Frame, app: &App, area: Rect) {
    let Some(dashboard) = app.current_dashboard() else {
        let empty = Paragraph::new("No dashboard selected. Press 'd' to select one.")
            .block(Block::default().borders(Borders::ALL).title(" Dashboard "));
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            dashboard
                .panels
                .iter()
                .map(|_| Constraint::Ratio(1, dashboard.panels.len() as u32))
                .collect::<Vec<_>>(),
        )
        .split(area);

    for (i, panel) in dashboard.panels.iter().enumerate() {
        let style = if i == app.selected_panel {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let content = match panel.panel_type {
            PanelType::Graph => {
                // Demo sparkline
                let data: Vec<f64> = (0..50).map(|i| 50.0 + 30.0 * (i as f64 * 0.1).sin()).collect();
                let sparkline = Sparkline {
                    data: data.clone(),
                    min: 20.0,
                    max: 80.0,
                };
                sparkline.render(40)
            }
            PanelType::Stat => "42.5%".to_string(),
            PanelType::Gauge => {
                let pct = 65;
                let filled = pct / 5;
                let bar: String = (0..20)
                    .map(|i| if i < filled { '█' } else { '░' })
                    .collect();
                format!("{} {}%", bar, pct)
            }
            PanelType::Table => "Data table".to_string(),
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ({}) ", panel.title, panel.panel_type.as_str()))
                    .border_style(style),
            )
            .alignment(if panel.panel_type == PanelType::Stat {
                Alignment::Center
            } else {
                Alignment::Left
            });

        frame.render_widget(paragraph, chunks[i]);
    }
}

fn render_alerts_view(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["State", "Name", "Severity", "Summary"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .alerts
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let style = if i == app.selected_alert {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let state_style = match alert.state {
                AlertState::Firing => Style::default().fg(Color::Red),
                AlertState::Pending => Style::default().fg(Color::Yellow),
                AlertState::Resolved => Style::default().fg(Color::Green),
            };

            let summary = alert
                .annotations
                .get("summary")
                .cloned()
                .unwrap_or_default();

            Row::new(vec![
                Cell::from(format!("{} {}", alert.state.icon(), alert.state.as_str()))
                    .style(state_style),
                Cell::from(alert.name.clone()),
                Cell::from(alert.severity.clone()),
                Cell::from(summary),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(50),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Alerts ({}) ", app.alerts.len())),
    );

    frame.render_widget(table, area);
}

fn render_metric_browser(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_metrics();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, metric)| {
            let style = if i == app.selected_metric {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(metric.as_str()).style(style)
        })
        .collect();

    let title = if app.metric_filter.is_empty() {
        format!(" Metrics ({}) ", filtered.len())
    } else {
        format!(" Metrics ({}) - filter: {} ", filtered.len(), app.metric_filter)
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();

    let auto_refresh = if app.auto_refresh { " [AUTO]" } else { "" };

    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {}{} ", status, auto_refresh)).style(style);

    frame.render_widget(paragraph, area);
}

fn render_query_editor(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 20, frame.area());
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .margin(1)
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Edit Query (Enter to execute, Esc to cancel) ");
    frame.render_widget(block, area);

    // Query input
    let cursor = if app.query_cursor <= app.query.len() {
        format!(
            "{}█{}",
            &app.query[..app.query_cursor],
            &app.query[app.query_cursor..]
        )
    } else {
        format!("{}█", app.query)
    };

    let query = Paragraph::new(cursor).block(Block::default().borders(Borders::ALL).title(" Query "));
    frame.render_widget(query, chunks[0]);

    // History
    let history: Vec<ListItem> = app
        .query_history
        .iter()
        .rev()
        .take(5)
        .map(|q| ListItem::new(q.as_str()))
        .collect();

    let history_list =
        List::new(history).block(Block::default().borders(Borders::ALL).title(" History (Up/Down) "));
    frame.render_widget(history_list, chunks[1]);
}

fn render_time_range_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(30, 40, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .time_range_presets
        .iter()
        .enumerate()
        .map(|(i, preset)| {
            let style = if i == app.selected_preset {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(format!("Last {}", preset.label)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Time Range "));

    frame.render_widget(list, area);
}

fn render_dashboard_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .dashboards
        .iter()
        .enumerate()
        .map(|(i, dashboard)| {
            let selected = app.active_dashboard == Some(i);
            let style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let prefix = if selected { "● " } else { "  " };
            ListItem::new(format!("{}{} ({} panels)", prefix, dashboard.name, dashboard.panels.len()))
                .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Dashboard "));

    frame.render_widget(list, area);
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
