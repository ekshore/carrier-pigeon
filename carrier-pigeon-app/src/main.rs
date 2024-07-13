use color_eyre::eyre::{bail, OptionExt, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};
use state::GlobalState;
use std::{collections::HashMap, env, fs, path::PathBuf, time::Duration};
use tokio::sync::mpsc;

mod errors;
mod model;
mod state;
mod tui;
mod ui;

use crate::{
    model::Request,
    state::{
        App, Collection, Environment, EnvironmentValues, Mode, Pane, Secret, SerializedCollection,
        Tab,
    },
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
    RequestPane(Pane),
    SaveCollection,
    SaveGlobal,
    SelectDown,
    SelectLeft,
    SelectRight,
    SelectUp,
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
    let mut app = App::builder()
        .logs(logs)
        .work_dir(
            env::current_dir()
                .expect("Failed to open working directory")
                .join(".pigeon"),
        )
        .global_state(load_global_state()?)
        .build();

    let (event_tx, mut event_rx) = mpsc::channel::<Option<Message>>(100);
    event_tx.send(Some(Message::Start)).await?;

    let event_thread_tx = event_tx.clone();
    let event_thread = tokio::spawn(async move { start_event_thread(event_thread_tx).await });

    while app.running {
        if event_thread.is_finished() {
            bail!("Event thread crashed");
        }

        tui.draw(|frame| ui::draw(&mut app, frame))?;
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

fn load_global_state() -> Result<GlobalState> {
    use env_home::env_home_dir;
    let app_dir = env_home_dir().ok_or_eyre("Failed to open Home directory")?;
    let app_dir = app_dir.join(".local/share/.carrier-pigeon");
    if app_dir.is_dir() {
        debug!("Loading global state from: {}", app_dir.display());
        if let Ok(secrets) = fs::read(app_dir.join("secrets")) {
            let secrets: HashMap<Box<str>, Secret> = serde_json::from_slice(&secrets)?;
            Ok(GlobalState { secrets })
        } else {
            bail!("Failed to load secrets file");
        }
    } else {
        info!("Global directory does not exist creating new");
        fs::create_dir_all(app_dir)?;
        Ok(GlobalState {
            secrets: HashMap::new(),
        })
    }
}

fn save_global_state(state: &GlobalState) -> Result<()> {
    use env_home::env_home_dir;
    let app_dir = env_home_dir().ok_or_eyre("Failed to open home directory")?;
    let app_dir = app_dir.join(".local/share/.carrier-pigeon");
    if app_dir.is_dir() {
        debug!("Saving global state to: {}", app_dir.display());
        let secrets: Vec<u8> = serde_json::to_vec(&state.secrets)?;
        fs::write(app_dir.join("secrets"), secrets)?;
    }
    Ok(())
}

async fn start_event_thread(tx: mpsc::Sender<Option<Message>>) -> Result<()> {
    loop {
        let msg = if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) => Some(Message::RawKeyEvent(key_event)),
                _ => None,
            }
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
            trace!("Input character recieved: '{}'", char);
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
            app.req_list_state.select_first();
            None
        }
        Message::ModeRequest(mode) => {
            trace!("Mode Reqeust: {:?}", mode);
            app.mode = mode;
            None
        }
        Message::NewCollection => {
            debug!("Creating new collection");
            let mut coll = crate::state::Collection::default();
            let request = Request::from_file_sync("./request.json".into())?;
            coll.requests.push(request);
            let mut env_vals: EnvironmentValues = HashMap::new();
            env_vals.insert(
                "TestValue".into(),
                state::EnvironmentValue::Value("Some value".into()),
            );
            coll.environments.push(Environment {
                name: "TestEnvironment".into(),
                values: env_vals,
            });

            app.collection = Some(coll);
            Some(Message::SaveCollection)
        }
        Message::Quit => {
            info!("Quitting...");
            app.running = false;
            update(app, Message::SaveCollection)?;
            update(app, Message::SaveGlobal)?;
            None
        }
        Message::RequestPane(pane) => {
            app.active_pane = pane;
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
            if !path.exists() {
                fs::create_dir_all(path)?;
            }

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
        Message::SaveGlobal => {
            info!("Saving global state");
            save_global_state(&app.global)?;
            None
        }
        Message::SelectDown => {
            trace!("Select Down");
            match app.active_pane {
                Pane::Select => app.req_list_state.select_next(),
                Pane::Request => {}
                Pane::Response => {}
                Pane::Url => {}
            }
            None
        }
        Message::SelectLeft => {
            trace!("Select Left");
            match app.active_pane {
                Pane::Select => {}
                Pane::Request => {
                    let cur_tab_idx: usize = app.req_tab.clone().into();
                    app.req_tab = Tab::from(if cur_tab_idx == 0 {
                        cur_tab_idx
                    } else {
                        cur_tab_idx - 1
                    });
                }
                Pane::Response => {}
                Pane::Url => {}
            }
            None
        }
        Message::SelectRight => {
            trace!("Select Right");
            match app.active_pane {
                Pane::Select => {}
                Pane::Request => {
                    let cur_tab_idx: usize = app.req_tab.clone().into();
                    app.req_tab = Tab::from(cur_tab_idx + 1);
                }
                Pane::Response => {}
                Pane::Url => {}
            }
            None
        }
        Message::SelectUp => {
            trace!("Select Up");
            match app.active_pane {
                Pane::Select => app.req_list_state.select_previous(),
                Pane::Request => {}
                Pane::Response => {}
                Pane::Url => {}
            }
            None
        }
        Message::Start => load_application(app)?,
        Message::ToggleDebug => {
            trace!("Debug Toggle");
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
            KeyCode::Char('1') => Some(Message::RequestPane(Pane::Select)),
            KeyCode::Char('2') => Some(Message::RequestPane(Pane::Url)),
            KeyCode::Char('3') => Some(Message::RequestPane(Pane::Request)),
            KeyCode::Char('4') => Some(Message::RequestPane(Pane::Response)),
            KeyCode::Char('i') => Some(Message::ModeRequest(Mode::Insert)),
            KeyCode::Char('h') => Some(Message::SelectLeft),
            KeyCode::Char('j') => Some(Message::SelectDown),
            KeyCode::Char('k') => Some(Message::SelectUp),
            KeyCode::Char('l') => Some(Message::SelectRight),
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
