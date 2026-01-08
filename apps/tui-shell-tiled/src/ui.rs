use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, Mode};
use crate::container::{Container, Direction};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical.into())
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Main content area
    let workspace = app.current_workspace();
    render_container(frame, &workspace.root, chunks[0], true, &app.config);

    // Status bar
    render_status_bar(frame, app, chunks[1]);

    // Overlays
    if app.show_help {
        render_help(frame);
    }

    if app.mode == Mode::Launcher {
        render_launcher(frame, app);
    }
}

fn render_container(frame: &mut Frame, container: &Container, area: Rect, focused: bool, config: &crate::config::Config) {
    match container {
        Container::Empty { .. } => {
            let border_color = if focused { Color::Cyan } else { Color::DarkGray };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(" Empty ");
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let hint = Paragraph::new("Press Ctrl+P to launch an app")
                .alignment(Alignment::Center);
            frame.render_widget(hint, inner);
        }
        Container::App { name, title, .. } => {
            let border_color = if focused { Color::Cyan } else { Color::DarkGray };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(format!(" {} ", title));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            // Simulate app content
            let content = Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("App: {}", name),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("This is a placeholder for the app content."),
                Line::from("In a real implementation, this would render"),
                Line::from("the actual application interface."),
            ])
            .alignment(Alignment::Center)
            .block(Block::default());
            frame.render_widget(content, inner);
        }
        Container::Split {
            direction,
            children,
            ratios,
            focused: focused_idx,
            ..
        } => {
            if children.is_empty() {
                return;
            }

            let gap = config.gaps.inner;
            let constraints: Vec<Constraint> = ratios
                .iter()
                .map(|r| Constraint::Percentage((*r * 100.0) as u16))
                .collect();

            let layout_direction = match direction {
                Direction::Horizontal => layout::Direction::Horizontal,
                Direction::Vertical => layout::Direction::Vertical,
            };

            let child_areas = Layout::default()
                .direction(layout_direction)
                .constraints(constraints)
                .margin(gap)
                .split(area);

            for (i, (child, child_area)) in children.iter().zip(child_areas.iter()).enumerate() {
                let is_focused = focused && i == *focused_idx;
                render_container(frame, child, *child_area, is_focused, config);
            }
        }
        Container::Tabbed { children, active, .. } => {
            if let Some(child) = children.get(*active) {
                render_container(frame, child, area, focused, config);
            }
        }
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(layout::Direction::Horizontal)
        .constraints([
            Constraint::Length(30),
            Constraint::Min(1),
            Constraint::Length(20),
        ])
        .split(area);

    // Workspace indicators
    let workspaces: String = app
        .workspaces
        .iter()
        .enumerate()
        .map(|(i, ws)| {
            if i == app.active_workspace {
                format!("[{}]", ws.name)
            } else if !ws.is_empty() {
                format!(" {} ", ws.name)
            } else {
                format!(" {} ", ws.name)
            }
        })
        .collect();

    let ws_widget = Paragraph::new(workspaces)
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(ws_widget, chunks[0]);

    // Title / message
    let title = app
        .message
        .as_ref()
        .cloned()
        .unwrap_or_else(|| app.focused_title());
    let title_widget = Paragraph::new(format!(" {} ", title))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(title_widget, chunks[1]);

    // Mode indicator
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Resize => "RESIZE",
        Mode::Move => "MOVE",
        Mode::Launcher => "LAUNCH",
    };
    let mode_style = match app.mode {
        Mode::Normal => Style::default().bg(Color::DarkGray),
        Mode::Resize => Style::default().bg(Color::Yellow).fg(Color::Black),
        Mode::Move => Style::default().bg(Color::Blue).fg(Color::White),
        Mode::Launcher => Style::default().bg(Color::Green).fg(Color::Black),
    };
    let mode_widget = Paragraph::new(format!(" {} ", mode_str))
        .style(mode_style)
        .alignment(Alignment::Right);
    frame.render_widget(mode_widget, chunks[2]);
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Tiled Shell Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+h/j/k/l   Focus left/down/up/right"),
        Line::from("  Ctrl+1-9       Switch workspace"),
        Line::from(""),
        Line::from(Span::styled("Layout", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+v         Split vertical"),
        Line::from("  Ctrl+b         Split horizontal"),
        Line::from("  Ctrl+w         Close focused"),
        Line::from(""),
        Line::from(Span::styled("Modes", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+r         Resize mode"),
        Line::from("  Ctrl+m         Move mode"),
        Line::from("  Ctrl+p         Launch app"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ?              Toggle help"),
        Line::from("  Ctrl+q         Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(help, area);
}

fn render_launcher(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 15, frame.area());
    frame.render_widget(Clear, area);

    let content = vec![
        Line::from("Enter app name:"),
        Line::from(""),
        Line::from(format!("{}|", app.launcher_input)),
    ];

    let launcher = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(" Launch App "),
        )
        .style(Style::default().bg(Color::Black));
    frame.render_widget(launcher, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}

impl From<Direction> for layout::Direction {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Horizontal => layout::Direction::Horizontal,
            Direction::Vertical => layout::Direction::Vertical,
        }
    }
}
