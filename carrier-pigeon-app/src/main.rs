#[allow(unused_imports)]
use log::{debug, error, info, warn};

use reqwest;
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

use carrier_pigeon_lib::{ Header, PigeonError, Request };

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() -> Result<(), PigeonError> {
    let _logger = CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);

    info!("Building Reqwest Client");
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    use native_dialog;

    let path = native_dialog::FileDialog::new()
        .set_location("~")
        .set_title("Please select a file")
        .add_filter("JSON File", &["json"])
        .show_open_single_file()
        .unwrap();

    let request;
    if let Some(path) = path {
        request = Request::from_file(path).await?;
    } else {
        return Err(PigeonError::Err);
    }

    let path = native_dialog::FileDialog::new()
        .set_location("~")
        .set_title("Please save file")
        .add_filter("JSON file", &["json"])
        .show_save_single_file()
        .unwrap();

    if let Some(path) = path {
        request.save_to_file(path).await?;
    }


    let res = client
        .request(request.method.into(), request.url)
        .body(request.body.unwrap())
        .headers(
            request
                .headers
                .iter()
                .fold(Ok(HeaderMap::new()), Header::fold)?,
        )
        .send()
        .await;

    info!("{:#?}", res.unwrap().text().await?);
    Ok(())
}
