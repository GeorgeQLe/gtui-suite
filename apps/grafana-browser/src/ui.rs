use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::app::{App, View};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_view {
        View::Folders => render_folders(f, app),
        View::Dashboards => render_dashboards(f, app),
        View::Panels => render_panels(f, app),
    }

    if app.show_help {
        render_help(f);
    }
}

fn render_folders(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Cell::from("Folder").style(Style::default().bold()),
        Cell::from("Dashboards").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .folders
        .iter()
        .enumerate()
        .map(|(i, folder)| {
            let style = if i == app.selected_folder {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("📁 {}", folder.title)).style(Style::default().fg(Color::Yellow)),
                Cell::from(folder.dashboard_count.to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Min(30), Constraint::Length(12)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Grafana Browser - Folders "));

    f.render_widget(table, chunks[0]);

    let help = " q:Quit | j/k:Navigate | Enter:Open | s:Starred only | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[1]);
}

fn render_dashboards(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Folder info
    let starred_text = if app.show_starred_only {
        " (starred only)"
    } else {
        ""
    };
    let info = format!(" Folder: {}{}", app.current_folder_name(), starred_text);
    let info_bar = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Location "));
    f.render_widget(info_bar, chunks[0]);

    // Dashboards table
    let header = Row::new(vec![
        Cell::from("").style(Style::default().bold()),
        Cell::from("Dashboard").style(Style::default().bold()),
        Cell::from("Tags").style(Style::default().bold()),
        Cell::from("Panels").style(Style::default().bold()),
    ])
    .height(1);

    let filtered = app.filtered_dashboards();
    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, dashboard)| {
            let style = if i == app.selected_dashboard {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let star = if dashboard.starred { "★" } else { "☆" };
            let star_style = if dashboard.starred {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            Row::new(vec![
                Cell::from(star).style(star_style),
                Cell::from(dashboard.title.clone()),
                Cell::from(dashboard.tags.join(", ")).style(Style::default().fg(Color::Cyan)),
                Cell::from(dashboard.panels.len().to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(25),
            Constraint::Length(25),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Dashboards "));

    f.render_widget(table, chunks[1]);

    let help = " Esc:Back | j/k:Nav | Enter:Panels | *:Star | s:Filter starred | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_panels(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Dashboard info
    if let Some(dashboard) = app.current_dashboard() {
        let info = format!(
            " Dashboard: {} | Panels: {}",
            dashboard.title,
            dashboard.panels.len()
        );
        let info_bar = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).title(" Dashboard "));
        f.render_widget(info_bar, chunks[0]);

        // Panels table
        let header = Row::new(vec![
            Cell::from("ID").style(Style::default().bold()),
            Cell::from("Title").style(Style::default().bold()),
            Cell::from("Type").style(Style::default().bold()),
            Cell::from("Datasource").style(Style::default().bold()),
        ])
        .height(1);

        let rows: Vec<Row> = dashboard
            .panels
            .iter()
            .enumerate()
            .map(|(i, panel)| {
                let style = if i == app.selected_panel {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let type_icon = match panel.panel_type {
                    crate::app::PanelType::Graph => "📈",
                    crate::app::PanelType::Stat => "🔢",
                    crate::app::PanelType::Gauge => "⏱",
                    crate::app::PanelType::Table => "📋",
                    crate::app::PanelType::Text => "📝",
                    crate::app::PanelType::Heatmap => "🗺",
                    crate::app::PanelType::Logs => "📜",
                };

                Row::new(vec![
                    Cell::from(panel.id.to_string()),
                    Cell::from(panel.title.clone()),
                    Cell::from(format!("{} {}", type_icon, panel.panel_type.as_str())),
                    Cell::from(panel.datasource.clone()),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Min(25),
                Constraint::Length(15),
                Constraint::Length(15),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Panels "));

        f.render_widget(table, chunks[1]);
    }

    let help = " Esc:Back | j/k:Navigate | Enter:View | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Grafana Browser - Keyboard Shortcuts

Folders View:
  j/k, Up/Down  - Navigate folders
  Enter         - Browse dashboards
  s             - Show starred only

Dashboards View:
  j/k, Up/Down  - Navigate dashboards
  Enter         - View panels
  *             - Toggle star
  s             - Filter starred
  Esc           - Back to folders

Panels View:
  j/k, Up/Down  - Navigate panels
  Enter         - Open panel
  Esc           - Back to dashboards

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
