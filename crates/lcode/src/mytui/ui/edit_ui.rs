use lcode_config::config::global::G_USER_CONFIG;
use leetcode_api::render::Render;
use ratatui::{prelude::*, widgets::*};

use crate::{
    app::inner::App,
    mytui::{
        helper::centered_rect_percent,
        // my_widget::*,
        TuiMode,
    },
};

/// show question's detail
pub fn draw_qs_content(f: &mut Frame, app: &mut App, area: Rect) {
    // If want to add effects, it is very troublesome to deal with
    // let Rect {
    //     x: _,
    //     y: _,
    //     width,
    //     height: _height,
    // } = area;
    // let qs_str = qs.to_tui_mdvec((width - 2) as usize);

    let title = if G_USER_CONFIG.config.translate {
        app.cur_qs
            .translated_title
            .as_ref()
            .unwrap_or_else(|| {
                app.cur_qs
                    .question_title
                    .as_ref()
                    .unwrap_or(&app.cur_qs.title)
            })
    }
    else {
        app.cur_qs
            .question_title
            .as_ref()
            .unwrap_or(&app.cur_qs.title)
    };

    let text = app.cur_qs.to_tui_vec();

    app.edit.vertical_row_len = text.len();
    app.edit.vertical_scroll_state = app
        .edit
        .vertical_scroll_state
        .content_length(text.len());

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.clone().bold().blue())
                .title_alignment(Alignment::Center)
                .title_position(block::Position::Top),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((app.edit.vertical_scroll as u16, 0));

    f.render_widget(paragraph, area);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut app.edit.vertical_scroll_state,
    );
}

/// for edit code
pub fn draw_code_block(f: &mut Frame, app: &mut App, area: Rect) {
    let title = match app.edit.code_block_mode {
        TuiMode::Normal => "Normal, Press q to exit edit, vim like keybind, ctrl + s save code",
        TuiMode::Insert => "Insert, emacs like keybind",
        TuiMode::OutEdit => "OutEdit, Press e to start edit",
        TuiMode::Visual => todo!(),
    };
    let blk = if matches!(app.edit.code_block_mode, TuiMode::OutEdit) {
        Block::default()
    }
    else {
        Block::default().fg(Color::Green)
    }
    .title(title)
    .borders(Borders::ALL);
    app.edit.code_block.set_block(blk);
    app.edit.code_block.set_cursor_style(
        Style::default()
            .fg(Color::Reset)
            .add_modifier(Modifier::REVERSED),
    );

    f.render_widget(app.edit.code_block.widget(), area);
}

pub fn draw_pop_menu(f: &mut Frame, app: &App, area: Rect) {
    let area = centered_rect_percent(40, 20, area);

    let text = vec![
        vec!["Default press ".into(), "S".bold(), " Submit".into()].into(),
        vec!["Default press ".into(), "T".bold(), " Test".into()].into(),
    ];

    let style = if app.edit.submitting {
        Style::default().fg(Color::Blue)
    }
    else {
        Style::default()
    };

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(style);

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

// #[allow(clippy::trivially_copy_pass_by_ref)]
// pub fn draw_pop_buttons(f: &mut Frame, _app: &App, area: Rect, states: &[State; 3]) {
//     let pat = helper::centered_rect_percent(40, 20, area);
//     let layout = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage(33),
//             Constraint::Percentage(33),
//             Constraint::Percentage(33),
//             Constraint::Min(0), // ignore remaining space
//         ])
//         .split(pat);
//     f.render_widget(Clear, pat);
//     f.render_widget(
//         Button::new("Red")
//             .theme(RED)
//             .state(states[0]),
//         layout[0],
//     );
//     f.render_widget(
//         Button::new("Green")
//             .theme(GREEN)
//             .state(states[1]),
//         layout[1],
//     );
//     f.render_widget(
//         Button::new("Blue")
//             .theme(BLUE)
//             .state(states[2]),
//         layout[2],
//     );
// }

pub fn draw_pop_submit(f: &mut Frame, app: &mut App, area: Rect) {
    let text = app.edit.test_res.to_tui_vec();
    app.edit.submit_row_len = text.len();

    let para = Paragraph::new(text)
        .block(
            Block::default()
                .border_style(Style::default().fg(Color::Cyan))
                .title(Line::from(vec![
                    "q exit, j/k up/down ".into(),
                    "Submit".bold(),
                ]))
                .borders(Borders::ALL),
        )
        .scroll((
            app.edit
                .submit_vert_scroll
                .try_into()
                .unwrap_or_default(),
            app.edit
                .submit_hori_scroll
                .try_into()
                .unwrap_or_default(),
        ));

    let area = centered_rect_percent(60, 60, area);
    f.render_widget(Clear, area);
    f.render_widget(para, area);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area,
        &mut app.edit.submit_vert_scroll_state,
    );
}

pub fn draw_pop_test(f: &mut Frame, app: &mut App, area: Rect) {
    let text = app.edit.test_res.to_tui_vec();
    app.edit.test_row_len = text.len();
    let para = Paragraph::new(text)
        .block(
            Block::default()
                .border_style(Style::default().fg(Color::Cyan))
                .title(Line::from(vec![
                    "q exit, j/k up/down ".into(),
                    "Test".bold(),
                ]))
                .borders(Borders::ALL),
        )
        .scroll((
            app.edit
                .test_vert_scroll
                .try_into()
                .unwrap_or_default(),
            app.edit
                .test_hori_scroll
                .try_into()
                .unwrap_or_default(),
        ));

    let area = centered_rect_percent(60, 60, area);
    f.render_widget(Clear, area);
    f.render_widget(para, area);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            // .track_symbol(Some("░"))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area.inner(&Margin { vertical: 0, horizontal: 1 }),
        &mut app.edit.test_vert_scroll_state,
    );
}

pub fn draw_save_state(f: &mut Frame, _app: &App, area: Rect) {
    let area = centered_rect_percent(30, 20, area);

    let para = Paragraph::new("save code done").block(
        Block::default()
            .borders(Borders::ALL)
            .title("default press `esc` close")
            .title_alignment(Alignment::Center),
    );

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}
