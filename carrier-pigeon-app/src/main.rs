use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use tokio::{fs, io::AsyncReadExt};

use reqwest;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

use carrier_pigeon_macros::FromWrappedError;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, FromWrappedError)]
enum PigeonError {
    #[wrapper]
    ReqwestError(reqwest::Error),
    #[wrapper]
    InvalidHeaderName(reqwest::header::InvalidHeaderName),
    #[wrapper]
    InvalidHeaderValue(reqwest::header::InvalidHeaderValue),
    #[wrapper]
    IoError(tokio::io::Error),
    #[wrapper]
    JsonError(serde_json::Error),
    Err,
}

#[derive(Debug, Deserialize, Serialize)]
enum Method {
    GET,
    POST,
}

impl Into<reqwest::Method> for Method {
    fn into(self) -> reqwest::Method {
        match self {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum Protocol {
    Http,
    Tcp,
    Rpc,
    Grpc,
}

#[derive(Debug, Deserialize, Serialize)]
struct Header {
    name: Box<str>,
    value: Box<str>,
}

impl Header {
    pub fn fold(headers: Result<HeaderMap, PigeonError>, el: &Self) -> Result<HeaderMap, PigeonError> {
        use reqwest::header::{HeaderName, HeaderValue};
        if let Ok(mut headers) = headers {
            headers.append(
                HeaderName::from_bytes(el.name.as_bytes())?,
                HeaderValue::from_bytes(el.value.as_bytes())?,
            );
            Ok(headers)
        } else {
            Err(PigeonError::Err)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Request {
    protocol: Option<Protocol>,
    url: String,
    method: Method,
    headers: Vec<Header>,
    body: Option<String>,
    path_params: Option<HashMap<String, String>>,
    query_params: Option<HashMap<String, String>>,
}

#[allow(dead_code)]
impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Request {
            protocol: None,
            url,
            method,
            headers: vec![],
            body: None,
            path_params: None,
            query_params: None,
        }
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn headers(mut self, mut headers: Vec<Header>) -> Self {
        self.headers.append(&mut headers);
        self
    }

    pub fn add_header(mut self, header: Header) -> Self {
        self.headers.push(header);
        self
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn path_params(mut self, params: HashMap<String, String>) -> Self {
        self.path_params = Some(params);
        self
    }

    pub fn path_param(mut self, key: String, value: String) -> Self {
        if let Some(mut params) = self.path_params {
            params.insert(key, value);
            self.path_params = Some(params);
        } else {
            let params = HashMap::from([(key, value)]);
            self.path_params = Some(params);
        }
        self
    }

    pub fn query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params = Some(params);
        self
    }

    pub fn query_param(mut self, key: String, value: String) -> Self {
        if let Some(mut params) = self.query_params {
            params.insert(key, value);
            self.query_params = Some(params);
        } else {
            let params = HashMap::from([(key, value)]);
            self.query_params = Some(params);
        }
        self
    }

    pub async fn from_file(file_path: PathBuf) -> Result<Self, PigeonError> {
        info!("Reading single request from file");
        let mut file = fs::File::open(file_path).await?;
        let mut buf = String::new();
        let bytes_read = file.read_to_string(&mut buf).await?;
        debug!("{} bytes read from file", bytes_read);
        let request: Self = serde_json::from_str(&buf)?;
        debug!("Request read from file {:#?}", request);
        Ok(request)
    }

    pub async fn save_to_file(&self, file_path: PathBuf) -> Result<(), PigeonError> {
        let req_json = serde_json::to_string(&self)?;
        let mut file = fs::File::create(file_path).await?;
        info!("Writing to file");
        file.write_all(req_json.as_bytes()).await?;
        Ok(())
    }
}

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
