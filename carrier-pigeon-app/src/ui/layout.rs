use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    terminal::Frame,
    widgets::{
        block::{Block, Position, Title},
        BorderType, Borders,
    },
};

pub struct ScreenLayout {
    pub req_list_area: Rect,
    pub url_area: Rect,
    pub req_area: Rect,
    pub res_area: Rect,
    pub _help_area: Rect,
}

pub fn screen_layout(frame: &Frame) -> ScreenLayout {
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
