use super::super::app::InputMode;
use ratatui::{
    prelude::*,
    style::{Style, Stylize},
    widgets::{block::Title, *},
    Frame,
};

use crate::{
    config::global::glob_user_config, leetcode::resps::run_res::TestSubmit,
    mytui::helper::centered_rect, render::Render,
};

use super::super::app::App;

/// show question's detail
pub(crate) fn draw_qs_content<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    // If want to add effects, it is very troublesome to deal with
    // let Rect {
    //     x: _,
    //     y: _,
    //     width,
    //     height: _height,
    // } = area;
    // let qs_str = qs.to_tui_mdvec((width - 2) as usize);

    let qs = &app.cur_qs;
    let text = qs.to_tui_vec(None);

    app.vertical_row_len = text.len();
    app.vertical_scroll_state = app
        .vertical_scroll_state
        .content_length(text.len() as u16);

    let title = match glob_user_config().translate {
        true => qs
            .translated_title
            .as_ref()
            .unwrap_or(
                qs.question_title
                    .as_ref()
                    .unwrap_or(&qs.title),
            ),
        false => qs
            .question_title
            .as_ref()
            .unwrap_or(&qs.title),
    }
    .trim_matches('"');

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(
                    Title::from(title.bold().blue())
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

/// for edit code
pub(crate) fn draw_code_block<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    app.code_block
        .set_style(match app.edit_code {
            false => Style::default(),
            true => Style::default().fg(Color::Green),
        });

    let (title, color) = if app.edit_code {
        match app.code_block_mode {
            InputMode::Normal => (
                "Normal, Press q to exit edit, vim like keybind, ctrl + s save code",
                Style::default()
                    .fg(Color::Reset)
                    .add_modifier(Modifier::REVERSED),
            ),
            InputMode::Insert => (
                "Insert, emacs like keybind",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::REVERSED),
            ),
        }
    } else {
        (
            "Normal, Press e to start edit",
            Style::default()
                .fg(Color::Reset)
                .add_modifier(Modifier::REVERSED),
        )
    };

    app.code_block.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title(title),
    );
    app.code_block
        .set_cursor_style(color);

    f.render_widget(app.code_block.widget(), area);
}

pub(crate) fn draw_pop_menu<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let area = centered_rect(40, 20, area);

    let text = vec![
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("S", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Submit"),
        ]),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("T", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Test"),
        ]),
        Line::from(""),
        Line::from("Please wait a while after pressing S or T"),
    ];

    let style = match app.submiting {
        true => Style::default().fg(Color::Blue),
        false => Style::default(),
    };

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(style);

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

pub(crate) fn draw_pop_submit<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let text = app
        .submit_res
        .to_tui_vec(Some(TestSubmit::Submit));

    app.submit_row_len = text.len();

    let para = Paragraph::new(text)
        .block(
            Block::default()
                .border_style(Style::default().fg(Color::Cyan))
                .title(Title::from(Line::from(vec![
                    Span::styled("q exit, j/k up/down ", Style::default()),
                    Span::styled("Submit", Style::default().bold()),
                ])))
                .borders(Borders::ALL),
        )
        .scroll((app.submit_vert_scroll as u16, 0));

    let area = centered_rect(60, 60, area);
    f.render_widget(Clear, area);
    f.render_widget(para, area);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut app.submit_vert_scroll_state,
    );
}

pub(crate) fn draw_pop_test<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let text = app
        .test_res
        .to_tui_vec(Some(TestSubmit::Test));

    app.test_row_len = text.len();

    let para = Paragraph::new(text)
        .block(
            Block::default()
                .border_style(Style::default().fg(Color::Cyan))
                .title(Title::from(Line::from(vec![
                    Span::styled("q exit, j/k up/down ", Style::default()),
                    Span::styled("Test", Style::default().bold()),
                ])))
                .borders(Borders::ALL),
        )
        .scroll((app.test_vert_scroll as u16, 0));

    let area = centered_rect(60, 60, area);
    f.render_widget(Clear, area);
    f.render_widget(para, area);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("░"))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut app.test_vert_scroll_state,
    );
}