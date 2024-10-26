use carrier_pigeon_core::{Method, Request};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::state::{App, Pane, WindowState};
use crate::ui::layout;

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
