use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{
        block::{Block, Position, Title},
        BorderType, Borders,
    },
};

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

pub fn modal(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
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

pub mod log {
    use std::sync::{Arc, Mutex};

    use log::{Level, LevelFilter, Log};
    use ratatui::{
        style::{Style, Stylize},
        text::{Line, Span},
    };
    use simplelog::{Config, SharedLogger};

    pub struct RecordBuff<'a> {
        pub log_lines: [Option<Box<Line<'a>>>; 256],
        latest_idx: u8,
    }

    pub struct UILogger {
        level: LevelFilter,
        config: Config,
        record_buf: Arc<Mutex<RecordBuff<'static>>>,
    }

    impl UILogger {
        pub fn new(level: LevelFilter, config: Config) -> (Self, Arc<Mutex<RecordBuff<'static>>>) {
            const INIT_LINES: Option<Box<Line<'static>>> = None;
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

            let log_record = format!(
                "[{}]:{}({}) - {}",
                record.level(),
                record.target(),
                record.line().map_or("-".into(), |v| format!("{}", v)),
                record.args()
            );

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
                record_buf.log_lines[idx as usize] = Some(Box::new(log_line));
                record_buf.latest_idx = idx;
            };
        }

        fn flush(&self) {}
    }

    #[cfg(test)]
    mod tests {
        use crate::ui::log::*;
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
                assert!(false)
            };
        }

        #[test]
        fn overflow_insertion() {
            let (logger, logs) = UILogger::new(LevelFilter::Debug, ConfigBuilder::new().build());

            let _ = (0..256)
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
                assert!(false);
            };
        }
    }
}
