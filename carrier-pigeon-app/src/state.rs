use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::model::Request;
use crate::ui::logging::RecordBuff;

#[derive(Default)]
pub enum App<'a> {
    #[default]
    Starting,
    Running {
        global_state: GlobalState<'a>,
        state: State,
    },
    Quitting {
        global_state: GlobalState<'a>,
        state: State,
    },
    Stopped,
}

pub struct GlobalState<'a> {
    logs: Arc<Mutex<RecordBuff<'a>>>,
    show_debug: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct State {
    collection: Vec<Request>,
}
