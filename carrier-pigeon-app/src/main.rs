#[allow(unused_imports)]
use log::{debug, error, info, warn};

#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};

use carrier_pigeon_lib::PigeonError;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Clone)]
enum Message {
    Loading,
    SelectCollection,
    ViewCollection(Option<String>),
}

#[derive(Debug, Default)]
enum App {
    #[default]
    Loading,
    SelectCollection(Option<Vec<String>>),
    ViewCollection(Option<String>),
}

#[tokio::main]
async fn main() -> Result<(), PigeonError> {
    let config = simplelog::ConfigBuilder::new()
        .add_filter_ignore_str("wgpu")
        .add_filter_ignore_str("iced_wgpu")
        .add_filter_ignore_str("naga")
        .build();
    let _logger = CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);

    Ok(())
}
