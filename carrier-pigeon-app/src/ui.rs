use log::warn;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    terminal::Frame,
    text::Line,
    widgets::{
        block::{Position, Title},
        Clear, Paragraph, Row, Table, Tabs, Wrap,
    },
};
use ratatui::style::Color;
use widgets::RequestSelect;

mod layout;
pub mod logging;
mod widgets;

use crate::state::*;
use crate::ui::layout::*;
use crate::ui::widgets::UrlBar;

pub fn draw(app: &mut App, frame: &mut Frame) {
    let window_state = &app.window_state;

    let layout = screen_layout(frame);
    let req_layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .margin(1)
        .split(layout.req_area);
    let res_layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .margin(1)
        .split(layout.res_area);

    let req_tabs: Vec<String> = RequestTab::to_vec()
        .iter()
        .map(|v| convert_case(v.to_string()))
        .collect();
    let res_tabs: Vec<String> = ResponseTab::to_vec()
        .iter()
        .map(|v| convert_case(v.to_string()))
        .collect();

    let req_details_block = title_block(
        " Request [3] ".into(),
        if window_state.focused_pane == Pane::Request {
            Color::Green
        } else {
            Color::White
        },
    );
    let req_tabs = Tabs::new(req_tabs)
        .highlight_style(Style::default().bg(Color::White).fg(Color::from_u32(40)))
        .select(app.window_state.req_tab.clone().into());

    let url_bar = UrlBar::construct(app);
    frame.render_widget(url_bar, layout.url_area);

    let req_list =
        RequestSelect::default().requests(app.collection.as_ref().map(|coll| &coll.requests));
    let req_list = if Pane::Select == window_state.focused_pane {
        req_list.focused()
    } else {
        req_list
    };

    match window_state.req_tab {
        RequestTab::Body => {
            let req_body = if let Some(coll) = &app.collection {
                if let Some(req) = &coll.requests.get(
                    window_state
                        .select_list_state
                        .selected()
                        .expect("Expected there to be a selected request"),
                ) {
                    if let Some(body) = &req.body {
                        Paragraph::new(body.as_str()).wrap(Wrap { trim: true })
                    } else {
                        Paragraph::default()
                    }
                } else {
                    warn!(
                        "Tried to retrieve request at index: {}",
                        window_state.select_list_state.selected().unwrap()
                    );
                    Paragraph::default()
                }
            } else {
                Paragraph::default()
            };
            frame.render_widget(req_body, req_layout[1]);
        }
        RequestTab::Headers => {
            let header_table = Table::default()
                .header(Row::new(vec!["Header Name", "Value"]).style(Style::new().bold()))
                .column_spacing(1)
                .rows(if let Some(coll) = &app.collection {
                    if let Some(req) = &coll.requests.get(
                        window_state
                            .select_list_state
                            .selected()
                            .expect("Expected there to be a selected request"),
                    ) {
                        req.headers
                            .iter()
                            .map(|header| {
                                Row::new(vec![header.name.as_ref(), header.value.as_ref()])
                            })
                            .collect()
                    } else {
                        vec![Row::new(vec!["", ""])]
                    }
                } else {
                    vec![Row::new(vec!["", ""])]
                });
            frame.render_widget(header_table, req_layout[1]);
        }
        RequestTab::PathParams => {}
        RequestTab::QueryParams => {}
    }

    let res_details_block = title_block(
        " Response [4] ".into(),
        if window_state.focused_pane == Pane::Response {
            Color::Green
        } else {
            Color::White
        },
    );
    let res_tabs = Tabs::new(res_tabs)
        .highlight_style(Style::default().bg(Color::White).fg(Color::from_u32(40)))
        .select(window_state.res_tab.clone().into());

    frame.render_stateful_widget(
        req_list,
        layout.req_list_area,
        &mut app.window_state.select_list_state,
    );

    frame.render_widget(req_details_block, layout.req_area);
    frame.render_widget(res_details_block, layout.res_area);
    frame.render_widget(req_tabs, req_layout[0]);
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

fn convert_case(str: String) -> String {
    let bytes = str.as_bytes().to_owned();
    let mut return_bytes: Vec<u8> = Vec::with_capacity(bytes.len());
    return_bytes.push(bytes[0]);

    for byte in bytes.iter().skip(1) {
        if *byte < b'a' {
            return_bytes.push(b' ');
        }
        return_bytes.push(*byte);
    }
    String::from_utf8(return_bytes).expect("If the blows up we have bigger problems")
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn convert_case_conversion() {
        let converted_string = convert_case("HelloWorld".into());
        assert_eq!("Hello World", converted_string);
    }

    #[test]
    fn convert_case_no_conversion() {
        let converted_string = convert_case("Hello".into());
        assert_eq!("Hello", converted_string);
    }
}
