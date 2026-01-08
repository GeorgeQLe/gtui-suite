use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, ComposeField, InputMode, View};
use crate::models::MailboxType;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_main(frame, app, chunks[0]);
    render_status(frame, app, chunks[1]);

    // Render overlays
    match app.input_mode {
        InputMode::Search => render_search(frame, app),
        InputMode::AccountSelect => render_account_select(frame, app),
        InputMode::MailboxSelect => render_mailbox_select(frame, app),
        _ => {}
    }
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::MailList => {
            if app.show_sidebar {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(25), Constraint::Min(40)])
                    .split(area);

                render_sidebar(frame, app, chunks[0]);
                render_mail_list(frame, app, chunks[1]);
            } else {
                render_mail_list(frame, app, area);
            }
        }
        View::Reading => {
            if app.show_sidebar {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(25), Constraint::Min(40)])
                    .split(area);

                render_sidebar(frame, app, chunks[0]);
                render_email_view(frame, app, chunks[1]);
            } else {
                render_email_view(frame, app, area);
            }
        }
        View::Compose => {
            render_compose(frame, app, area);
        }
    }
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let Some(account) = app.current_account() else {
        return;
    };

    let items: Vec<ListItem> = account
        .mailboxes
        .iter()
        .enumerate()
        .map(|(i, mb)| {
            let style = if i == app.active_mailbox {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let unread = if mb.unread > 0 {
                Span::styled(format!(" ({})", mb.unread), Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            };

            ListItem::new(Line::from(vec![
                Span::raw(mb.display_icon()),
                Span::raw(" "),
                Span::raw(&mb.name),
                unread,
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", account.email)));

    frame.render_widget(list, area);
}

fn render_mail_list(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["", "From", "Subject", "Date"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .emails
        .iter()
        .enumerate()
        .map(|(i, email)| {
            let style = if i == app.selected_email {
                Style::default().bg(Color::DarkGray)
            } else if !email.flags.seen {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let flags = format!(
                "{}{}{}",
                if email.flags.flagged { "★" } else { " " },
                if email.flags.answered { "↩" } else { " " },
                if !email.flags.seen { "●" } else { " " }
            );

            Row::new(vec![
                Cell::from(flags),
                Cell::from(email.from.short_display()),
                Cell::from(email.subject.clone()),
                Cell::from(email.date.format("%m/%d %H:%M").to_string()),
            ])
            .style(style)
        })
        .collect();

    let mailbox_name = app
        .current_mailbox()
        .map(|m| m.name.as_str())
        .unwrap_or("Inbox");

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" {} ", mailbox_name)));

    frame.render_widget(table, area);
}

fn render_email_view(frame: &mut Frame, app: &App, area: Rect) {
    let Some(email) = app.current_email() else {
        let empty = Paragraph::new("No email selected")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5)])
        .split(area);

    // Headers
    let headers = vec![
        Line::from(vec![
            Span::styled("From: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(email.from.display()),
        ]),
        Line::from(vec![
            Span::styled("To: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                email
                    .to
                    .iter()
                    .map(|a| a.display())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(email.date.format(&app.config.display.date_format).to_string()),
        ]),
        Line::from(vec![
            Span::styled("Subject: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&email.subject),
        ]),
    ];

    let header_widget = Paragraph::new(headers)
        .block(Block::default().borders(Borders::ALL).title(" Message "));
    frame.render_widget(header_widget, chunks[0]);

    // Body
    let body = email.body_text.as_deref().unwrap_or("(No text content)");
    let body_widget = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(body_widget, chunks[1]);
}

fn render_compose(frame: &mut Frame, app: &App, area: Rect) {
    let Some(compose) = &app.compose else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);

    let field_style = |field: ComposeField| {
        if app.compose_field == field {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        }
    };

    let to = Paragraph::new(format!(
        "{}{}",
        compose.to,
        if app.compose_field == ComposeField::To { "_" } else { "" }
    ))
    .style(field_style(ComposeField::To))
    .block(Block::default().borders(Borders::ALL).title(" To "));

    let cc = Paragraph::new(format!(
        "{}{}",
        compose.cc,
        if app.compose_field == ComposeField::Cc { "_" } else { "" }
    ))
    .style(field_style(ComposeField::Cc))
    .block(Block::default().borders(Borders::ALL).title(" Cc "));

    let subject = Paragraph::new(format!(
        "{}{}",
        compose.subject,
        if app.compose_field == ComposeField::Subject { "_" } else { "" }
    ))
    .style(field_style(ComposeField::Subject))
    .block(Block::default().borders(Borders::ALL).title(" Subject "));

    let body = Paragraph::new(format!(
        "{}{}",
        compose.body,
        if app.compose_field == ComposeField::Body { "_" } else { "" }
    ))
    .style(field_style(ComposeField::Body))
    .block(Block::default().borders(Borders::ALL).title(" Body "))
    .wrap(Wrap { trim: false });

    frame.render_widget(to, chunks[0]);
    frame.render_widget(cc, chunks[1]);
    frame.render_widget(subject, chunks[2]);
    frame.render_widget(body, chunks[3]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = match app.view {
        View::Compose => Style::default().bg(Color::Blue).fg(Color::White),
        _ => Style::default().bg(Color::DarkGray),
    };

    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);

    frame.render_widget(paragraph, area);
}

fn render_search(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(" Search "));

    frame.render_widget(search, area);
}

fn render_account_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .accounts
        .iter()
        .enumerate()
        .map(|(i, acc)| {
            let style = if i == app.active_account {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(format!("{} <{}>", acc.name, acc.email)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Account "));

    frame.render_widget(list, area);
}

fn render_mailbox_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 50, frame.area());
    frame.render_widget(Clear, area);

    let mailboxes: Vec<ListItem> = app
        .current_account()
        .map(|a| &a.mailboxes)
        .unwrap_or(&vec![])
        .iter()
        .enumerate()
        .map(|(i, mb)| {
            let style = if i == app.active_mailbox {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let unread = if mb.unread > 0 {
                format!(" ({})", mb.unread)
            } else {
                String::new()
            };

            ListItem::new(format!("{} {}{}", mb.display_icon(), mb.name, unread)).style(style)
        })
        .collect();

    let list = List::new(mailboxes)
        .block(Block::default().borders(Borders::ALL).title(" Select Mailbox "));

    frame.render_widget(list, area);
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
