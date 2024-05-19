use crate::tui::Tui;
use carrier_pigeon_lib::Request;
use ratatui::{
    layout::{Constraint, Direction, Layout},
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
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(10)])
            .split(frame.size());

        let title = Title::from(" Carrier Pigeon ".bold()).position(Position::Top);
        let key_binds = Title::from(Line::from(vec![])).position(Position::Bottom);

        let app_block = Block::default()
            .title(title)
            .title(key_binds)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        frame.render_widget(app_block, frame.size());
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        todo!()
    }
}
