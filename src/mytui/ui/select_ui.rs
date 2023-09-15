use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use rayon::prelude::*;

use crate::{
    entities::index,
    fuzzy_search::filter,
    mytui::app::{App, InputMode},
};

/// soem info
pub(crate) fn draw_msg<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let (msg, style) = match app.input_line_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("C-q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::DIM),
        ),
        InputMode::Insert => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to reset the message"),
            ],
            Style::default(),
        ),
    };

    let mut text = Text::from(Line::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);

    f.render_widget(help_message, area);
}

/// input to filter question
pub(crate) fn draw_input_line<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    app.text_line
        .set_style(match app.input_line_mode {
            InputMode::Normal => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Yellow),
        });
    app.text_line.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Input to filter"),
    );

    f.render_widget(app.text_line.widget(), area);
}

/// list questions
pub(crate) fn draw_table<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let line = &app.text_line.lines()[0];

    match app.input_line_mode {
        InputMode::Normal => {}
        InputMode::Insert => {
            app.questions_filtered = app
                .questions
                .clone()
                .into_par_iter()
                .filter(|v| filter(line, &"", &v.to_string(), 1))
                .collect::<Vec<index::Model>>();
        }
    };

    let items = app
        .questions_filtered
        .par_iter()
        .map(|v| {
            let cells = vec![
                Cell::from(format!("{:07}", v.question_id)),
                Cell::from(format!("{:07}", v.frontend_question_id)),
                Cell::from(v.category.to_owned()),
                Cell::from(v.question_title.to_owned()),
                Cell::from(
                    v.pass_rate
                        .unwrap_or_default()
                        .to_string(),
                ),
                Cell::from(match v.paid_only {
                    true => "",
                    false => "",
                }),
                match v.difficulty {
                    1 => Cell::from("⛳Easy").style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    2 => Cell::from("🕎Medium").style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    3 => Cell::from("💀Hard").style(
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    ),
                    _ => Cell::from(" Unknown").style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                },
                Cell::from(if v.status.is_some() { "👍" } else { "" }),
            ];

            Row::new(cells)
                .height(1)
                .bottom_margin(0)
        })
        .collect::<Vec<Row>>();

    // let items = items.collect::<Vec<Row>>();
    app.questions_len = items.len();

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);

    let header_cells = [
        "ID",
        "Frontend ID",
        "Category",
        "Title",
        "Passing Rate",
        "Paid Only",
        "Difficulty",
        "Status",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let items = Table::new(items)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Sum: {}", app.questions_len)),
        )
        .highlight_style(selected_style)
        .highlight_symbol("")
        .widths(&[
            Constraint::Max(7),
            Constraint::Max(12),
            Constraint::Max(11),
            Constraint::Max(65),
            Constraint::Max(12),
            Constraint::Max(9),
            Constraint::Max(10),
            Constraint::Max(10),
        ]);

    f.render_stateful_widget(items, area, &mut app.state)
}