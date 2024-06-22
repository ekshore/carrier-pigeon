use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
#[allow(unused_imports)]
use log::{debug, error, info, warn};

#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};

use std::{io, time::Duration};

use state::App;

mod errors;
mod model;
mod state;
mod tui;
mod ui;

#[allow(dead_code)]
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

enum Message {
    Input(char),
    Quit,
    ToggleDebug,
}

#[tokio::main]
async fn main() -> Result<()> {
    errors::install_hooks()?;
    let config = simplelog::ConfigBuilder::new().build();
    let (ui_logger, logs) = ui::log::UILogger::new(LevelFilter::Debug, config.clone());
    let _logger = CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Off,
            config,
            TerminalMode::Stdout,
            ColorChoice::Auto,
        ),
        Box::new(ui_logger),
    ]);

    let mut tui = tui::init()?;
    let mut app = App::new(logs);

    while app.running {
        tui.draw(|frame| ui::draw(&app, frame))?;
        let mut current_event = handle_events(&app)?;

        while let Some(msg) = current_event {
            current_event = update(&mut app, msg)?;
        }
    }

    tui::restore()?;

    Ok(())
}

fn handle_events(_: &App) -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(100))? {
        let event = match event::read()? {
            Event::Key(key_event) => handle_key_event(key_event),
            _ => None,
        };
        Ok(event)
    } else {
        Ok(None)
    }
}

fn handle_key_event(key_event: KeyEvent) -> Option<Message> {
    if key_event.kind == event::KeyEventKind::Press {
        match key_event.code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('Q') => Some(Message::Quit),
            KeyCode::F(12) => Some(Message::ToggleDebug),
            _ => None,
        }
    } else {
        None
    }
}

fn update(app: &mut App, msg: Message) -> Result<Option<Message>> {
    debug!("Update Start Current App State: {:?}", &app);
    match msg {
        Message::Quit => {
            app.exit();
            Ok(None)
        }
        Message::ToggleDebug => {
            app.toggle_debug();
            Ok(None)
        }
        _ => Ok(None),
    }
}
