use lcode_config::config::global::USER_CONFIG;
use ratatui::{prelude::*, style::Style, widgets::*, Frame};
use rayon::prelude::*;

use crate::mytui::{
    app::{inner::App, Tab2Panel},
    helper::bottom_rect,
    TuiMode,
};

pub fn draw_difficults(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .topic
        .difficultys
        .par_iter()
        .map(|v| ListItem::new(v.as_str()))
        .collect();

    let style = if app.topic.index == Tab2Panel::Difficulty {
        Style::default().fg(Color::Blue)
    }
    else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .border_style(style)
                .borders(Borders::ALL)
                .title(
                    if app.topic.user_diff.is_empty() {
                        "Difficulty"
                    }
                    else {
                        &app.topic.user_diff
                    },
                )
                .title_alignment(Alignment::Center),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, &mut app.topic.difficultys_state);
}
pub fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let status = &app.topic.ac_status;

    let status_widgets: Vec<Paragraph<'_>> = app
        .topic
        .ac_status
        .iter()
        .map(|v| {
            let temp = Paragraph::new(format!("{}/{}", v.1, v.2))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(v.0.as_str()),
                );
            temp
        })
        .collect();
    for (index, wid) in status_widgets
        .into_iter()
        .enumerate()
    {
        f.render_widget(wid, chunk[index]);
    }

    let total = Paragraph::new(format!(
        "{}/{}",
        status
            .iter()
            .map(|v| { v.1 })
            .sum::<u32>(),
        status
            .iter()
            .map(|v| { v.2 })
            .sum::<u32>(),
    ))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("total"),
    );
    f.render_widget(total, chunk[3]);
}
pub fn draw_all_topic_tags(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .topic
        .topic_tags
        .par_iter()
        .map(|v| {
            let name = if USER_CONFIG.config.translate {
                let mut name = v
                    .name_translated
                    .as_deref()
                    .unwrap_or_default();
                if name.is_empty() {
                    name = v.name.as_str();
                }
                name
            }
            else {
                v.name.as_str()
            };
            ListItem::new(name)
        })
        .collect();
    let style = if app.topic.index == Tab2Panel::AllTopics {
        Style::default().fg(Color::Blue)
    }
    else {
        Style::default()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .border_style(style)
                .borders(Borders::ALL)
                .title("All Topic Tag")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    // .highlight_symbol(">>");
    f.render_stateful_widget(list, area, &mut app.topic.topic_tags_state);
}

pub fn draw_user_topic(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem<'_>> = if USER_CONFIG.config.translate {
        app.topic
            .user_topic_tags_translated
            .par_iter()
            .map(|v| ListItem::new(v.as_str()))
            .collect()
    }
    else {
        app.topic
            .user_topic_tags
            .par_iter()
            .map(|v| ListItem::new(v.as_str()))
            .collect()
    };

    let style = if app.topic.index == Tab2Panel::UserTopics {
        Style::default().fg(Color::Blue)
    }
    else {
        Style::default()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .border_style(style)
                .borders(Borders::ALL)
                .title("User Topic Tag")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    // .highlight_symbol(">>");
    f.render_stateful_widget(list, area, &mut app.topic.user_topic_tags_state);
}

pub fn draw_filtered_qs(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .topic
        .filtered_qs
        .par_iter()
        .map(|v| ListItem::new(v.to_string()))
        .collect();

    let style = if app.topic.index == Tab2Panel::Questions {
        Style::default().fg(Color::Blue)
    }
    else {
        Style::default()
    };
    let count = items.len();
    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("Questions count: {}", count))
                .title_alignment(Alignment::Center)
                .border_style(style)
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    // .highlight_symbol(">>");
    f.render_stateful_widget(list, area, &mut app.topic.filtered_topic_qs_state);
}

/// progress bar, it will draw in `area` bottom
pub fn draw_sync_progress_new(f: &mut Frame, app: &App, area: Rect) {
    let label = Span::styled(
        format!("{:.2}%", app.topic.cur_perc * 100.0),
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::ITALIC | Modifier::BOLD),
    );
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title("waiting sync ……")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Cyan))
        .label(label)
        .ratio(app.topic.cur_perc);

    // let area = centered_rect(60, 20, area);
    let area = bottom_rect(60, area);

    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(gauge, area);
}

/// input to filter question
pub fn draw_input_line(f: &mut Frame, app: &mut App, area: Rect) {
    let (title, sty) = match app.topic.input_line_mode {
        TuiMode::Normal => {
            unreachable!()
        },
        TuiMode::Insert => (
            "Default press `Esc` escape input line",
            Style::default().fg(Color::Yellow),
        ),
        TuiMode::Visual => todo!(),
        TuiMode::OutEdit => ("Default press `e` for input", Style::default()),
    };
    app.topic.text_line.set_block(
        Block::default()
            .borders(Borders::ALL)
            .set_style(sty)
            .title(title),
    );
    f.render_widget(app.topic.text_line.widget(), area);
}
