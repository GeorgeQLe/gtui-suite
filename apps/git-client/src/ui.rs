use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Mode, StatusSection, View};
use crate::git_ops::FileState;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    match app.view {
        View::Status => render_status(frame, app, chunks[0]),
        View::Log => render_log(frame, app, chunks[0]),
        View::Branches => render_branches(frame, app, chunks[0]),
        View::Stash => render_stash(frame, app, chunks[0]),
        View::Diff => render_diff(frame, app, chunks[0]),
        View::Help => render_help(frame, chunks[0]),
    }

    render_status_bar(frame, app, chunks[1]);

    match &app.mode {
        Mode::Commit(text) => render_input_dialog(frame, "Commit Message", text),
        Mode::CreateBranch(text) => render_input_dialog(frame, "New Branch Name", text),
        Mode::StashMessage(text) => render_input_dialog(frame, "Stash Message (optional)", text),
        Mode::Normal => {}
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let head_name = app.repo.head_name();
    let state = if app.repo.is_rebasing() {
        " (rebasing)"
    } else if app.repo.is_merging() {
        " (merging)"
    } else {
        ""
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} {} ", head_name, state));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Staged files
    let staged_style = if app.section == StatusSection::Staged {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let staged_items: Vec<ListItem> = app
        .staged_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let selected = app.section == StatusSection::Staged && i == app.file_index;
            let bg = if selected { Color::DarkGray } else { Color::Reset };
            let symbol_color = match file.status {
                FileState::New => Color::Green,
                FileState::Modified => Color::Yellow,
                FileState::Deleted => Color::Red,
                _ => Color::White,
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", file.status.symbol()),
                    Style::default().fg(symbol_color).bg(bg),
                ),
                Span::styled(&file.path, Style::default().bg(bg)),
            ]))
        })
        .collect();

    let staged_block = Block::default()
        .borders(Borders::ALL)
        .border_style(staged_style)
        .title(format!(" Staged ({}) ", app.staged_files.len()));

    let staged_list = List::new(staged_items).block(staged_block);
    frame.render_widget(staged_list, chunks[0]);

    // Unstaged files
    let unstaged_style = if app.section == StatusSection::Unstaged {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let unstaged_items: Vec<ListItem> = app
        .unstaged_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let selected = app.section == StatusSection::Unstaged && i == app.file_index;
            let bg = if selected { Color::DarkGray } else { Color::Reset };
            let symbol_color = match file.status {
                FileState::Untracked => Color::Gray,
                FileState::Modified => Color::Yellow,
                FileState::Deleted => Color::Red,
                _ => Color::White,
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", file.status.symbol()),
                    Style::default().fg(symbol_color).bg(bg),
                ),
                Span::styled(&file.path, Style::default().bg(bg)),
            ]))
        })
        .collect();

    let unstaged_block = Block::default()
        .borders(Borders::ALL)
        .border_style(unstaged_style)
        .title(format!(" Changes ({}) ", app.unstaged_files.len()));

    let unstaged_list = List::new(unstaged_items).block(unstaged_block);
    frame.render_widget(unstaged_list, chunks[1]);
}

fn render_log(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Commit Log ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let selected = i == app.commit_index;
            let bg = if selected { Color::DarkGray } else { Color::Reset };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", commit.short_id),
                    Style::default().fg(Color::Yellow).bg(bg),
                ),
                Span::styled(&commit.message, Style::default().bg(bg)),
                Span::styled(
                    format!(" <{}>", commit.author),
                    Style::default().fg(Color::Cyan).bg(bg),
                ),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_branches(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Branches ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .branches
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            let selected = i == app.branch_index;
            let bg = if selected { Color::DarkGray } else { Color::Reset };

            let prefix = if branch.is_current {
                "* "
            } else if branch.is_remote {
                "  "
            } else {
                "  "
            };

            let name_style = if branch.is_current {
                Style::default().fg(Color::Green).bg(bg)
            } else if branch.is_remote {
                Style::default().fg(Color::Red).bg(bg)
            } else {
                Style::default().bg(bg)
            };

            let mut parts = vec![
                Span::styled(prefix, Style::default().bg(bg)),
                Span::styled(&branch.name, name_style),
            ];

            if branch.ahead > 0 || branch.behind > 0 {
                parts.push(Span::styled(
                    format!(" [+{}, -{}]", branch.ahead, branch.behind),
                    Style::default().fg(Color::Cyan).bg(bg),
                ));
            }

            ListItem::new(Line::from(parts))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_stash(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Stash ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.stashes.is_empty() {
        let msg = Paragraph::new("No stashes. Press 'S' in status view to stash changes.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .stashes
        .iter()
        .enumerate()
        .map(|(i, stash)| {
            let selected = i == app.stash_index;
            let bg = if selected { Color::DarkGray } else { Color::Reset };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("stash@{{{}}}: ", stash.index),
                    Style::default().fg(Color::Yellow).bg(bg),
                ),
                Span::styled(&stash.message, Style::default().bg(bg)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_diff(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Diff ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = app
        .diff_content
        .lines()
        .skip(app.diff_scroll)
        .map(|line| {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                Style::default().fg(Color::Green)
            } else if line.starts_with('-') && !line.starts_with("---") {
                Style::default().fg(Color::Red)
            } else if line.starts_with("@@") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            Line::from(Span::styled(line, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "Git Client Help",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Status View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k          Navigate files"),
        Line::from("  Tab          Switch staged/unstaged"),
        Line::from("  s            Stage file"),
        Line::from("  u            Unstage file"),
        Line::from("  a            Stage all"),
        Line::from("  c            Commit staged"),
        Line::from("  Enter/d      View diff"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  l            Log view"),
        Line::from("  b            Branches view"),
        Line::from("  t            Stash view"),
        Line::from(""),
        Line::from(Span::styled(
            "Remote Operations",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  f            Fetch"),
        Line::from("  P            Pull"),
        Line::from("  p            Push"),
        Line::from(""),
        Line::from(Span::styled(
            "Branches View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Enter        Checkout branch"),
        Line::from("  n            New branch"),
        Line::from("  d            Delete branch"),
        Line::from(""),
        Line::from(Span::styled(
            "Stash View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  S            Create stash"),
        Line::from("  Enter/a      Pop stash"),
        Line::from("  d            Drop stash"),
        Line::from(""),
        Line::from(Span::styled(
            "General",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ?            Toggle help"),
        Line::from("  Esc          Back"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.view {
        View::Status => "Status",
        View::Log => "Log",
        View::Branches => "Branches",
        View::Stash => "Stash",
        View::Diff => "Diff",
        View::Help => "Help",
    };

    let message = app
        .message
        .as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(" [{}] {} ", view_name, message)).style(style);
    frame.render_widget(status, area);
}

fn render_input_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input = Paragraph::new(format!("{}|", value));
    frame.render_widget(input, inner);
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
