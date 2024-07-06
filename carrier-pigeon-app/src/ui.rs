use crate::state::*;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    terminal::Frame,
    text::Line,
    widgets::{
        block::{Block, Position, Title},
        BorderType, Borders, Clear, Paragraph, Wrap,
    },
};

use crate::state::App;

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

pub fn draw(app: &App, frame: &mut Frame) {
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

    let request_select_block = title_block(" Requests ".into(), Color::White);
    let request_details_block = title_block(" Request ".into(), Color::White);
    let response_details_block = title_block(" Response ".into(), Color::White);

    let url_bar = title_block(" URL ".into(), Color::White);
    let main_view = title_block(" Body ".into(), Color::White);

    frame.render_widget(request_select_block, view_options[0]);
    frame.render_widget(request_details_block, view_options[1]);
    frame.render_widget(response_details_block, view_options[2]);

    frame.render_widget(url_bar, view_panes[0]);
    frame.render_widget(main_view, view_panes[1]);

    match app.active_modal {
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
