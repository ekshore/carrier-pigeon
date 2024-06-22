use crate::model::Request;
use crate::ui;

use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub enum Modal {
    #[default]
    Loading,
    Environment,
    None,
}

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, Default)]
pub enum Pane {
    #[default]
    Select,
    Request,
    Response,
    Url,
    Main,
}

pub struct App<'a> {
    pub mode: Mode,
    pub active_modal: Modal,
    pub active_pane: Pane,
    pub requests: Vec<Request>,
    pub running: bool,
    // Debugging
    pub debug_logs: Arc<Mutex<ui::log::RecordBuff<'a>>>,
    pub show_debug: bool,
}

impl<'a> App<'a> {
    pub fn new(debug_logs: Arc<Mutex<ui::log::RecordBuff<'a>>>) -> Self {
        App {
            mode: Mode::default(),
            active_modal: Modal::default(),
            active_pane: Pane::default(),
            requests: vec![],
            running: true,
            debug_logs,
            show_debug: false,
        }
    }

    pub fn exit(&mut self) {
        self.running = false;
    }

    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }
}

impl<'a> std::fmt::Debug for App<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("mode:", &format_args!("{:?}", self.mode))
            .field("active_modal", &format_args!("{:?}", self.active_modal))
            .field("active_pane", &format_args!("{:?}", self.active_pane))
            .field("requests", &format_args!("{:?}", self.requests))
            .field("running", &self.running)
            .finish()
    }
}
