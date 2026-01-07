use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use similar::ChangeTag;

use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    match app.view {
        View::SideBySide => render_side_by_side(frame, app, chunks[0]),
        View::Unified => render_unified(frame, app, chunks[0]),
        View::Help => render_help(frame, chunks[0]),
    }

    render_status_bar(frame, app, chunks[1]);
}

fn render_side_by_side(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_title = app.left_path.as_ref()
        .map(|p| format!(" {} ", p.display()))
        .unwrap_or_else(|| " Left ".to_string());

    let right_title = app.right_path.as_ref()
        .map(|p| format!(" {} ", p.display()))
        .unwrap_or_else(|| " Right ".to_string());

    let visible_height = chunks[0].height.saturating_sub(2) as usize;

    // Left pane
    let left_lines: Vec<Line> = app.left_lines.iter()
        .skip(app.scroll)
        .take(visible_height)
        .map(|line| {
            let style = match line.tag {
                ChangeTag::Delete => Style::default().bg(Color::Red).fg(Color::White),
                ChangeTag::Insert => Style::default().bg(Color::Green).fg(Color::Black),
                ChangeTag::Equal => Style::default(),
            };
            let num = line.old_line.map(|n| format!("{:4} ", n)).unwrap_or_else(|| "     ".to_string());
            Line::from(vec![
                Span::styled(num, Style::default().fg(Color::DarkGray)),
                Span::styled(line.content.trim_end(), style),
            ])
        })
        .collect();

    let left_para = Paragraph::new(left_lines)
        .block(Block::default().borders(Borders::ALL).title(left_title));
    frame.render_widget(left_para, chunks[0]);

    // Right pane
    let right_lines: Vec<Line> = app.right_lines.iter()
        .skip(app.scroll)
        .take(visible_height)
        .map(|line| {
            let style = match line.tag {
                ChangeTag::Delete => Style::default().bg(Color::Red).fg(Color::White),
                ChangeTag::Insert => Style::default().bg(Color::Green).fg(Color::Black),
                ChangeTag::Equal => Style::default(),
            };
            let num = line.new_line.map(|n| format!("{:4} ", n)).unwrap_or_else(|| "     ".to_string());
            Line::from(vec![
                Span::styled(num, Style::default().fg(Color::DarkGray)),
                Span::styled(line.content.trim_end(), style),
            ])
        })
        .collect();

    let right_para = Paragraph::new(right_lines)
        .block(Block::default().borders(Borders::ALL).title(right_title));
    frame.render_widget(right_para, chunks[1]);
}

fn render_unified(frame: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;

    let lines: Vec<Line> = app.left_lines.iter()
        .zip(app.right_lines.iter())
        .skip(app.scroll)
        .take(visible_height)
        .flat_map(|(left, right)| {
            let mut result = Vec::new();

            if left.tag == ChangeTag::Delete {
                result.push(Line::from(vec![
                    Span::styled("-", Style::default().fg(Color::Red)),
                    Span::styled(left.content.trim_end(), Style::default().fg(Color::Red)),
                ]));
            }
            if right.tag == ChangeTag::Insert {
                result.push(Line::from(vec![
                    Span::styled("+", Style::default().fg(Color::Green)),
                    Span::styled(right.content.trim_end(), Style::default().fg(Color::Green)),
                ]));
            }
            if left.tag == ChangeTag::Equal && !left.content.is_empty() {
                result.push(Line::from(vec![
                    Span::raw(" "),
                    Span::raw(left.content.trim_end()),
                ]));
            }

            result
        })
        .collect();

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Unified Diff "));
    frame.render_widget(para, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Diff Tool Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, ↑/↓     Scroll up/down"),
        Line::from("  g/G          Top/bottom"),
        Line::from("  PgUp/PgDn    Page up/down"),
        Line::from("  n/p          Next/previous hunk"),
        Line::from(""),
        Line::from(Span::styled("View", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Tab          Toggle side-by-side/unified"),
        Line::from("  w            Toggle ignore whitespace"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ?            Show help"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (additions, deletions) = app.stats();

    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | Tab Toggle view | n/p Hunks | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(
        " +{} -{} | {} ",
        additions, deletions, message
    )).style(style);

    frame.render_widget(status, area);
}
