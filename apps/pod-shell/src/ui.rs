use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, PodStatus, View};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_view {
        View::PodList => render_pod_list(f, app),
        View::ContainerSelect => {
            render_pod_list(f, app);
            render_container_select(f, app);
        }
        View::Shell => render_shell(f, app),
    }

    if app.show_help {
        render_help(f);
    }
}

fn render_pod_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Cell::from("Pod").style(Style::default().bold()),
        Cell::from("Namespace").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
        Cell::from("Containers").style(Style::default().bold()),
        Cell::from("Restarts").style(Style::default().bold()),
        Cell::from("Node").style(Style::default().bold()),
        Cell::from("Age").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .pods
        .iter()
        .enumerate()
        .map(|(i, pod)| {
            let status_style = match pod.status {
                PodStatus::Running => Style::default().fg(Color::Green),
                PodStatus::Pending => Style::default().fg(Color::Yellow),
                PodStatus::Failed => Style::default().fg(Color::Red),
                PodStatus::Succeeded => Style::default().fg(Color::Blue),
                PodStatus::Unknown => Style::default().fg(Color::Gray),
            };

            let style = if i == app.selected_pod {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(pod.name.clone()),
                Cell::from(pod.namespace.clone()),
                Cell::from(pod.status.as_str()).style(status_style),
                Cell::from(pod.containers.len().to_string()),
                Cell::from(pod.restarts.to_string()),
                Cell::from(pod.node.clone()),
                Cell::from(pod.age.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(35),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Length(10),
            Constraint::Length(6),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Pod Shell - Select Pod "));

    f.render_widget(table, chunks[0]);

    let help = " q:Quit | j/k:Navigate | Enter:Shell | l:Logs | d:Describe | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[1]);
}

fn render_container_select(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 40, f.area());
    f.render_widget(Clear, area);

    let containers = app.selected_pod_containers();
    let items: Vec<ListItem> = containers
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i == app.selected_container {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(c.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Container "),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

fn render_shell(f: &mut Frame, app: &App) {
    if let Some(ref session) = app.session {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.area());

        // Output area
        let output_text = session.output.join("\n");
        let output = Paragraph::new(output_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} - {} ", session.pod, session.container)),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(output, chunks[0]);

        // Input area
        let input = Paragraph::new(format!("$ {}", session.input))
            .block(Block::default().borders(Borders::ALL).title(" Command "));

        f.render_widget(input, chunks[1]);
    }
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Pod Shell - Keyboard Shortcuts

Pod List:
  j/k, Up/Down  - Navigate pods
  Enter         - Open shell
  l             - View logs
  d             - Describe pod

Shell:
  Type          - Enter command
  Enter         - Execute command
  Ctrl+D, Esc   - Exit shell

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
