use color_eyre::eyre::bail;
use color_eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Deserialize, Serialize)]
pub enum Method {
    GET,
    POST,
}

impl From<Method> for reqwest::Method {
    fn from(val: Method) -> Self {
        match val {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Protocol {
    Http,
    Tcp,
    Rpc,
    Grpc,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Header {
    name: Box<str>,
    value: Box<str>,
}

impl Header {
    pub fn fold(headers: Result<HeaderMap>, el: &Self) -> Result<HeaderMap> {
        use reqwest::header::{HeaderName, HeaderValue};
        if let Ok(mut headers) = headers {
            headers.append(
                HeaderName::from_bytes(el.name.as_bytes())?,
                HeaderValue::from_bytes(el.value.as_bytes())?,
            );
            Ok(headers)
        } else {
            bail!("Invalid Headers")
        }
    }
}

pub struct RequestBuilder<N, M, U> {
    pub name: N,
    pub method: M,
    pub url: U,
}

pub struct NoName;
pub struct Name(String);

pub struct NoMethod;
pub struct HasMethod(Method);

pub struct NoUrl;
pub struct Url(String);

impl<N, M, U> RequestBuilder<N, M, U> {
    pub fn name(self, name: String) -> RequestBuilder<Name, M, U> {
        RequestBuilder::<Name, M, U> {
            name: Name(name),
            method: self.method,
            url: self.url,
        }
    }

    pub fn method(self, method: Method) -> RequestBuilder<N, HasMethod, U> {
        RequestBuilder::<N, HasMethod, U> {
            name: self.name,
            method: HasMethod(method),
            url: self.url,
        }
    }

    pub fn url(self, url: String) -> RequestBuilder<N, M, Url> {
        RequestBuilder::<N, M, Url> {
            name: self.name,
            method: self.method,
            url: Url(url),
        }
    }
}

impl RequestBuilder<Name, HasMethod, Url> {
    pub fn build(self) -> Request {
        Request {
            name: self.name.0,
            protocol: None,
            url: self.url.0,
            method: self.method.0,
            headers: vec![],
            body: None,
            path_params: None,
            query_params: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    pub name: String,
    pub protocol: Option<Protocol>,
    pub url: String,
    pub method: Method,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub path_params: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
}

impl Request {
    pub fn builder() -> RequestBuilder<NoName, NoMethod, NoUrl> {
        RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
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

    pub async fn from_file(file_path: PathBuf) -> Result<Self> {
        info!("Reading single request from file");
        let mut file = fs::File::open(file_path).await?;
        let mut buf = String::new();
        let bytes_read = file.read_to_string(&mut buf).await?;
        debug!("{} bytes read from file", bytes_read);
        let request: Self = serde_json::from_str(&buf)?;
        debug!("Request read from file {:#?}", request);
        Ok(request)
    }

    pub fn from_file_sync(file_path: PathBuf) -> Result<Self> {
        use color_eyre::eyre::WrapErr;
        use std::io::Read;
        info!("Reading single request from file");
        let mut file = std::fs::File::open(&file_path).wrap_err_with(|| {
            let path_str: String = file_path.into_os_string().into_string().unwrap();
            format!("File path: {}", path_str)
        })?;
        let mut buf = String::new();
        let bytes_read = file.read_to_string(&mut buf)?;
        debug!("{} bytes read from file", bytes_read);
        let request: Self = serde_json::from_str(&buf)?;
        debug!("Request read from file {:#?}", request);
        Ok(request)
    }

    pub async fn save_to_file(&self, file_path: PathBuf) -> Result<()> {
        let req_json = serde_json::to_string(&self)?;
        let mut file = fs::File::create(file_path).await?;
        info!("Writing to file");
        file.write_all(req_json.as_bytes()).await?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Collection {
    pub requests: Vec<Request>,
    pub environment: HashMap<String, String>,
}

impl Collection {
    pub fn new() -> Self {
        Collection::default()
    }

    pub fn add_request(&mut self, req: Request) {
        self.requests.push(req);
    }
}
