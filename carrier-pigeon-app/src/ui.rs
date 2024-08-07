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
    let layout = screen_layout(frame);
    let req_layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .margin(1)
        .split(layout.req_area);
    let res_layout = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .margin(1)
        .split(layout.res_area);

    //if app.show_debug {
    //    let debug_modal = title_block(" Debug Log ".into(), Color::LightGreen);
    //    let area = modal_layout(75, 50, frame.size());
    //
    //    let logs = if let Ok(log_buf) = app.debug_logs.lock() {
    //        log_buf.display_logs()
    //    } else {
    //        vec![Line::from("SHIT")]
    //    };
    //
    //    let log_paragraph = Paragraph::new(logs)
    //        .wrap(Wrap { trim: true })
    //        .block(debug_modal)
    //        .alignment(Alignment::Left);
    //
    //    let line_count = log_paragraph.line_count(area.width) as u16;
    //    let scroll_offset = if line_count > area.height {
    //        line_count - area.height
    //    } else {
    //        0
    //    };
    //
    //    let log_paragraph = log_paragraph.scroll((scroll_offset, 0));
    //
    //    frame.render_widget(Clear, area);
    //    frame.render_widget(log_paragraph, area);
    //}
}

use crate::model::Request;
impl<'a> From<&Request> for Text<'a> {
    fn from(value: &Request) -> Self {
        use crate::model::Method;
        let method_style = match value.method {
            Method::Get => Style::new().green().bold(),
            Method::Post => Style::new().magenta().bold(),
        };
        Line::from(vec![
            Span::styled(format!("{:5}", value.method.to_string()), method_style),
            Span::raw(": "),
            Span::raw(value.name.clone()),
        ])
        .into()
    }
}

pub mod logging {
    use std::sync::{Arc, Mutex};

    use log::{Level, LevelFilter, Log};
    use ratatui::{
        style::{Style, Stylize},
        text::{Line, Span},
    };
    use simplelog::{Config, SharedLogger};

    pub struct RecordBuff<'a> {
        pub log_lines: [Option<Arc<Line<'a>>>; 256],
        latest_idx: u8,
    }

    impl<'a> RecordBuff<'a> {
        pub fn display_logs(&self) -> Vec<Line<'a>> {
            let mut cur_idx: u8 = if unsafe {
                self.log_lines
                    .get_unchecked((self.latest_idx.wrapping_add(1)) as usize)
            }
            .is_some()
            {
                self.latest_idx.wrapping_add(1)
            } else {
                0
            };

