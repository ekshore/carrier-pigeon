use color_eyre::eyre::Result;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};

use state::App;

mod state;
mod ui;
mod tui;

#[allow(dead_code)]
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() -> Result<()> {
    let config = simplelog::ConfigBuilder::new().build();
    let _logger = CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);

    let mut tui = tui::init()?;

    let mut app = App::default();
    app.run(&mut tui)?;

    tui::restore()?;

    Ok(())
}
