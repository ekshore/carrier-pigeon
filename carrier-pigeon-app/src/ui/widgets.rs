use ratatui::{
    layout::Rect,
    style::Color,
    widgets::{Paragraph, Widget},
};

use crate::state::{App, Pane, WindowState};

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
