#[allow(unused_imports)]
use log::{debug, error, info, warn};

#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};

use carrier_pigeon_lib::{PigeonError, Request};

#[allow(dead_code)]
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Default)]
enum Modal {
    Loading,
    Environment,
    #[default]
    None,
}

#[derive(Debug, Default)]
enum Pane {
    #[default]
    Select,
    Request,
    Response,
}

#[derive(Debug, Default)]
struct App {
    active_modal: Modal,
    active_pane: Pane,
    requests: Vec<Request>,
}

#[tokio::main]
async fn main() -> Result<(), PigeonError> {
    let config = simplelog::ConfigBuilder::new().build();
    let _logger = CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);

    Ok(())
}
