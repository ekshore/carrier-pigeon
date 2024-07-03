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

pub type EnvironmentValues = HashMap<String, String>;
pub type Secrets = HashMap<String, String>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Environment {
    pub values: EnvironmentValues,
    pub secrets: Secrets,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Collection {
    pub requests: Vec<Request>,
    pub environments: HashMap<String, Environment>,
    #[serde(skip_serializing)]
    pub save_location: Option<PathBuf>,
}

pub struct SerializedCollection {
    pub requests: HashMap<Box<str>, Box<str>>,
    pub environments: HashMap<Box<str>, Box<str>>,
    pub secrets: HashMap<Box<str>, Box<str>>,
}

impl Collection {
    pub fn serialize(&self) -> SerializedCollection {
        let requests: HashMap<Box<str>, Box<str>> = self
            .requests
            .iter()
            .filter_map(|req| {
                if let Some(ser_req) = serde_json::to_string(req).ok() {
                    Some((req.name.clone(), ser_req))
                } else {
                    None
                }
            })
            .fold(HashMap::new(), |mut requests, (name, ser_req)| {
                requests.insert(name.into_boxed_str(), ser_req.into_boxed_str());
                requests
            });

        let (environments, secrets): (HashMap<Box<str>, Box<str>>, HashMap<Box<str>, Box<str>>) =
            self.environments
                .keys()
                .filter_map(|env_name| {
                    let ser_env = serde_json::to_string(
                        self.environments
                            .get(env_name)
                            .expect("Failed to unwrap value for key inside of iterator"),
                    )
                    .ok();
                    let ser_sec = serde_json::to_string(
                        self.environments
                            .get(env_name)
                            .expect("Failed to unwrap value for key inside of iterator"),
                    )
                    .ok();

                    if let (Some(env), Some(secrets)) = (ser_env, ser_sec) {
                        Some((env_name, env, secrets))
                    } else {
                        None
                    }
                })
                .fold(
                    (HashMap::new(), HashMap::new()),
                    |(mut e, mut s), (name, env, secret)| {
                        e.insert(name.clone().into_boxed_str(), env.into_boxed_str());
                        s.insert(name.clone().into_boxed_str(), secret.into_boxed_str());
                        (e, s)
                    },
                );

        SerializedCollection {
            requests,
            environments,
            secrets,
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
