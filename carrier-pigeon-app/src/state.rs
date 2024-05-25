use crate::model::Request;
use crate::tui::Tui;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Paragraph,
    },
    Frame,
};

use std::io;

#[derive(Debug, Default)]
enum Modal {
    Loading,
    Environment,
    #[default]
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

#[derive(Debug, Default)]
pub struct App {
    active_modal: Modal,
    active_pane: Pane,
    requests: Vec<Request>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?
        }
        Ok(())
    }

    pub fn render_frame(&self, frame: &mut Frame) {
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

        let request_select_pane = Block::default()
            .title(
                Title::from(" Requests ".bold())
                    .position(Position::Top)
                    .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        let request_details_pane = Block::default()
            .title(
                Title::from(" Request ".bold())
                    .position(Position::Top)
                    .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        let response_details_pane = Block::default()
            .title(
                Title::from(" Response ".bold())
                    .position(Position::Top)
                    .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        let url_bar = Block::default()
            .title(
                Title::from(" URL ".bold())
                    .position(Position::Top)
                    .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        let main_view = Block::default()
            .title(
                Title::from(" Body ".bold())
                .position(Position::Top)
                .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        frame.render_widget(request_select_pane, view_options[0]);
        frame.render_widget(request_details_pane, view_options[1]);
        frame.render_widget(response_details_pane, view_options[2]);

        frame.render_widget(url_bar, view_panes[0]);
        frame.render_widget(main_view, view_panes[1]);
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
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