            let mut display_logs: Vec<Line<'a>> = Vec::with_capacity(256);
            let mut empty = unsafe { self.log_lines.get_unchecked(cur_idx as usize).is_none() };
            while !empty {
                if let Some(line) = unsafe { self.log_lines.get_unchecked(cur_idx as usize) } {
                    display_logs.push(line.as_ref().to_owned());
                    empty = cur_idx == self.latest_idx;
                    cur_idx = cur_idx.wrapping_add(1);
                } else {
                    empty = true;
                }
            }
            display_logs
        }
    }

    pub struct UILogger {
        level: LevelFilter,
        config: Config,
        record_buf: Arc<Mutex<RecordBuff<'static>>>,
    }

    impl UILogger {
        pub fn new(level: LevelFilter, config: Config) -> (Self, Arc<Mutex<RecordBuff<'static>>>) {
            const INIT_LINES: Option<Arc<Line<'static>>> = None;
            let latest_idx = 255;
            let log_lines = [INIT_LINES; 256];
            let record_buf = Arc::new(Mutex::new(RecordBuff {
                latest_idx,
                log_lines,
            }));
            (
                UILogger {
                    level,
                    config,
                    record_buf: record_buf.clone(),
                },
                record_buf,
            )
        }
    }

    impl SharedLogger for UILogger {
        fn level(&self) -> LevelFilter {
            self.level
        }

        fn config(&self) -> Option<&Config> {
            Some(&self.config)
        }

        fn as_log(self: Box<Self>) -> Box<dyn log::Log> {
            Box::new(self)
        }
    }

    impl Log for UILogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= self.level
        }

        fn log(&self, record: &log::Record) {
            if !self.enabled(record.metadata()) {
                return;
            }

            let level_sytle = match record.level() {
                Level::Trace => Style::new().light_cyan().bold(),
                Level::Debug => Style::new().light_green().bold(),
                Level::Info => Style::new().light_blue().bold(),
                Level::Warn => Style::new().light_yellow().bold(),
                Level::Error => Style::new().light_red().bold(),
            };

            let log_line = Line::from(vec![
                Span::raw("["),
                Span::styled(record.level().to_string(), level_sytle),
                Span::raw(match record.level() {
                    Level::Info | Level::Warn => "]  ",
                    _ => "] ",
                }),
                Span::raw(record.target().to_owned()),
                Span::raw(": "),
                Span::raw(record.args().to_string()),
            ]);

            let record_buf = self.record_buf.clone();
            if let Ok(mut record_buf) = record_buf.lock() {
                let idx = record_buf.latest_idx.wrapping_add(1);
                record_buf.log_lines[idx as usize] = Some(Arc::new(log_line));
                record_buf.latest_idx = idx;
            };
        }

        fn flush(&self) {}
    }

    #[cfg(test)]
    mod tests {
        use crate::ui::logging::*;
        use log::{Level, Metadata, Record};
        use simplelog::ConfigBuilder;

        #[test]
        fn single_log_insertion() {
            let (logger, logs) = UILogger::new(LevelFilter::Debug, ConfigBuilder::new().build());
            let record = Record::builder()
                .metadata(Metadata::builder().level(Level::Debug).build())
                .level(Level::Debug)
                .target("test::target")
                .args(format_args!("Test Log"))
                .build();

            logger.log(&record);

            if let Ok(logs) = logs.lock() {
                assert_eq!(logs.latest_idx, 0);
                assert_ne!(logs.log_lines[0], None);
            };
        }

        #[test]
        fn multiple_log_insertion() {
            let (logger, logs) = UILogger::new(LevelFilter::Debug, ConfigBuilder::new().build());

            let record = Record::builder()
                .metadata(Metadata::builder().level(Level::Debug).build())
                .level(Level::Debug)
                .target("test::target")
                .args(format_args!("Test Log"))
                .build();

            logger.log(&record);
            logger.log(&record);
            logger.log(&record);

            if let Ok(logs) = logs.lock() {
                assert_eq!(logs.latest_idx, 2);
            } else {
                panic!("lockis poisoned");
            };
        }

        #[test]
        fn overflow_insertion() {
            let (logger, logs) = UILogger::new(LevelFilter::Debug, ConfigBuilder::new().build());

            (0..256)
                .map(|_| {
                    Record::builder()
                        .metadata(Metadata::builder().level(Level::Debug).build())
                        .level(Level::Debug)
                        .target("test::target")
                        .args(format_args!("Test Log"))
                        .build()
                })
                .for_each(|record| logger.log(&record));

            let overflow_record = Record::builder()
                .metadata(Metadata::builder().level(Level::Debug).build())
                .level(Level::Debug)
                .target("test::target")
                .args(format_args!("Test Log :: OVERFLOW"))
                .build();
            logger.log(&overflow_record);

            if let Ok(logs) = logs.lock() {
                assert!(!logs.log_lines[255]
                    .as_ref()
                    .unwrap()
                    .to_string()
                    .contains("OVERFLOW"));
                assert!(logs.log_lines[0]
                    .as_ref()
                    .unwrap()
                    .to_string()
                    .contains("OVERFLOW"));
                assert_eq!(logs.latest_idx, 0);
            } else {
                panic!("lock is poisoned");
            };
        }
    }
}
