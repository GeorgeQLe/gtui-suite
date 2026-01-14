use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::app::{App, View};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_view {
        View::Workspaces => render_workspaces(f, app),
        View::Resources => render_resources(f, app),
        View::Details => {
            render_resources(f, app);
            render_details(f, app);
        }
    }

    if app.show_help {
        render_help(f);
    }
}

fn render_workspaces(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Cell::from("Workspace").style(Style::default().bold()),
        Cell::from("Backend").style(Style::default().bold()),
        Cell::from("Resources").style(Style::default().bold()),
        Cell::from("Last Modified").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .workspaces
        .iter()
        .enumerate()
        .map(|(i, ws)| {
            let style = if i == app.selected_workspace {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(ws.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(ws.backend.clone()),
                Cell::from(ws.resources.to_string()),
                Cell::from(ws.last_modified.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(35),
            Constraint::Length(10),
            Constraint::Length(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Terraform State - Workspaces "));

    f.render_widget(table, chunks[0]);

    let help = " q:Quit | j/k:Navigate | Enter:Browse | r:Refresh | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[1]);
}

fn render_resources(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Workspace info
    let ws = &app.workspaces[app.selected_workspace];
    let info = format!(
        " Workspace: {} | Backend: {} | Resources: {}",
        ws.name, ws.backend, ws.resources
    );
    let info_bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" State Info "));
    f.render_widget(info_bar, chunks[0]);

    // Resources table
    let header = Row::new(vec![
        Cell::from("Address").style(Style::default().bold()),
        Cell::from("Type").style(Style::default().bold()),
        Cell::from("Mode").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .resources
        .iter()
        .enumerate()
        .map(|(i, res)| {
            let style = if i == app.selected_resource {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status = if res.tainted {
                Cell::from("TAINTED").style(Style::default().fg(Color::Red))
            } else {
                Cell::from("OK").style(Style::default().fg(Color::Green))
            };

            Row::new(vec![
                Cell::from(res.address.clone()),
                Cell::from(res.resource_type.clone()),
                Cell::from(res.mode.clone()),
                status,
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(30),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Resources "));

    f.render_widget(table, chunks[1]);

    let help = " Esc:Back | j/k:Nav | Enter:Details | t:Taint | d:Destroy | m:Move | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_details(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, area);

    if app.resources.is_empty() {
        return;
    }

    let res = &app.resources[app.selected_resource];
    let details = format!(
        r#"
Resource Details

Address:  {}
Type:     {}
Provider: {}
Mode:     {}
Tainted:  {}

Attributes:
  id        = "i-0abc123def456789"
  arn       = "arn:aws:ec2:us-east-1:123456789:instance/i-0abc123"
  ami       = "ami-0123456789abcdef0"
  instance_type = "t3.medium"

  tags = {{
    Name        = "web-server"
    Environment = "{}"
  }}

Dependencies:
  - aws_vpc.main
  - aws_subnet.public[0]
  - aws_security_group.allow_http
"#,
        res.address,
        res.resource_type,
        res.provider,
        res.mode,
        res.tainted,
        app.current_workspace_name()
    );

    let paragraph = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", res.address)),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Terraform State - Keyboard Shortcuts

Workspaces View:
  j/k, Up/Down  - Navigate workspaces
  Enter         - Browse resources
  r             - Refresh state

Resources View:
  j/k, Up/Down  - Navigate resources
  Enter         - View details
  t             - Toggle taint
  d             - Destroy resource
  m             - Move resource
  Esc           - Back to workspaces

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
