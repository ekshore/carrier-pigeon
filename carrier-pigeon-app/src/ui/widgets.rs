use log::warn;
use carrier_pigeon_core::{Method, Request};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListState, Paragraph, Row, StatefulWidget, Table, Tabs, Widget, Wrap},
};

use crate::state::{App, Pane, WindowState};
use crate::ui::layout;

use super::{util, RequestDetailsState, RequestTab};

#[derive(Default)]
pub struct UrlBar<'a> {
    is_focused: bool,
    url_text: Option<&'a String>,
}

impl UrlBar<'_> {
    pub fn _new(is_focused: bool, url_text: Option<&String>) -> UrlBar {
        UrlBar {
            is_focused,
            url_text,
        }
    }
}

impl<'a> UrlBar<'a> {
    pub fn construct(app: &'a App) -> UrlBar<'a> {
        let sw: &WindowState = &app.window_state;
        let url_text = if let Some(coll) = &app.collection {
            coll.requests
                .get(
                    sw.select_list_state
                        .selected()
                        .expect("Expected there to be a selected Request"),
                )
                .map(|req| &req.url)
        } else {
            None
        };

        UrlBar {
            url_text,
            is_focused: Pane::Url == sw.focused_pane,
        }
    }

    pub fn _focused(mut self) -> UrlBar<'a> {
        self.is_focused = true;
        self
    }

    pub fn _url(mut self, url: &'a String) -> UrlBar<'a> {
        self.url_text = Some(url);
        self
    }
}

impl Widget for UrlBar<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let url_bar = super::title_block(
            " URL [2] ".into(),
            if self.is_focused {
                Color::Green
            } else {
                Color::White
            },
        );
        if let Some(url) = self.url_text {
            let url_bar = Paragraph::new(url.as_str()).block(url_bar);
            url_bar.render(area, buf);
        } else {
            url_bar.render(area, buf);
        }
    }
}

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

pub struct RequestSelect<'a> {
    requests: List<'a>,
    is_focused: bool,
}

impl<'a> RequestSelect<'a> {
    pub fn _construct(app: &'a App) -> Self {
        let requests: List = if let Some(coll) = &app.collection {
            List::new(coll.requests.iter().map(request_text).collect::<Vec<_>>())
        } else {
            List::new(Vec::<String>::new())
        };
        let is_focused = Pane::Select == app.window_state.focused_pane;
        Self {
            requests,
            is_focused,
        }
    }

    pub fn requests(mut self, reqs: Option<&'a Vec<Request>>) -> Self {
        self.requests = if let Some(reqs) = reqs {
            List::new(reqs.iter().map(request_text).collect::<Vec<_>>())
        } else {
            List::new(Vec::<String>::new())
        };
        self
    }

    pub fn focused(mut self) -> Self {
        self.is_focused = true;
        self
    }
}

impl Default for RequestSelect<'_> {
    fn default() -> Self {
        Self {
            requests: List::new(Vec::<String>::new()),
            is_focused: false,
        }
    }
}

impl StatefulWidget for RequestSelect<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let block = layout::title_block(
            String::from(" Request [1] "),
            if self.is_focused {
                Color::Green
            } else {
                Color::White
            },
        );
        let this = self
            .requests
            .block(block)
            .style(Style::default().fg(Color::White))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .highlight_style(Style::new().add_modifier(Modifier::UNDERLINED))
            .direction(ratatui::widgets::ListDirection::TopToBottom);
        ratatui::widgets::StatefulWidget::render(this, area, buf, state);
    }
}

#[derive(Default)]
pub struct RequestDetails<'a> {
    request: Option<&'a Request>,
    is_focused: bool,
}

impl<'a> RequestDetails<'a> {
    pub fn construct(app: &'a App) -> Self {
        let ws = &app.window_state;
        let is_focused = Pane::Request == ws.focused_pane;
        let request = if let Some(coll) = &app.collection {
            coll.requests
                .get(ws.select_list_state.selected().expect("Should be selected"))
        } else {
            None
        };

        Self {
            request,
            is_focused,
        }
    }

    pub fn request(mut self, req: Option<&'a Request>) -> Self {
        self.request = req;
        self
    }

    pub fn is_focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        self
    }
}

impl StatefulWidget for RequestDetails<'_> {
    type State = RequestDetailsState;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
            .margin(1)
            .split(area);
        let block = layout::title_block(
            String::from(" Request [3] "),
            if self.is_focused {
                Color::Green
            } else {
                Color::White
            },
        );
        block.render(area, buf);

        let req_tabs: Vec<String> = RequestTab::to_vec()
            .iter()
            .map(|v| util::convert_case(v.to_string()))
            .collect();
        let req_tabs = Tabs::new(req_tabs)
            .highlight_style(Style::default().bg(Color::White).fg(Color::from_u32(40)))
            .select(usize::from(state.selected_tab.clone()));
        req_tabs.render(layout[0], buf);

        if let Some(req) = self.request {
            match state.selected_tab {
                RequestTab::Body => {
                    let body = if let Some(body) = &req.body {
                        Paragraph::new(body.as_str()).wrap(Wrap { trim: true })
                    } else {
                        Paragraph::default()
                    };
                    body.render(layout[1], buf);
                }
                RequestTab::Headers => {
                    let header_table = Table::default()
                        .header(Row::new(vec!["Header Name", "Value"]))
                        .rows(
                            req.headers
                                .iter()
                                .map(|header| {
                                    Row::new(vec![header.name.as_ref(), header.value.as_ref()])
                                })
                                .collect::<Vec<Row>>(),
                        );
                    Widget::render(header_table, layout[1], buf);
                }
                RequestTab::PathParams => {}
                RequestTab::QueryParams => {}
            };
        }
    }
}
