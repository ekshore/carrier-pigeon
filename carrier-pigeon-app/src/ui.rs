use ratatui::style::Color;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Style,
    terminal::Frame,
    text::Line,
    widgets::{
        block::{Position, Title},
        Clear, Paragraph, Tabs, Wrap,
    },
};
use widgets::{RequestDetails, RequestSelect};

pub mod logging;

mod layout;
mod util;
mod widgets;

use crate::state::*;
use crate::ui::layout::*;
use crate::ui::widgets::UrlBar;

pub fn draw(app: &mut App, frame: &mut Frame) {
    let layout = screen_layout(frame);

    let url_bar = UrlBar::construct(app);
    frame.render_widget(url_bar, layout.url_area);

    let req_list =
        RequestSelect::default().requests(app.collection.as_ref().map(|coll| &coll.requests));
    let req_list = if Pane::Select == app.window_state.focused_pane {
        req_list.focused()
    } else {
        req_list
    };

    let res_layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .margin(1)
        .split(layout.res_area);
    let res_tabs: Vec<String> = ResponseTab::to_vec()
        .iter()
        .map(|v| util::convert_case(v.to_string()))
        .collect();

    let req_details = if let Some(coll) = &app.collection {
        RequestDetails::default()
            .request(
                coll.requests
                    .get(app.window_state.select_list_state.selected().unwrap_or(0)),
            )
            .is_focused(Pane::Request == app.window_state.focused_pane)
    } else {
        RequestDetails::default()
    };
    frame.render_stateful_widget(
        req_details,
        layout.req_area,
        &mut app.window_state.req_state,
    );

    let res_details_block = title_block(
        " Response [4] ".into(),
        if app.window_state.focused_pane == Pane::Response {
            Color::Green
        } else {
            Color::White
        },
    );
    let res_tabs = Tabs::new(res_tabs)
        .highlight_style(Style::default().bg(Color::White).fg(Color::from_u32(40)))
        .select(app.window_state.res_tab.clone().into());

    frame.render_stateful_widget(
        req_list,
        layout.req_list_area,
        &mut app.window_state.select_list_state,
    );

    frame.render_widget(res_details_block, layout.res_area);
    frame.render_widget(res_tabs, res_layout[0]);

    match &app.window_state.modal {
        Modal::None => {}
        Modal::LoadCollection => {
            let modal = title_block(" Load Collection ".into(), Color::White);
            let modal = modal.title(
                Title::from(" (c) to create / (q) to quit ")
                    .position(Position::Bottom)
                    .alignment(Alignment::Center),
            );
            let modal_area = modal_layout(50, 25, frame.size());

            frame.render_widget(Clear, modal_area);
            frame.render_widget(modal, modal_area);
        }
        Modal::Environment => todo!(),
    }

    if app.show_debug {
        let debug_modal = title_block(" Debug Log ".into(), Color::LightGreen);
        let area = modal_layout(75, 50, frame.size());

        let logs = if let Ok(log_buf) = app.debug_logs.lock() {
            log_buf.display_logs()
        } else {
            vec![Line::from("SHIT")]
        };

        let log_paragraph = Paragraph::new(logs)
            .wrap(Wrap { trim: true })
            .block(debug_modal)
            .alignment(Alignment::Left);

        let line_count = log_paragraph.line_count(area.width) as u16;
        let scroll_offset = if line_count > area.height {
            line_count - area.height
        } else {
            0
        };

        let log_paragraph = log_paragraph.scroll((scroll_offset, 0));

        frame.render_widget(Clear, area);
        frame.render_widget(log_paragraph, area);
    }
}
