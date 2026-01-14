use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, EditField, InputMode, View};
use crate::models::CronPreset;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.view {
        View::List => "Jobs",
        View::Create => "New Job",
        View::Edit => "Edit Job",
        View::Presets => "Presets",
        View::Details => "Details",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CRON MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(view_name, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::raw(format!("{} jobs", app.jobs.len())),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Crontab Editor "));

    frame.render_widget(header, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::List => render_list(frame, app, area),
        View::Create | View::Edit => render_editor(frame, app, area),
        View::Presets => render_presets(frame, app, area),
        View::Details => render_details(frame, app, area),
    }
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.jobs.is_empty() {
        let empty = Paragraph::new("No cron jobs. Press 'n' to create one.")
            .block(Block::default().borders(Borders::ALL).title(" Jobs "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let rows: Vec<Row> = app
        .jobs
        .iter()
        .enumerate()
        .map(|(i, job)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let enabled_icon = if job.enabled { "●" } else { "○" };
            let enabled_style = if job.enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(Span::styled(enabled_icon, enabled_style)),
                Cell::from(job.expression.to_string()),
                Cell::from(job.human_description()),
                Cell::from(job.command.clone()),
                Cell::from(job.description.clone().unwrap_or_default()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(15),
            Constraint::Length(25),
            Constraint::Min(20),
            Constraint::Min(15),
        ],
    )
    .header(
        Row::new(vec!["", "Expression", "Schedule", "Command", "Description"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(Block::default().borders(Borders::ALL).title(" Cron Jobs "));

    frame.render_widget(table, area);
}

fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(area);

    let job = app.edit_job.as_ref();

    // Expression field
    let expr_value = job.map(|j| j.expression.to_string()).unwrap_or_default();
    let expr_human = job.map(|j| j.human_description()).unwrap_or_default();

    let expr_content = if app.input_mode == InputMode::EditExpression {
        format!("{}_", app.edit_buffer)
    } else {
        expr_value
    };

    let expr_style = if app.edit_field == EditField::Expression {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let expr_block = Block::default()
        .borders(Borders::ALL)
        .title(" Expression ")
        .border_style(expr_style);

    let expr_para = Paragraph::new(vec![
        Line::from(expr_content),
        Line::from(Span::styled(
            format!("  → {}", expr_human),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(expr_block);

    frame.render_widget(expr_para, chunks[0]);

    // Command field
    let cmd_value = job.map(|j| j.command.clone()).unwrap_or_default();

    let cmd_content = if app.input_mode == InputMode::EditCommand {
        format!("{}_", app.edit_buffer)
    } else {
        cmd_value
    };

    let cmd_style = if app.edit_field == EditField::Command {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let cmd_block = Block::default()
        .borders(Borders::ALL)
        .title(" Command ")
        .border_style(cmd_style);

    let cmd_para = Paragraph::new(cmd_content).block(cmd_block);
    frame.render_widget(cmd_para, chunks[1]);

    // Description field
    let desc_value = job
        .and_then(|j| j.description.clone())
        .unwrap_or_default();

    let desc_content = if app.input_mode == InputMode::EditDescription {
        format!("{}_", app.edit_buffer)
    } else {
        desc_value
    };

    let desc_style = if app.edit_field == EditField::Description {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let desc_block = Block::default()
        .borders(Borders::ALL)
        .title(" Description (optional) ")
        .border_style(desc_style);

    let desc_para = Paragraph::new(desc_content).block(desc_block);
    frame.render_widget(desc_para, chunks[2]);

    // Preview
    if let Some(ref job) = app.edit_job {
        let preview = Paragraph::new(vec![
            Line::from(Span::styled(
                "Preview:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Crontab line: {}", job.to_crontab_line())),
            Line::from(""),
            Line::from(format!("Schedule: {}", job.human_description())),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Preview "));

        frame.render_widget(preview, chunks[3]);
    }
}

fn render_presets(frame: &mut Frame, app: &App, area: Rect) {
    let presets = CronPreset::all();

    let items: Vec<ListItem> = presets
        .iter()
        .enumerate()
        .map(|(i, preset)| {
            let style = if i == app.selected_preset {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(preset.as_str(), Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled(
                    preset.expression(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Common Presets "));

    frame.render_widget(list, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(job) = app.selected_job() else {
        let empty = Paragraph::new("No job selected")
            .block(Block::default().borders(Borders::ALL).title(" Details "));
        frame.render_widget(empty, area);
        return;
    };

    let enabled_str = if job.enabled { "Yes" } else { "No" };
    let enabled_color = if job.enabled { Color::Green } else { Color::Red };

    let next_run = job
        .next_run
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let last_run = job
        .last_run
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Never".to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::raw(job.id.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Expression: ", Style::default().fg(Color::Gray)),
            Span::styled(
                job.expression.to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Schedule: ", Style::default().fg(Color::Gray)),
            Span::raw(job.human_description()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Command: ", Style::default().fg(Color::Gray)),
            Span::styled(&job.command, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Gray)),
            Span::raw(job.description.clone().unwrap_or_else(|| "None".to_string())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enabled: ", Style::default().fg(Color::Gray)),
            Span::styled(enabled_str, Style::default().fg(enabled_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Next Run: ", Style::default().fg(Color::Gray)),
            Span::raw(next_run),
        ]),
        Line::from(vec![
            Span::styled("Last Run: ", Style::default().fg(Color::Gray)),
            Span::raw(last_run),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Job Details "))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();

    let style = if app.validation_error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
}
