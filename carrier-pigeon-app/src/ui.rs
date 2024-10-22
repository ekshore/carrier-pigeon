use crate::state::*;
use log::warn;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    terminal::Frame,
    text::{Line, Span, Text},
    widgets::{
        block::{Block, Position, Title},
        BorderType, Borders, Clear, List, Paragraph, Row, Table, Tabs, Wrap,
    },
};

use crate::state::App;

pub mod logging;

struct ScreenLayout {
    req_list_area: Rect,
    url_area: Rect,
    req_area: Rect,
    res_area: Rect,
    _help_area: Rect,
}

fn screen_layout(frame: &Frame) -> ScreenLayout {
    let vert_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100), Constraint::Length(1)])
        .split(frame.size());

    let horz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(4)])
        .split(vert_chunks[0]);

    let vert_sects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Percentage(100)])
        .split(horz_chunks[1]);

    let view_panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vert_sects[1]);

    ScreenLayout {
        req_list_area: horz_chunks[0],
        url_area: vert_sects[0],
        req_area: view_panes[0],
        res_area: view_panes[1],
        _help_area: vert_chunks[1],
    }
}

pub fn title_block(title_txt: String, color: Color) -> Block<'static> {
    Block::default()
        .title(
            Title::from(title_txt.bold())
                .position(Position::Top)
                .alignment(Alignment::Left),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
}

pub fn modal_layout(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(chunks[1])[1]
}

pub fn draw(app: &mut App, frame: &mut Frame) {
    let window_state = &app.window_state;

    let layout = screen_layout(frame);
    let req_layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .margin(1)
        .split(layout.req_area);
    let res_layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .margin(1)
        .split(layout.res_area);

    let req_select_block = title_block(
        " Requests [1] ".into(),
        if window_state.focused_pane == Pane::Select {
            Color::Green
        } else {
            Color::White
        },
    );

    let req_list: List = if let Some(collection) = &app.collection {
        List::new(
            collection
                .requests
                .iter()
                .map(request_text)
                .collect::<Vec<_>>(),
        )
    } else {
        let empty_list: Vec<String> = vec![];
        List::new(empty_list)
    };

    let req_list = req_list
        .block(req_select_block)
        .style(Style::default().fg(Color::White))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .highlight_style(Style::new().add_modifier(Modifier::UNDERLINED))
        .direction(ratatui::widgets::ListDirection::TopToBottom);

    let url_bar = title_block(
        " URL [2] ".into(),
        if window_state.focused_pane == Pane::Url {
            Color::Green
        } else {
            Color::White
        },
    );

    let tabs: Vec<String> = Tab::to_vec().into_iter().map(|v| v.to_string()).collect();

    let req_details_block = title_block(
        " Request [3] ".into(),
        if window_state.focused_pane == Pane::Request {
            Color::Green
        } else {
            Color::White
        },
    );
    let req_tabs = Tabs::new(tabs.clone())
        .highlight_style(Style::default().bg(Color::White).fg(Color::from_u32(40)))
        .select(app.window_state.req_tab.clone().into());

    if let Some(coll) = &app.collection {
        if let Some(req) = &coll.requests.get(
            app.window_state.select_list_state
                .selected()
                .expect("Expected there to be a selected request"),
        ) {
            let url_text = Paragraph::new(req.url.as_str()).block(url_bar);
            frame.render_widget(url_text, layout.url_area);
        } else {
            frame.render_widget(url_bar, layout.url_area);
        }
    }

    match window_state.req_tab {
        Tab::Body => {
            let req_body = if let Some(coll) = &app.collection {
                if let Some(req) = &coll.requests.get(
                    window_state.select_list_state
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
        Tab::Headers => {
            let header_table = Table::default()
                .header(Row::new(vec!["Header Name", "Value"]).style(Style::new().bold()))
                .column_spacing(1)
                .rows(if let Some(coll) = &app.collection {
                    if let Some(req) = &coll.requests.get(
                        window_state.select_list_state
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
    }

    let res_details_block = title_block(
        " Response [4] ".into(),
        if window_state.focused_pane == Pane::Response {
            Color::Green
        } else {
            Color::White
        },
    );
    let res_tabs = Tabs::new(tabs)
        .highlight_style(Style::default().bg(Color::White).fg(Color::from_u32(40)))
        .select(window_state.res_tab.clone().into());

    frame.render_stateful_widget(req_list, layout.req_list_area, &mut app.window_state.select_list_state);
    //frame.render_widget(url_bar, layout.url_area);
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

use carrier_pigeon_core::{Method, Request};
fn request_text(req: &Request) -> Text<'_> {
    let method_style = match req.method {
        Method::Get => Style::new().green().bold(),
        Method::Post => Style::new().magenta().bold(),
    };
    Line::from(vec![
        Span::styled(format!("{:5}", req.method.to_string()), method_style),
        Span::raw(": "),
        Span::raw(req.name.clone()),
    ])
    .into()
}


    }

    }
}
