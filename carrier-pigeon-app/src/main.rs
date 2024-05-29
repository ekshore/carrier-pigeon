use color_eyre::eyre::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};

use state::App;

mod errors;
mod model;
mod state;
mod tui;
mod ui;

#[allow(dead_code)]
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() -> Result<()> {
    errors::install_hooks()?;
    let config = simplelog::ConfigBuilder::new().build();
    let (ui_logger, logs) = ui::log::UILogger::new(LevelFilter::Debug, config.clone());
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
    app.run(&mut tui)?;

    tui::restore()?;

    Ok(())
}
