//! UI rendering for task manager.

use crate::app::{App, InputField, View};
use crate::models::Status;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);

    match app.view {
        View::List | View::Today => draw_list_view(f, app, chunks[1]),
        View::Board => draw_board_view(f, app, chunks[1]),
        View::Projects => draw_projects_view(f, app, chunks[1]),
    }

    draw_status_bar(f, app, chunks[2]);

    if app.editing {
        draw_input_dialog(f, app);
    }

    if app.show_help {
        draw_help(f);
    }
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] List", "[2] Board", "[3] Projects", "[4] Today"];
    let selected = match app.view {
        View::List => 0,
        View::Board => 1,
        View::Projects => 2,
        View::Today => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Task Manager "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, area);
}

fn draw_list_view(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let status_icon = task.status.symbol();
            let priority_icon = task.priority.symbol();

            let mut style = Style::default();
            if task.is_overdue() {
                style = style.fg(Color::Red);
            } else if task.is_due_today() {
                style = style.fg(Color::Yellow);
            } else if task.status == Status::Done {
                style = style.fg(Color::DarkGray);
            }

            if i == app.selected_index {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }

            let due = task.due_date
                .map(|d| format!(" [{}]", d))
                .unwrap_or_default();

            let project = app.get_project_name(task.project_id)
                .map(|n| format!(" @{}", n))
                .unwrap_or_default();

            let content = format!("{} {} {}{}{}", status_icon, priority_icon, task.title, due, project);
            ListItem::new(content).style(style)
        })
        .collect();

    let title = match app.view {
        View::Today => " Today's Tasks ",
        _ => " Tasks ",
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, area);
}

fn draw_board_view(f: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let statuses = [
        (Status::Todo, "Todo", Color::White),
        (Status::InProgress, "In Progress", Color::Cyan),
        (Status::Blocked, "Blocked", Color::Red),
        (Status::Done, "Done", Color::Green),
    ];

    for (i, (status, title, color)) in statuses.iter().enumerate() {
        let tasks: Vec<ListItem> = app
            .get_tasks_by_status(*status)
            .iter()
            .map(|task| {
                let priority = task.priority.symbol();
                ListItem::new(format!("{} {}", priority, task.title))
            })
            .collect();

        let count = tasks.len();
        let block_title = format!(" {} ({}) ", title, count);

        let list = List::new(tasks)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(block_title)
                .border_style(Style::default().fg(*color)));

        f.render_widget(list, columns[i]);
    }
}

fn draw_projects_view(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .projects
        .iter()
        .map(|project| {
            let task_count = app.tasks.iter()
                .filter(|t| t.project_id == Some(project.id))
                .count();
            ListItem::new(format!("{} ({} tasks)", project.name, task_count))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Projects "));

    f.render_widget(list, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Stats
    let stats = format!(
        "Total: {} | Todo: {} | In Progress: {} | Done: {} | Overdue: {}",
        app.stats.total,
        app.stats.todo,
        app.stats.in_progress,
        app.stats.done,
        app.stats.overdue,
    );
    let stats_widget = Paragraph::new(stats)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(stats_widget, chunks[0]);

    // Message or help hint
    let msg = app.message.clone().unwrap_or_else(|| "? for help | q to quit".to_string());
    let msg_widget = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(msg_widget, chunks[1]);
}

fn draw_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_field {
        InputField::TaskTitle => " New Task ",
        InputField::TaskDescription => " Edit Description ",
        InputField::ProjectName => " New Project ",
        InputField::Search => " Search ",
        InputField::None => " Input ",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(input, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k or arrows  Move selection"),
        Line::from("  g/G            Go to first/last"),
        Line::from("  1-4            Switch views"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  a              Add task"),
        Line::from("  A              Add project"),
        Line::from("  Enter/Space    Toggle status"),
        Line::from("  e              Edit description"),
        Line::from("  d              Delete task"),
        Line::from("  p              Cycle priority"),
        Line::from("  P              Assign project"),
        Line::from(""),
        Line::from(Span::styled("Filters", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /              Search"),
        Line::from("  c              Toggle completed"),
        Line::from("  Esc            Clear filters"),
        Line::from(""),
        Line::from("  q              Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help, area);
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
