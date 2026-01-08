use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, Mode};
use crate::window::WindowState;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    app.set_screen_size(area.width, area.height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Desktop area
    render_desktop(frame, app, chunks[0]);

    // Taskbar
    render_taskbar(frame, app, chunks[1]);

    // Overlays
    if app.show_help {
        render_help(frame);
    }

    if app.mode == Mode::Launcher {
        render_launcher(frame, app);
    }
}

fn render_desktop(frame: &mut Frame, app: &App, area: Rect) {
    // Desktop background
    let bg = Block::default().style(Style::default().bg(Color::DarkGray));
    frame.render_widget(bg, area);

    let desktop = app.current_desktop();
    let focused_id = desktop.focused;

    // Render windows in z-order
    for window in desktop.windows_sorted_by_z() {
        let is_focused = focused_id == Some(window.id);
        render_window(frame, window, is_focused, area);
    }
}

fn render_window(frame: &mut Frame, window: &crate::window::Window, focused: bool, _desktop_area: Rect) {
    if window.state == WindowState::Minimized {
        return;
    }

    let rect = window.rect.to_ratatui();

    // Window border color
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::Gray
    };

    let title_style = if focused {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    // Title bar
    let title = format!(
        " {} {}",
        window.title,
        if window.always_on_top { "[T]" } else { "" }
    );
    let title_len = title.len().min(rect.width.saturating_sub(8) as usize);
    let title_truncated = &title[..title_len];

    let buttons = " [_][□][X]";
    let title_bar = format!(
        "{:width$}{}",
        title_truncated,
        buttons,
        width = rect.width.saturating_sub(buttons.len() as u16) as usize
    );

    // Window frame
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    // Title bar (inside the window)
    if inner.height > 0 {
        let title_area = Rect::new(inner.x, inner.y, inner.width, 1);
        let title_widget = Paragraph::new(title_bar).style(title_style);
        frame.render_widget(title_widget, title_area);
    }

    // Content area
    if inner.height > 1 {
        let content_area = Rect::new(
            inner.x,
            inner.y + 1,
            inner.width,
            inner.height.saturating_sub(1),
        );

        let content = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  App: {}", window.name),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  This is a placeholder for the"),
            Line::from("  application content."),
        ];

        let content_widget = Paragraph::new(content).style(Style::default().bg(Color::Black));
        frame.render_widget(content_widget, content_area);
    }
}

fn render_taskbar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(1),
            Constraint::Length(15),
        ])
        .split(area);

    // Desktop switcher
    let desktop_switcher: String = app
        .desktops
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let has_windows = !d.windows.is_empty();
            if i == app.active_desktop {
                format!("[{}]", i + 1)
            } else if has_windows {
                format!(" {} ", i + 1)
            } else {
                format!(" {} ", i + 1)
            }
        })
        .collect();

    let desktop_widget = Paragraph::new(desktop_switcher)
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(desktop_widget, chunks[0]);

    // Window list
    let window_list: String = app
        .current_desktop()
        .windows
        .iter()
        .map(|w| {
            let indicator = match w.state {
                WindowState::Minimized => "_",
                WindowState::Maximized => "□",
                WindowState::Normal => "",
            };
            let focused = app.current_desktop().focused == Some(w.id);
            if focused {
                format!("[{}{}]", &w.title[..w.title.len().min(10)], indicator)
            } else {
                format!(" {}{} ", &w.title[..w.title.len().min(10)], indicator)
            }
        })
        .collect();

    let title_or_message = app
        .message
        .as_ref()
        .cloned()
        .unwrap_or(window_list);

    let window_widget = Paragraph::new(title_or_message)
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(window_widget, chunks[1]);

    // Mode indicator
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Move => "MOVE",
        Mode::Resize => "RESIZE",
        Mode::Launcher => "LAUNCH",
    };
    let mode_style = match app.mode {
        Mode::Normal => Style::default().bg(Color::DarkGray),
        Mode::Move => Style::default().bg(Color::Yellow).fg(Color::Black),
        Mode::Resize => Style::default().bg(Color::Blue).fg(Color::White),
        Mode::Launcher => Style::default().bg(Color::Green).fg(Color::Black),
    };

    let mode_widget = Paragraph::new(format!(" {} ", mode_str))
        .style(mode_style)
        .alignment(Alignment::Right);
    frame.render_widget(mode_widget, chunks[2]);
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Floating Shell Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Window Management", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+Tab       Cycle windows"),
        Line::from("  Ctrl+w         Close window"),
        Line::from("  Ctrl+m         Move mode"),
        Line::from("  Ctrl+r         Resize mode"),
        Line::from(""),
        Line::from(Span::styled("Window States", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+Shift+m   Maximize/restore"),
        Line::from("  Ctrl+n         Minimize"),
        Line::from("  Ctrl+Left      Snap left"),
        Line::from("  Ctrl+Right     Snap right"),
        Line::from("  Ctrl+Up        Maximize"),
        Line::from(""),
        Line::from(Span::styled("Desktops", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+1-4       Switch desktop"),
        Line::from("  Ctrl+Shift+1-4 Move to desktop"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+p         Launch app"),
        Line::from("  Ctrl+t         Toggle always on top"),
        Line::from("  Ctrl+a         Cascade windows"),
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
        .split(popup[1])[1]
}
