mod edit_ui;
mod filter_topic;
mod infos;
mod select_ui;

use ratatui::{prelude::*, widgets::*};

use super::helper::*;
use crate::app::{inner::App, TuiIndex};

pub(super) fn start_ui(f: &mut Frame, app: &mut App) {
    let constraints = [Constraint::Length(2), Constraint::Min(1)];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_ref())
        .split(f.size());

    assert!(chunks.len() >= 2);
    draw_tab(f, app, chunks[0]);

    match app.tab_index {
        TuiIndex::Select => {
            let constraints = [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints.as_ref())
                .split(chunks[1]);

            assert!(chunks.len() >= 3);
            select_ui::draw_msg(f, app, chunks[0]);
            select_ui::draw_input_line(f, app, chunks[1]);

            select_ui::draw_table(f, app, chunks[2]);

            if app.select.all_questions.is_empty() {
                select_ui::draw_pop_msg(f, f.size());
            }
            if app.select.sync_state {
                select_ui::draw_sync_progress(f, app, f.size());
            }
        },
        TuiIndex::Edit => {
            let area = chunks[1];
            let chunks1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);

            assert!(chunks1.len() >= 2);
            edit_ui::draw_qs_content(f, app, chunks1[0]);
            edit_ui::draw_code_block(f, app, chunks1[1]);

            if app.edit.show_pop_menu {
                edit_ui::draw_pop_menu(f, app, f.size());
            }

            if app.edit.show_submit_res {
                edit_ui::draw_pop_submit(f, app, f.size());
            }
            if app.edit.show_test_res {
                edit_ui::draw_pop_test(f, app, f.size());
            }
            if app.save_code {
                edit_ui::draw_save_state(f, app, f.size());
            }

            // let button_states = &mut [State::Selected, State::Normal, State::Normal];
            // edit_ui::draw_pop_buttons(f, app, f.size(), button_states);
        },
        TuiIndex::Topic => {
            let area = chunks[1];

            let chunks1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Max(30), Constraint::Max(25), Constraint::Min(0)].as_ref(),
                )
                .split(area);

            assert!(chunks1.len() >= 3);
            let topic_tag_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                .split(chunks1[0]);
            let status_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(4)])
                .split(chunks1[1]);

            let qs_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(chunks1[2]);

            assert!(topic_tag_area.len() >= 2);
            filter_topic::draw_all_topic_tags(f, app, topic_tag_area[0]);
            filter_topic::draw_user_topic(f, app, topic_tag_area[1]);

            assert!(status_area.len() >= 2);
            filter_topic::draw_difficults(f, app, status_area[0]);
            filter_topic::draw_status(f, app, status_area[1]);

            assert!(qs_area.len() >= 2);
            filter_topic::draw_filtered_qs(f, app, qs_area[1]);
            filter_topic::draw_input_line(f, app, qs_area[0]);

            if app.topic.topic_tags.is_empty() {
                select_ui::draw_pop_msg(f, f.size());
            }
            if app.topic.sync_state {
                filter_topic::draw_sync_progress_new(f, app, f.size());
            }
        },
        TuiIndex::Infos => infos::draw_infos(f, app, chunks[1]),
        // 4 => show_config::draw_config(f, app, chunks[1]),
    };

    if app.pop_temp {
        draw_pop_temp(f, app, f.size());
    }
}

fn draw_pop_temp(f: &mut Frame, app: &App, area: Rect) {
    let para = Paragraph::new(app.temp_str.clone()).block(Block::default().borders(Borders::ALL));
    let area = centered_rect_percent(50, 50, area);
    // Clear.render(area, buf);
    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

/// tab bar
fn draw_tab(f: &mut Frame, app: &App, area: Rect) {
    let titles = app.titles.iter().map(|t| {
        let (first, rest) = t.split_at(1);
        Line::from(vec![first.yellow(), rest.green()])
    });
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default()),
        )
        .dim()
        .hidden()
        .select(app.tab_index.into())
        .style(Style::default().fg(Color::Cyan).dim())
        .highlight_style(
            Style::default().add_modifier(Modifier::BOLD), // .bg(Color::Black),
        );
    f.render_widget(tabs, area);
}
