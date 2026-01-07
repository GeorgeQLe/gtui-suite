use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Mode, View};
use crate::board::Priority;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    match app.view {
        View::BoardList => render_board_list(frame, app, chunks[0]),
        View::Board => render_board(frame, app, chunks[0]),
        View::CardDetail => render_card_detail(frame, app, chunks[0]),
        View::Help => render_help(frame, chunks[0]),
    }

    render_status_bar(frame, app, chunks[1]);

    // Render input dialogs
    match &app.mode {
        Mode::AddBoard(text) => render_input_dialog(frame, "New Board", text),
        Mode::AddColumn(text) => render_input_dialog(frame, "New Column", text),
        Mode::AddCard(text) => render_input_dialog(frame, "New Card", text),
        Mode::EditTitle(text) => render_input_dialog(frame, "Edit Title", text),
        Mode::AddChecklist(text) => render_input_dialog(frame, "Add Checklist Item", text),
        Mode::Normal => {}
    }
}

fn render_board_list(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Boards ");

    let items: Vec<ListItem> = app
        .boards
        .iter()
        .enumerate()
        .map(|(i, board)| {
            let style = if i == app.board_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(&board.name, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_board(frame: &mut Frame, app: &App, area: Rect) {
    let board_name = app
        .current_board
        .as_ref()
        .map(|b| format!(" {} ", b.name))
        .unwrap_or_else(|| " Board ".to_string());

    let block = Block::default().borders(Borders::ALL).title(board_name);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.columns.is_empty() {
        let msg = Paragraph::new("No columns. Press 'A' to add a column.");
        frame.render_widget(msg, inner);
        return;
    }

    // Calculate column widths
    let col_count = app.columns.len();
    let col_width = inner.width / col_count as u16;

    for (col_idx, column) in app.columns.iter().enumerate() {
        let col_area = Rect {
            x: inner.x + (col_idx as u16 * col_width),
            y: inner.y,
            width: col_width.saturating_sub(1),
            height: inner.height,
        };

        render_column(frame, app, column, col_idx, col_area);
    }
}

fn render_column(frame: &mut Frame, app: &App, column: &crate::board::Column, col_idx: usize, area: Rect) {
    let is_selected = col_idx == app.column_index;
    let border_style = if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let card_count = app.cards.get(col_idx).map(|c| c.len()).unwrap_or(0);
    let title = if let Some(limit) = column.wip_limit {
        format!(" {} ({}/{}) ", column.name, card_count, limit)
    } else {
        format!(" {} ({}) ", column.name, card_count)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cards = app.cards.get(col_idx).map(|c| c.as_slice()).unwrap_or(&[]);

    for (card_idx, card) in cards.iter().enumerate() {
        let card_y = inner.y + (card_idx as u16 * 3);
        if card_y + 2 > inner.y + inner.height {
            break;
        }

        let card_area = Rect {
            x: inner.x,
            y: card_y,
            width: inner.width,
            height: 3,
        };

        let is_card_selected = is_selected && card_idx == app.card_index;
        render_card_preview(frame, card, is_card_selected, card_area);
    }
}

fn render_card_preview(frame: &mut Frame, card: &crate::board::Card, selected: bool, area: Rect) {
    let bg = if selected {
        Color::DarkGray
    } else {
        Color::Reset
    };

    let priority_color = match card.priority {
        Priority::Urgent => Color::Red,
        Priority::High => Color::Yellow,
        Priority::Medium => Color::Blue,
        Priority::Low => Color::Green,
    };

    let mut title_line = vec![
        Span::styled(
            format!("{} ", card.priority.symbol()),
            Style::default().fg(priority_color),
        ),
        Span::styled(&card.title, Style::default().bg(bg)),
    ];

    if card.is_overdue() {
        title_line.push(Span::styled(" OVERDUE", Style::default().fg(Color::Red)));
    }

    let (done, total) = card.checklist_progress();
    let mut info_parts = Vec::new();
    if total > 0 {
        info_parts.push(format!("[{}/{}]", done, total));
    }
    if let Some(due) = card.due_date {
        info_parts.push(due.format("%m/%d").to_string());
    }

    let info_line = if info_parts.is_empty() {
        Line::from("")
    } else {
        Line::from(Span::styled(
            info_parts.join(" "),
            Style::default().fg(Color::DarkGray),
        ))
    };

    let text = vec![Line::from(title_line), info_line];
    let paragraph = Paragraph::new(text).style(Style::default().bg(bg));
    frame.render_widget(paragraph, area);
}

fn render_card_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(card) = &app.detail_card else {
        return;
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", card.title));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Priority & dates
            Constraint::Min(1),    // Checklist
        ])
        .split(inner);

    // Priority and dates
    let priority_color = match card.priority {
        Priority::Urgent => Color::Red,
        Priority::High => Color::Yellow,
        Priority::Medium => Color::Blue,
        Priority::Low => Color::Green,
    };

    let mut info_parts = vec![Span::styled(
        format!("Priority: {} {:?}", card.priority.symbol(), card.priority),
        Style::default().fg(priority_color),
    )];

    if let Some(due) = card.due_date {
        let due_style = if card.is_overdue() {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        };
        info_parts.push(Span::raw("  |  "));
        info_parts.push(Span::styled(format!("Due: {}", due), due_style));
    }

    let info = Paragraph::new(Line::from(info_parts));
    frame.render_widget(info, chunks[0]);

    // Checklist
    if !card.checklist.is_empty() {
        let checklist_block = Block::default()
            .borders(Borders::TOP)
            .title(" Checklist ");
        let checklist_inner = checklist_block.inner(chunks[1]);
        frame.render_widget(checklist_block, chunks[1]);

        let items: Vec<ListItem> = card
            .checklist
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let checkbox = if item.completed { "[x]" } else { "[ ]" };
                let style = if i == app.checklist_index {
                    Style::default().bg(Color::DarkGray)
                } else if item.completed {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{} {}", checkbox, item.text),
                    style,
                )))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, checklist_inner);
    } else {
        let msg = Paragraph::new("No checklist items. Press 'c' to add one.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, chunks[1]);
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "Kanban Help",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Board List",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k          Navigate boards"),
        Line::from("  Enter        Open board"),
        Line::from("  a            Add board"),
        Line::from("  d            Delete board"),
        Line::from(""),
        Line::from(Span::styled(
            "Board View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  h/l          Previous/next column"),
        Line::from("  j/k          Move in column"),
        Line::from("  H/L          Move card left/right"),
        Line::from("  J/K          Move card up/down"),
        Line::from("  Enter        Open card details"),
        Line::from("  a            Add card"),
        Line::from("  A            Add column"),
        Line::from("  d            Delete card"),
        Line::from("  p            Change priority"),
        Line::from("  Esc          Return to board list"),
        Line::from(""),
        Line::from(Span::styled(
            "Card Detail",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k          Navigate checklist"),
        Line::from("  Space        Toggle item"),
        Line::from("  c            Add checklist item"),
        Line::from("  e            Edit title"),
        Line::from("  p            Change priority"),
        Line::from("  Esc          Save and return"),
        Line::from(""),
        Line::from(Span::styled(
            "General",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ?            Toggle help"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
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

    let status = Paragraph::new(format!(" {} ", message)).style(style);
    frame.render_widget(status, area);
}

fn render_input_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(50, 20, frame.area());
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
