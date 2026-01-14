use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, InputMode, View};
use crate::models::QuestionType;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_main(frame, app, chunks[0]);
    render_status(frame, app, chunks[1]);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::WizardList => render_wizard_list(frame, app, area),
        View::RunWizard => render_run_wizard(frame, app, area),
        View::Preview => render_preview(frame, app, area),
        View::CreateWizard => render_create_wizard(frame, app, area),
    }
}

fn render_wizard_list(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    // Header
    let header = Paragraph::new("Select a wizard to run or create a new one")
        .block(Block::default().borders(Borders::ALL).title(" CLI Wizard Generator "));
    frame.render_widget(header, chunks[0]);

    // Wizard list
    let items: Vec<ListItem> = app
        .wizards
        .iter()
        .enumerate()
        .map(|(i, wizard)| {
            let style = if i == app.selected_wizard {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled("📋 ", Style::default().fg(Color::Yellow)),
                Span::styled(&wizard.name, style),
                Span::styled(
                    format!(" - {}", wizard.description),
                    Style::default().fg(Color::Gray),
                ),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Wizards ({}) ", app.wizards.len())),
    );

    frame.render_widget(list, chunks[1]);
}

fn render_run_wizard(frame: &mut Frame, app: &App, area: Rect) {
    let Some(session) = &app.session else {
        let empty = Paragraph::new("No wizard running")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Wizard title
    let title = Paragraph::new(format!(
        "{} - {}",
        session.wizard.name, session.wizard.description
    ))
    .block(Block::default().borders(Borders::ALL).title(" Wizard "));
    frame.render_widget(title, chunks[0]);

    // Progress
    let (current, total) = session.progress();
    let progress_text = format!("Question {} of {}", current, total);
    let progress_bar_width = ((current as f64 / total as f64) * (chunks[1].width as f64 - 4.0)) as u16;
    let progress = Paragraph::new(Line::from(vec![
        Span::raw(&progress_text),
        Span::raw(" "),
        Span::styled(
            "█".repeat(progress_bar_width as usize),
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            "░".repeat((chunks[1].width.saturating_sub(4).saturating_sub(progress_bar_width).saturating_sub(progress_text.len() as u16 + 1)) as usize),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Progress "));
    frame.render_widget(progress, chunks[1]);

    // Current question
    if let Some(question) = session.current_question() {
        render_question(frame, app, question, chunks[2]);
    } else {
        let completed = Paragraph::new("All questions answered! Press Ctrl+p to preview output.")
            .block(Block::default().borders(Borders::ALL).title(" Complete "));
        frame.render_widget(completed, chunks[2]);
    }

    // Validation error
    if let Some(ref error) = app.validation_error {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title(" Error "));
        frame.render_widget(error_widget, chunks[3]);
    } else {
        let help = match app.input_mode {
            InputMode::Input => "Type your answer and press Enter",
            InputMode::Select => "Use j/k to navigate, Enter to select",
            InputMode::MultiSelect => "Use j/k to navigate, Space to toggle, Enter to confirm",
            InputMode::Confirm => "Press y/n or use arrows and Enter",
            InputMode::Normal => "Press Enter to start answering",
        };
        let help_widget = Paragraph::new(help)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help_widget, chunks[3]);
    }
}

fn render_question(frame: &mut Frame, app: &App, question: &crate::models::Question, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    // Question prompt
    let prompt = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", question.question_type.icon()),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(&question.prompt, Style::default().add_modifier(Modifier::BOLD)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", question.question_type.as_str())),
    );
    frame.render_widget(prompt, chunks[0]);

    // Answer input
    match question.question_type {
        QuestionType::Text | QuestionType::Password | QuestionType::Number | QuestionType::Path => {
            render_text_input(frame, app, question.question_type, chunks[1]);
        }
        QuestionType::Select => {
            render_select_input(frame, app, question, chunks[1]);
        }
        QuestionType::MultiSelect => {
            render_multi_select_input(frame, app, question, chunks[1]);
        }
        QuestionType::Confirm => {
            render_confirm_input(frame, app, chunks[1]);
        }
    }
}

fn render_text_input(frame: &mut Frame, app: &App, question_type: QuestionType, area: Rect) {
    let display_text = if question_type == QuestionType::Password {
        "•".repeat(app.input_buffer.len())
    } else {
        app.input_buffer.clone()
    };

    let style = if app.input_mode == InputMode::Input {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let input = Paragraph::new(display_text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(" Answer "));
    frame.render_widget(input, area);

    // Show cursor
    if app.input_mode == InputMode::Input {
        frame.set_cursor_position(Position::new(
            area.x + 1 + app.cursor_position as u16,
            area.y + 1,
        ));
    }
}

fn render_select_input(frame: &mut Frame, app: &App, question: &crate::models::Question, area: Rect) {
    let items: Vec<ListItem> = question
        .options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == app.selected_option {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let marker = if i == app.selected_option { "▸ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Yellow)),
                Span::raw(&opt.label),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Options "));
    frame.render_widget(list, area);
}

fn render_multi_select_input(frame: &mut Frame, app: &App, question: &crate::models::Question, area: Rect) {
    let items: Vec<ListItem> = question
        .options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let selected = app.selected_options.get(i).copied().unwrap_or(false);
            let style = if i == app.selected_option {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let checkbox = if selected { "☑" } else { "☐" };
            let checkbox_style = if selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", checkbox), checkbox_style),
                Span::raw(&opt.label),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Options (Space to toggle) "),
    );
    frame.render_widget(list, area);
}

fn render_confirm_input(frame: &mut Frame, app: &App, area: Rect) {
    let yes_style = if app.selected_option == 0 {
        Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let no_style = if app.selected_option == 1 {
        Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };

    let confirm = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(" Yes (y) ", yes_style),
        Span::raw("   "),
        Span::styled(" No (n) ", no_style),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Confirm "));
    frame.render_widget(confirm, area);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let content = app.preview_content.as_deref().unwrap_or("No content to preview");

    let lines: Vec<&str> = content.lines().collect();
    let visible_lines: Vec<Line> = lines
        .iter()
        .skip(app.preview_scroll)
        .take(area.height.saturating_sub(2) as usize)
        .enumerate()
        .map(|(i, line)| {
            let line_num = app.preview_scroll + i + 1;
            Line::from(vec![
                Span::styled(
                    format!("{:4} │ ", line_num),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(*line),
            ])
        })
        .collect();

    let preview = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Preview ({}/{} lines) - w:write q:cancel ",
                    app.preview_scroll + 1,
                    lines.len()
                )),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(preview, area);
}

fn render_create_wizard(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup_area);

    let content = Paragraph::new("Wizard creation interface (coming soon)")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Create New Wizard "),
        );
    frame.render_widget(content, popup_area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
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
