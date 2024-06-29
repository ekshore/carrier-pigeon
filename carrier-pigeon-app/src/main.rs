use color_eyre::eyre::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};
use std::time::Duration;
use tokio::sync::mpsc;

mod errors;
mod model;
mod state;
mod tui;
mod ui;

use crate::state::{App, Mode};

#[allow(dead_code)]
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

enum Message {
    Crash(String),
    LoadRequest(String),
    Input(char),
    ModeRequest(Mode),
    Quit,
    RawKeyEvent(KeyEvent),
    Start,
    ToggleDebug,
}

#[tokio::main]
async fn main() -> Result<()> {
    errors::install_hooks()?;
    let config = simplelog::ConfigBuilder::new().build();
    let (ui_logger, logs) = ui::logging::UILogger::new(LevelFilter::Debug, config.clone());
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

    let (event_tx, mut event_rx) = mpsc::channel::<Option<Message>>(50);
    event_tx.send(Some(Message::Start)).await?;

    let event_thread_tx = event_tx.clone();
    let event_thread = tokio::spawn(async move { start_event_thread(event_thread_tx).await });

    while app.running {
        if event_thread.is_finished() {
            bail!("Event thread crashed");
        }

        tui.draw(|frame| ui::draw(&app, frame))?;
        while let Some(msg) = event_rx
            .recv()
            .await
            .unwrap_or_else(|| Some(Message::Crash(String::from("Event channel closed"))))
        {
            if let Some(msg) = update(&mut app, msg)? {
                event_tx.send(Some(msg)).await?;
            }
        }
    }

    tui::restore()?;

    Ok(())
}

async fn start_event_thread(tx: mpsc::Sender<Option<Message>>) -> Result<()> {
    loop {
        let msg = if event::poll(Duration::from_millis(100))? {
            let msg = match event::read()? {
                Event::Key(key_event) => Some(Message::RawKeyEvent(key_event)),
                _ => None,
            };
            msg
        } else {
            None
        };
        tx.send(msg).await?;
    }
}

fn update(app: &mut App, msg: Message) -> Result<Option<Message>> {
    debug!("Start update app state: {:?}", &app);
    let msg = if let Message::RawKeyEvent(event) = msg {
        let msg = match app.mode {
            Mode::Insert => handle_insert_key(event),
            Mode::Normal => handle_normal_key(event),
        };
        if msg.is_none() {
            return Ok(None);
        }
        msg.expect("None check preformed")
    } else {
        msg
    };

    let msg = match msg {
        Message::Crash(message) => {
            bail!("{}", message);
        }
        Message::Input(char) => {
            debug!("Input character recieved: '{}'", char);
            None
        }
        Message::ModeRequest(mode) => {
            app.mode = mode;
            None
        }
        Message::Quit => {
            app.exit();
            None
        }
        Message::Start => load_application(app)?,
        Message::ToggleDebug => {
            app.toggle_debug();
            None
        }
        _ => None,
    };
    debug!("End Update app state: {:?}", &app);
    Ok(msg)
}

fn handle_normal_key(key_event: KeyEvent) -> Option<Message> {
    if key_event.kind == event::KeyEventKind::Press {
        match key_event.code {
            KeyCode::Char('i') => Some(Message::ModeRequest(Mode::Insert)),
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('Q') => Some(Message::Quit),
            KeyCode::F(12) => Some(Message::ToggleDebug),
            _ => None,
        }
    } else {
        None
    }
}

fn handle_insert_key(key_event: KeyEvent) -> Option<Message> {
    if key_event.kind == event::KeyEventKind::Press {
        match key_event.code {
            KeyCode::Esc => Some(Message::ModeRequest(Mode::Normal)),
            KeyCode::Char(char) => Some(Message::Input(char)),
            KeyCode::F(12) => Some(Message::ToggleDebug),
            _ => None,
        }
    } else {
        None
    }
}

fn load_application(app: &mut App) -> Result<Option<Message>> {
    Ok(None)
}
