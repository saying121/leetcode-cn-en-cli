use std::thread;

use ratatui::{
    prelude::{Backend, Constraint, Layout, Rect, *},
    style::{Style, Stylize},
    widgets::{block::Title, *},
    Frame,
};
use rayon::prelude::*;
use tokio::runtime::Builder;

use crate::{
    config::global::{global_leetcode, global_user_config},
    render::Render,
};

use super::app::{App, InputMode};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let constraints = [Constraint::Length(2), Constraint::Min(1)];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_ref())
        .split(f.size());

    draw_tab(f, app, chunks[0]);

    match app.tab_index {
        0 => {
            let constraints = [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints.as_ref())
                .split(chunks[1]);

            draw_msg(f, app, chunks[0]);
            draw_input(f, app, chunks[1]);

            draw_table(f, app, chunks[2]);
        }
        1 => {
            let constraints = [Constraint::Length(1), Constraint::Min(1)];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints.as_ref())
                .split(chunks[1]);

            let area = chunks[1];
            let chunks1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                )
                .split(area);

            draw_qs_content(f, app, chunks1[0]);
        }
        _ => unreachable!(),
    };
}

pub fn draw_qs_content<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let Rect {
        x: _,
        y: _,
        width,
        height: _height,
    } = area;

    let qs_id = app.current_qs();

    let qs = thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime build failed");

        rt.block_on(async {
            let lcd = global_leetcode();

            lcd.get_problem_detail(crate::leetcode::IdSlug::Id(qs_id), false)
                .await
                .unwrap_or_default()
        })
    })
    .join()
    .unwrap_or_default();

    let qs_str = qs.to_tui_mdvec((width - 2) as usize);

    let text: Vec<Line> = qs_str
        .iter()
        .map(|v| Line::from(Span::raw(v)))
        .collect();
    app.vertical_row_len = text.len();
    app.vertical_scroll_state = app
        .vertical_scroll_state
        .content_length(text.len() as u16);

    let title = match global_user_config().translate {
        true => qs.translated_title.unwrap_or(
            qs.question_title
                .unwrap_or(qs.title),
        ),
        false => qs
            .question_title
            .unwrap_or(qs.title),
    }
    .trim_matches('"')
    .to_string();

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(
                    Title::from(format!("[{}]", title))
                        .alignment(Alignment::Center)
                        .position(block::Position::Top),
                ),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((app.vertical_scroll as u16, 0));
    f.render_widget(paragraph, area);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut app.vertical_scroll_state,
    );
}

pub fn draw_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let titles = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![first.yellow(), rest.green()])
        })
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default()),
        )
        .dim()
        .hidden()
        .select(app.tab_index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, area);
}

pub fn draw_msg<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
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

pub fn draw_input<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let width = area.width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = app
        .input
        .visual_scroll(width as usize);

    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .scroll((0, scroll as u16))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input to filter"),
        );
    f.render_widget(input, area);

    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}
        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                area.x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                // Move one line down, from the border to the input line
                area.y + 1,
            )
        }
    }
}

pub fn draw_table<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let items = app
        .questions
        .par_iter()
        .filter_map(|v| {
            use crate::fuzzy_search::filter;
            let input = app.input.value();

            match filter(input, &"", &v.to_string(), 1) {
                true => {
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
                        Cell::from(v.paid_only.to_string()),
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
                    ];

                    Some(
                        Row::new(cells)
                            .height(1)
                            .bottom_margin(0),
                    )
                }
                false => None,
            }
        });

    let items = items.collect::<Vec<Row>>();
    app.len = items.len();

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
                .title(format!("Sum: {}", app.len)),
        )
        .highlight_style(selected_style)
        .highlight_symbol("")
        .widths(&[
            Constraint::Max(7),
            Constraint::Max(11),
            Constraint::Max(11),
            Constraint::Max(65),
            Constraint::Max(12),
            Constraint::Max(9),
            Constraint::Max(10),
        ]);

    f.render_stateful_widget(items, area, &mut app.state)
}