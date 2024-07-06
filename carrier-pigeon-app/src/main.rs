use color_eyre::eyre::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};
use std::{collections::HashMap, env, fs, path::PathBuf, time::Duration};
use tokio::sync::mpsc;

mod errors;
mod model;
mod state;
mod tui;
mod ui;

use crate::{
    model::Request,
    state::{App, Collection, Mode, SerializedCollection},
};

#[allow(dead_code)]
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

enum Message {
    Crash(String),
    LoadCollection(PathBuf),
    Input(char),
    ModeRequest(Mode),
    NewCollection,
    Quit,
    RawKeyEvent(KeyEvent),
    SaveCollection,
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

    let (event_tx, mut event_rx) = mpsc::channel::<Option<Message>>(100);
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
    trace!("Start update app state: {:?}", &app);
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
        Message::LoadCollection(path) => {
            info!("Loading collection at: {}", &path.display().to_string());

            let request_dir = path.join("requests");
            let requests: HashMap<Box<str>, Box<[u8]>> =
                if let Ok(files) = fs::read_dir(request_dir) {
                    files.filter_map(|dir_entry| dir_entry.ok()).fold(
                        HashMap::new(),
                        |mut reqs, file| {
                            let data = fs::read(file.path()).unwrap();
                            reqs.insert(
                                file.file_name().into_string().unwrap().into_boxed_str(),
                                data.into_boxed_slice(),
                            );
                            reqs
                        },
                    )
                } else {
                    bail!("Failed to load requests from collection directory");
                };

            let env_dir = path.join("environments");
            let environments: HashMap<Box<str>, Box<[u8]>> =
                if let Ok(files) = fs::read_dir(env_dir) {
                    files.filter_map(|dir_entry| dir_entry.ok()).fold(
                        HashMap::new(),
                        |mut envs, file| {
                            let data = fs::read(file.path()).unwrap();
                            envs.insert(
                                file.file_name().into_string().unwrap().into_boxed_str(),
                                data.into_boxed_slice(),
                            );
                            envs
                        },
                    )
                } else {
                    bail!("Failed to load environments from collection directory");
                };

            app.collection = Some(Collection::deserialize(
                path,
                SerializedCollection {
                    requests,
                    environments,
                },
            ));

            None
        }
        Message::ModeRequest(mode) => {
            app.mode = mode;
            None
        }
        Message::NewCollection => {
            debug!("Creating new collection");
            let mut coll = crate::state::Collection::default();
            let request = Request::from_file_sync("./request.json".into())?;
            coll.requests.push(request);
            app.collection = Some(coll);
            None
        }
        Message::Quit => {
            app.running = false;
            update(app, Message::SaveCollection)?;
            None
        }
        Message::SaveCollection => {
            info!("Saving current collection");
            let (ser_collection, path) = if let Some(coll) = &app.collection {
                let save_path = match &coll.save_location {
                    Some(local) => local,
                    None => &app.work_dir,
                };
                (coll.serialize(), save_path)
            } else {
                bail!("Attempted to serialize a none collection");
            };

            let req_dir = path.join("requests");
            if !req_dir.exists() {
                fs::create_dir(&req_dir)?;
            }
            ser_collection.requests.keys().for_each(|req_key| {
                let _ = fs::write(
                    req_dir.join(req_key.clone().into_string()),
                    ser_collection
                        .requests
                        .get(req_key)
                        .expect("Failed retrieveing from map inside iterator"),
                );
            });

            let env_dir = path.join("environments");
            if !env_dir.exists() {
                fs::create_dir(&env_dir)?;
            }
            ser_collection.environments.keys().for_each(|env_key| {
                let _ = fs::write(
                    env_dir.join(env_key.clone().into_string()),
                    ser_collection
                        .environments
                        .get(env_key)
                        .expect("Failed retrieving from map inside iterator"),
                );
            });

            None
        }
        Message::Start => load_application(app)?,
        Message::ToggleDebug => {
            app.show_debug = !app.show_debug;
            None
        }
        _ => None,
    };
    trace!("End Update app state: {:?}", &app);
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
    debug!("load_application()");
    if app.work_dir.as_path().exists() {
        Ok(Some(Message::LoadCollection(app.work_dir.clone())))
    } else {
        Ok(Some(Message::NewCollection))
    }
}
