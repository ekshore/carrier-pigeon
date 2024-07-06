use crate::model::Request;
use crate::ui;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Modal {
    LoadCollection,
    Environment,
    #[default]
    None,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Pane {
    #[default]
    Select,
    Request,
    Response,
    Url,
    Main,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EnvironmentValue {
    Secret(String),
    Value(String),
}

pub type EnvironmentValues = HashMap<String, EnvironmentValue>;
pub type Secrets = HashMap<String, String>;

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
                    reqs.insert(req.name.clone().into_boxed_str(), ser_req.into_boxed_slice());
                }
                reqs
            });

        let environments: HashMap<Box<str>, Box<[u8]>> =
            self.environments
                .iter()
                .fold(HashMap::new(), |mut envs, env| {
                    if let Ok(ser_env) = serde_json::to_vec(env) {
                        envs.insert(env.name.clone().into_boxed_str(), ser_env.into_boxed_slice());
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
                        .expect("Failed to retrieve from map inside key iterator")
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
                        .expect("Failed to retrieve from map inside key iterator")
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

pub struct App<'a> {
    pub mode: Mode,
    pub active_modal: Modal,
    pub active_pane: Pane,
    pub collection: Option<Collection>,
    pub running: bool,
    // Debugging
    pub debug_logs: Arc<Mutex<ui::logging::RecordBuff<'a>>>,
    pub show_debug: bool,
}

impl<'a> App<'a> {
    pub fn new(debug_logs: Arc<Mutex<ui::logging::RecordBuff<'a>>>) -> Self {
        App {
            mode: Mode::default(),
            active_modal: Modal::default(),
            active_pane: Pane::default(),
            collection: None,
            running: true,
            debug_logs,
            show_debug: false,
        }
    }
}

impl<'a> std::fmt::Debug for App<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("mode", &format_args!("{:?}", self.mode))
            .field("active_modal", &format_args!("{:?}", self.active_modal))
            .field("active_pane", &format_args!("{:?}", self.active_pane))
            .field("collection", &format_args!("{:?}", self.collection))
            .field("running", &self.running)
            .finish()
    }
}
