//! UI rendering for process monitor.

use crate::app::{App, View};
use crate::process::{format_bytes, ProcessState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_processes(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);

    if app.show_help {
        draw_help(f);
    }

    if app.show_detail {
        draw_detail(f, app);
    }

    if app.confirm_kill.is_some() {
        draw_confirm_kill(f, app);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let paused = if app.paused { " [PAUSED]" } else { "" };
    let user_filter = app
        .filter_user
        .as_ref()
        .map(|u| format!(" [User: {}]", u))
        .unwrap_or_default();

    let title = format!(
        " Process Monitor | {} processes | Sort: {} {}{}{} ",
        app.filtered_processes().len(),
        app.sort_label(),
        if app.sort_ascending { "↑" } else { "↓" },
        user_filter,
        paused
    );

    let header = Paragraph::new(title).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(header, area);
}

fn draw_processes(f: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::List => draw_list_view(f, app, area),
        View::Tree => draw_tree_view(f, app, area),
    }
}

fn draw_list_view(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_processes();

    let header_cells = vec!["PID", "USER", "CPU%", "MEM", "S", "COMMAND"];
    let header = Row::new(header_cells)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .height(1);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, proc)| {
            let _state_color = match proc.state {
                ProcessState::Running => Color::Green,
                ProcessState::Sleeping => Color::Reset,
                ProcessState::DiskSleep => Color::Yellow,
                ProcessState::Zombie => Color::Red,
                ProcessState::Stopped => Color::Magenta,
                _ => Color::DarkGray,
            };

            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let cells = vec![
                proc.pid.to_string(),
                proc.user.clone(),
                format!("{:.1}", proc.cpu_percent),
                format_bytes(proc.memory_rss),
                proc.state.label().to_string(),
                proc.name.clone(),
            ];

            Row::new(cells).style(style).height(1)
        })
        .collect();

    let title = if app.searching {
        format!(" Search: {} ", app.search)
    } else {
        format!(" {} view ", if app.view == View::List { "List" } else { "Tree" })
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(table, area);
}

fn draw_tree_view(f: &mut Frame, app: &App, area: Rect) {
    let tree = app.tree_processes();

    let items: Vec<ListItem> = tree
        .iter()
        .enumerate()
        .map(|(i, (depth, proc))| {
            let indent = "  ".repeat(*depth);
            let prefix = if *depth > 0 { "├─ " } else { "" };

            let state_color = match proc.state {
                ProcessState::Running => Color::Green,
                ProcessState::Zombie => Color::Red,
                ProcessState::Stopped => Color::Magenta,
                _ => Color::Reset,
            };

            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled(proc.state.label(), Style::default().fg(state_color)),
                Span::raw(" "),
                Span::styled(format!("{:>6}", proc.pid), Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(&proc.name, style),
                Span::styled(
                    format!(" ({:.1}% / {})", proc.cpu_percent, format_bytes(proc.memory_rss)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let title = if app.searching {
        format!(" Search: {} ", app.search)
    } else {
        " Tree view ".to_string()
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(list, area);
}

fn draw_status(f: &mut Frame, _app: &App, area: Rect) {
    let status = "q:Quit ?:Help t:Tree s:Sort /:Search Space:Pause 9:Kill T:Term u:FilterUser";
    let para = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));
    f.render_widget(para, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help = Paragraph::new(vec![
        Line::from(Span::styled("Process Monitor Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("  j/k, ↑/↓      Navigate up/down"),
        Line::from("  J/K, PgUp/Dn  Page up/down"),
        Line::from("  g/G           Go to top/bottom"),
        Line::from("  Enter         View process details"),
        Line::from("  t             Toggle tree/list view"),
        Line::from("  s             Cycle sort column"),
        Line::from("  S             Toggle sort direction"),
        Line::from("  /             Search processes"),
        Line::from("  u             Filter by selected user"),
        Line::from("  Space         Pause/resume updates"),
        Line::from("  T             Send SIGTERM (graceful)"),
        Line::from("  9             Send SIGKILL (confirm)"),
        Line::from("  i             Toggle I/O stats"),
        Line::from("  n             Toggle namespace info"),
        Line::from("  ?             Show this help"),
        Line::from("  q             Quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Help "));

    f.render_widget(help, area);
}

fn draw_detail(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, area);

    let content = if let Some(proc) = app.selected_process() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Process Details", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(format!("  PID:      {}", proc.pid)),
            Line::from(format!("  PPID:     {}", proc.ppid)),
            Line::from(format!("  Name:     {}", proc.name)),
            Line::from(format!("  User:     {} ({})", proc.user, proc.uid)),
            Line::from(format!("  State:    {} ({})", proc.state.label(), proc.state.description())),
            Line::from(format!("  CPU:      {:.2}%", proc.cpu_percent)),
            Line::from(format!("  Memory:   {} RSS / {} VMS", format_bytes(proc.memory_rss), format_bytes(proc.memory_vms))),
            Line::from(format!("  Threads:  {}", proc.threads)),
            Line::from(format!("  Nice:     {}", proc.nice)),
            Line::from(""),
            Line::from(format!("  Cmdline:  {}", truncate(&proc.cmdline, 50))),
            Line::from(format!("  Exe:      {}", proc.exe.display())),
        ];

        if let Some(ref io) = proc.io {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("  I/O Stats:", Style::default().add_modifier(Modifier::BOLD))));
            lines.push(Line::from(format!("    Read:   {} ({} syscalls)", format_bytes(io.read_bytes), io.read_syscalls)));
            lines.push(Line::from(format!("    Write:  {} ({} syscalls)", format_bytes(io.write_bytes), io.write_syscalls)));
        }

        if let Some(ref cgroup) = proc.cgroup {
            lines.push(Line::from(""));
            lines.push(Line::from(format!("  Cgroup:   {}", truncate(cgroup, 50))));
        }

        if let Some(ref ns) = proc.namespace {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("  Namespaces:", Style::default().add_modifier(Modifier::BOLD))));
            lines.push(Line::from(format!("    PID:    {}", ns.pid_ns)));
            lines.push(Line::from(format!("    Net:    {}", ns.net_ns)));
            lines.push(Line::from(format!("    Mount:  {}", ns.mnt_ns)));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Press Enter or Esc to close", Style::default().fg(Color::DarkGray))));

        lines
    } else {
        vec![Line::from("No process selected")]
    };

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Process Details "));

    f.render_widget(detail, area);
}

fn draw_confirm_kill(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let pid = app.confirm_kill.unwrap_or(0);
    let name = app
        .processes
        .iter()
        .find(|p| p.pid == pid)
        .map(|p| p.name.as_str())
        .unwrap_or("unknown");

    let confirm = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("Kill Process?", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red))),
        Line::from(""),
        Line::from(format!("  PID: {}", pid)),
        Line::from(format!("  Name: {}", name)),
        Line::from(""),
        Line::from("  This will send SIGKILL (force kill)."),
        Line::from(""),
        Line::from(Span::styled("  [Y]es  [N]o", Style::default().fg(Color::Yellow))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Confirm "));

    f.render_widget(confirm, area);
}

fn centered_rect(px: u16, py: u16, area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - py) / 2),
            Constraint::Percentage(py),
            Constraint::Percentage((100 - py) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - px) / 2),
            Constraint::Percentage(px),
            Constraint::Percentage((100 - px) / 2),
        ])
        .split(v[1])[1]
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
