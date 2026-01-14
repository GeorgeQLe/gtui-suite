use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view {
        View::List => render_list(frame, app, chunks[1]),
        View::Detail => render_detail(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "MOUNT MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} mount points", app.mounts.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::raw(if app.show_all {
            "Showing all"
        } else {
            "Filtered"
        }),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Filesystems "));

    frame.render_widget(header, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .mounts
        .iter()
        .enumerate()
        .map(|(i, mount)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let usage = mount.usage_percent();
            let usage_color = if usage > 90.0 {
                Color::Red
            } else if usage > 75.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let fs_icon = match mount.fs_type.as_str() {
                "ext4" | "ext3" | "ext2" => "💾",
                "xfs" | "btrfs" => "🗄️",
                "vfat" | "ntfs" => "📀",
                "tmpfs" => "⚡",
                "nfs" | "cifs" => "🌐",
                _ => "📦",
            };

            let line = Line::from(vec![
                Span::raw(format!("{} ", fs_icon)),
                Span::styled(
                    format!("{:<15}", truncate(&mount.mount_point, 15)),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:>6}", mount.used_formatted()),
                    Style::default().fg(usage_color),
                ),
                Span::raw("/"),
                Span::styled(
                    format!("{:<6}", mount.size_formatted()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:>5.1}%", usage),
                    Style::default().fg(usage_color),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", mount.fs_type),
                    Style::default().fg(Color::Blue),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Mount Points "),
    );

    frame.render_widget(list, area);
}

fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(mount) = app.selected_mount() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);

    // Info section
    let info = vec![
        Line::from(vec![
            Span::styled("Device:      ", Style::default().fg(Color::Cyan)),
            Span::raw(&mount.device),
        ]),
        Line::from(vec![
            Span::styled("Mount Point: ", Style::default().fg(Color::Cyan)),
            Span::raw(&mount.mount_point),
        ]),
        Line::from(vec![
            Span::styled("Filesystem:  ", Style::default().fg(Color::Cyan)),
            Span::raw(&mount.fs_type),
        ]),
        Line::from(vec![
            Span::styled("Total Size:  ", Style::default().fg(Color::Cyan)),
            Span::raw(mount.size_formatted()),
        ]),
        Line::from(vec![
            Span::styled("Used:        ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ({:.1}%)", mount.used_formatted(), mount.usage_percent())),
        ]),
        Line::from(vec![
            Span::styled("Available:   ", Style::default().fg(Color::Cyan)),
            Span::raw(mount.available_formatted()),
        ]),
    ];

    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", mount.mount_point)))
        .wrap(Wrap { trim: false });

    frame.render_widget(info_widget, chunks[0]);

    // Usage gauge
    let usage = mount.usage_percent();
    let gauge_color = if usage > 90.0 {
        Color::Red
    } else if usage > 75.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Usage "))
        .gauge_style(Style::default().fg(gauge_color))
        .percent(usage as u16)
        .label(format!("{:.1}%", usage));

    frame.render_widget(gauge, chunks[1]);

    // Options section
    let options_text = mount.options.join(", ");
    let options = Paragraph::new(options_text)
        .block(Block::default().borders(Borders::ALL).title(" Mount Options "))
        .wrap(Wrap { trim: false });

    frame.render_widget(options, chunks[2]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
