use crate::model::Request;
use crate::tui::Tui;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use log::{debug, error, info, trace, warn};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Color,
    text::Line,
    widgets::{
        block::{Position, Title},
        Paragraph,
    },
    Frame,
};

use std::{
    io,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
enum Modal {
    #[default]
    Loading,
    Environment,
    None,
}

#[derive(Debug, Default)]
enum Pane {
    #[default]
    Select,
    Request,
    Response,
    Url,
    Main,
}

pub struct App<'a> {
    active_modal: Modal,
    active_pane: Pane,
    requests: Vec<Request>,
    exit: bool,
    // Debugging
    debug_logs: Arc<Mutex<ui::log::RecordBuff<'a>>>,
    show_debug: bool,
}

impl<'a> App<'a> {
    pub fn new(debug_logs: Arc<Mutex<ui::log::RecordBuff<'a>>>) -> Self {
        App {
            active_modal: Modal::default(),
            active_pane: Pane::default(),
            requests: vec![],
            exit: false,
            debug_logs,
            show_debug: false,
        }
    }
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?
        }
        Ok(())
    }

    pub fn render_frame(&self, frame: &mut Frame) {
        trace!("trace");
        debug!("Start render_frame()");
        info!("info");
        warn!("warn");
        error!("error");
        let vertical_panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(10)])
            .split(frame.size());

        let view_options = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(vertical_panes[0]);

        let view_panes = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(40)])
            .split(vertical_panes[1]);

        let request_select_block = ui::title_block(" Requests ".into(), Color::White);
        let request_details_block = ui::title_block(" Request ".into(), Color::White);
        let response_details_block = ui::title_block(" Response ".into(), Color::White);

        let url_bar = ui::title_block(" URL ".into(), Color::White);
        let main_view = ui::title_block(" Body ".into(), Color::White);

        frame.render_widget(request_select_block, view_options[0]);
        frame.render_widget(request_details_block, view_options[1]);
        frame.render_widget(response_details_block, view_options[2]);

        frame.render_widget(url_bar, view_panes[0]);
        frame.render_widget(main_view, view_panes[1]);

        match self.active_modal {
            Modal::None => {}
            Modal::Loading => {
                let modal = ui::title_block(" Load Collection ".into(), Color::White);
                let modal = modal.title(
                    Title::from(" (c) to create / (q) to quit ")
                        .position(Position::Bottom)
                        .alignment(Alignment::Center),
                );
                let modal_area = ui::modal(50, 25, frame.size());

                frame.render_widget(modal, modal_area);
            }
            Modal::Environment => todo!(),
        }

        if self.show_debug {
            let debug_modal = ui::title_block(" Debug Log ".into(), Color::LightGreen);
            let area = ui::modal(50, 50, frame.size());

            let logs = if let Ok(log_buf) = self.debug_logs.lock() {
                log_buf
                    .log_lines
                    .iter()
                    .filter(|line| line.is_some())
                    .map(|line| line.as_ref().unwrap())
                    .map(|line| line.as_ref().to_owned())
                    .collect()
            } else {
                vec![Line::from("SHIT")]
            };

            let logs = Paragraph::new(logs)
                .block(debug_modal)
                .alignment(Alignment::Left);

            frame.render_widget(logs, area);
        }
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) => self.handle_key_events(key_event),
            _ => {}
        }

        Ok(())
    }

    pub fn handle_key_events(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit(),
            KeyCode::F(12) => self.toggle_debug(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }
}

impl<'a> std::fmt::Debug for App<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("active_modal", &format_args!("{:?}", self.active_modal))
            .field("active_pane", &format_args!("{:?}", self.active_pane))
            .field("requests", &format_args!("{:?}", self.requests))
            .field("exit", &self.exit)
            .finish()
    }
}
