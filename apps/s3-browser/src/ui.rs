use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::app::{format_size, App, View};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_view {
        View::Buckets => render_buckets(f, app),
        View::Objects => render_objects(f, app),
    }

    if app.show_help {
        render_help(f);
    }

    if app.show_properties {
        render_properties(f, app);
    }
}

fn render_buckets(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Cell::from("Bucket").style(Style::default().bold()),
        Cell::from("Region").style(Style::default().bold()),
        Cell::from("Objects").style(Style::default().bold()),
        Cell::from("Size").style(Style::default().bold()),
        Cell::from("Created").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .buckets
        .iter()
        .enumerate()
        .map(|(i, bucket)| {
            let style = if i == app.selected_bucket {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(bucket.name.clone()).style(Style::default().fg(Color::Yellow)),
                Cell::from(bucket.region.clone()),
                Cell::from(bucket.objects.to_string()),
                Cell::from(bucket.size.clone()),
                Cell::from(bucket.created.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(25),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" S3 Browser - Buckets "));

    f.render_widget(table, chunks[0]);

    let help = " q:Quit | j/k:Navigate | Enter:Browse | i:Info | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[1]);
}

fn render_objects(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Path bar
    let path_text = format!(
        " Bucket: {} | Path: {}",
        app.current_bucket_name(),
        app.current_path_display()
    );
    let path_bar = Paragraph::new(path_text)
        .block(Block::default().borders(Borders::ALL).title(" Location "));
    f.render_widget(path_bar, chunks[0]);

    // Objects table
    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().bold()),
        Cell::from("Size").style(Style::default().bold()),
        Cell::from("Last Modified").style(Style::default().bold()),
        Cell::from("Storage Class").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .objects
        .iter()
        .enumerate()
        .map(|(i, obj)| {
            let style = if i == app.selected_object {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let name_style = if obj.is_folder {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };

            let icon = if obj.is_folder { "📁 " } else { "📄 " };

            Row::new(vec![
                Cell::from(format!("{}{}", icon, obj.key)).style(name_style),
                Cell::from(format_size(obj.size)),
                Cell::from(obj.last_modified.clone()),
                Cell::from(obj.storage_class.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(30),
            Constraint::Length(12),
            Constraint::Length(18),
            Constraint::Length(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Objects "));

    f.render_widget(table, chunks[1]);

    let help = " q:Quit | j/k:Nav | Enter:Open | Backspace:Back | d:Download | D:Delete | i:Info | ?:Help ";
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, chunks[2]);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
S3 Browser - Keyboard Shortcuts

Navigation:
  j/k, Up/Down    - Navigate items
  Enter, l, Right - Open bucket/folder
  Backspace, h    - Go back
  Esc             - Exit to buckets

Actions:
  d               - Download file
  D               - Delete file/folder
  u               - Upload file
  i               - Show properties

General:
  ?               - Toggle help
  q               - Quit
"#;

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}

fn render_properties(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 50, f.area());
    f.render_widget(Clear, area);

    let content = match &app.current_view {
        View::Buckets => {
            let bucket = &app.buckets[app.selected_bucket];
            format!(
                r#"
Bucket Properties

Name:     {}
Region:   {}
Created:  {}
Objects:  {}
Size:     {}

Versioning: Enabled
Encryption: AES-256
ACL:        Private
"#,
                bucket.name, bucket.region, bucket.created, bucket.objects, bucket.size
            )
        }
        View::Objects => {
            if app.objects.is_empty() {
                "No object selected".to_string()
            } else {
                let obj = &app.objects[app.selected_object];
                format!(
                    r#"
Object Properties

Key:           {}
Size:          {}
Last Modified: {}
Storage Class: {}
Type:          {}

ETag:     "abc123..."
Metadata: {{}}
"#,
                    obj.key,
                    format_size(obj.size),
                    obj.last_modified,
                    obj.storage_class,
                    if obj.is_folder { "Folder" } else { "File" }
                )
            }
        }
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Properties "))
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
