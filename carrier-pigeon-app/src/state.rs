use crate::ui;
use carrier_pigeon_core::Request;

use carrier_pigeon_macros::{DisplayEnum, ListEnum, OrderedEnum};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum Modal {
    LoadCollection,
    Environment,
    #[default]
    None,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum Pane {
    #[default]
    Select,
    Request,
    Response,
    Url,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RequestDetailsState {
    pub selected_tab: RequestTab,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, DisplayEnum, ListEnum, OrderedEnum)]
pub enum RequestTab {
    #[default]
    Body,
    Headers,
    PathParams,
    QueryParams,
}

impl RequestTab {
    pub fn prev_tab(self) -> Self {
        let idx: usize = usize::from(self);
        if idx == 0 {
            idx.into()
        } else {
            (idx - 1).into()
        }
    }

    pub fn next_tab(self) -> Self {
        Self::from(usize::from(self) + 1usize)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, DisplayEnum, ListEnum, OrderedEnum)]
pub enum ResponseTab {
    #[default]
    Body,
    Headers,
}

impl ResponseTab {
    pub fn prev_tab(self) -> Self {
        let idx: usize = usize::from(self);
        if idx == 0 {
            idx.into()
        } else {
            (idx - 1).into()
        }
    }

    pub fn next_tab(self) -> Self {
        Self::from(usize::from(self) + 1usize)
    }
}


#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct WindowState {
    pub modal: Modal,
    pub focused_pane: Pane,
    pub req_state: RequestDetailsState,
    pub res_tab: ResponseTab,
    #[serde(skip_serializing, skip_deserializing)]
    pub select_list_state: ListState,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Secret {
    RawValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EnvironmentValue {
    Secret(String),
    Value(String),
}

pub type EnvironmentValues = HashMap<String, EnvironmentValue>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Environment {
    pub name: String,
    pub values: EnvironmentValues,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Collection {
    pub requests: Vec<Request>,
    pub environments: Vec<Environment>,
    #[serde(skip_serializing)]
    pub save_location: Option<PathBuf>,
}

pub struct SerializedCollection {
    pub requests: HashMap<Box<str>, Box<[u8]>>,
    pub environments: HashMap<Box<str>, Box<[u8]>>,
}

impl Collection {
    pub fn serialize(&self) -> SerializedCollection {
        let requests: HashMap<Box<str>, Box<[u8]>> =
            self.requests.iter().fold(HashMap::new(), |mut reqs, req| {
                if let Ok(ser_req) = serde_json::to_vec(req) {
                    reqs.insert(
                        req.name.clone().into_boxed_str(),
                        ser_req.into_boxed_slice(),
                    );
                }
                reqs
            });

        let environments: HashMap<Box<str>, Box<[u8]>> =
            self.environments
                .iter()
                .fold(HashMap::new(), |mut envs, env| {
                    if let Ok(ser_env) = serde_json::to_vec(env) {
                        envs.insert(
                            env.name.clone().into_boxed_str(),
                            ser_env.into_boxed_slice(),
                        );
                    }
                    envs
                });

        SerializedCollection {
            requests,
            environments,
        }
    }

    pub fn deserialize(save_location: PathBuf, ser_coll: SerializedCollection) -> Self {
        let requests: Vec<Request> = ser_coll
            .requests
            .keys()
            .filter_map(|key| {
                serde_json::from_slice(
                    ser_coll
                        .requests
                        .get(key)
                        .expect("Failed to retrieve from map inside key iterator"),
                )
                .ok()
            })
            .collect();
        let environments: Vec<Environment> = ser_coll
            .environments
            .keys()
            .filter_map(|key| {
                if let Ok(values) = serde_json::from_slice(
                    ser_coll
                        .environments
                        .get(key)
                        .expect("Failed to retrieve from map inside key iterator"),
                ) {
                    Some(Environment {
                        name: key.to_string(),
                        values,
                    })
                } else {
                    None
                }
            })
            .collect();

        Collection {
            requests,
            environments,
            save_location: Some(save_location),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalState {
    pub secrets: HashMap<Box<str>, Secret>,
}

pub struct AbsentValue;
pub struct Logs<'a>(Arc<Mutex<ui::logging::RecordBuff<'a>>>);
pub struct State(GlobalState);
pub struct WorkDir(PathBuf);

pub struct AppBuilder<L, G, W> {
    pub logs: L,
    pub global_state: G,
    pub work_dir: W,
}

impl<'a, L, G, W> AppBuilder<L, G, W> {
    pub fn logs(self, logs: Arc<Mutex<ui::logging::RecordBuff<'a>>>) -> AppBuilder<Logs, G, W> {
        AppBuilder::<Logs, G, W> {
            logs: Logs(logs),
            global_state: self.global_state,
            work_dir: self.work_dir,
        }
    }

    pub fn global_state(self, state: GlobalState) -> AppBuilder<L, State, W> {
        AppBuilder::<L, State, W> {
            logs: self.logs,
            global_state: State(state),
            work_dir: self.work_dir,
        }
    }

    pub fn work_dir(self, work_dir: PathBuf) -> AppBuilder<L, G, WorkDir> {
        AppBuilder::<L, G, WorkDir> {
            logs: self.logs,
            global_state: self.global_state,
            work_dir: WorkDir(work_dir),
        }
    }
}

impl<'a> AppBuilder<Logs<'a>, State, WorkDir> {
    pub fn build(self) -> App<'a> {
        App {
            window_state: WindowState::default(),
            mode: Mode::default(),
            collection: None,
            running: true,
            work_dir: self.work_dir.0,
            global: self.global_state.0,
            input_buf: String::default(),
            debug_logs: self.logs.0,
            show_debug: false,
        }
    }
}

pub struct App<'a> {
    pub window_state: WindowState,
    pub mode: Mode,
    pub collection: Option<Collection>,
    pub running: bool,
    pub work_dir: PathBuf,
    pub global: GlobalState,
    pub input_buf: String,
    // Debugging
    pub debug_logs: Arc<Mutex<ui::logging::RecordBuff<'a>>>,
    pub show_debug: bool,
}

impl<'a> App<'a> {
    pub fn builder() -> AppBuilder<AbsentValue, AbsentValue, AbsentValue> {
        AppBuilder::<AbsentValue, AbsentValue, AbsentValue> {
            logs: AbsentValue,
            global_state: AbsentValue,
            work_dir: AbsentValue,
        }
    }
}

impl<'a> std::fmt::Debug for App<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("window_state", &format_args!("{:?}", self.window_state))
            .field("mode", &format_args!("{:?}", self.mode))
            .field("collection", &format_args!("{:?}", self.collection))
            .field("running", &self.running)
            .finish()
    }
}
