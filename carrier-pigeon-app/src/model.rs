use color_eyre::eyre::bail;
use color_eyre::Result;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use log::{debug, info};

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Method {
    Get,
    Post,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Get => "GET",
            Self::Post => "POST",
        })
    }
}

impl From<Method> for reqwest::Method {
    fn from(val: Method) -> Self {
        match val {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Protocol {
    Http,
    Tcp,
    Rpc,
    Grpc,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Header {
    pub name: Box<str>,
    pub value: Box<str>,
}

#[allow(dead_code)]
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

#[derive(Debug, PartialEq)]
pub struct NoName;
#[derive(Debug, PartialEq)]
pub struct Name(String);

#[derive(Debug, PartialEq)]
pub struct NoMethod;
#[derive(Debug, PartialEq)]
pub struct HasMethod(Method);

#[derive(Debug, PartialEq)]
pub struct NoUrl;
#[derive(Debug, PartialEq)]
pub struct Url(String);

#[derive(Debug, PartialEq)]
pub struct RequestBuilder<N, M, U> {
    pub name: N,
    pub method: M,
    pub url: U,
    pub protocol: Option<Protocol>,
    pub headers: Option<Vec<Header>>,
    pub body: Option<String>,
    pub path_params: Option<HashMap<String, String>>,
    pub query_params: Option<HashMap<String, String>>,
}

#[allow(dead_code)]
impl<N, M, U> RequestBuilder<N, M, U> {
    pub fn name(self, name: String) -> RequestBuilder<Name, M, U> {
        RequestBuilder::<Name, M, U> {
            name: Name(name),
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: self.headers,
            body: self.body,
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }

    pub fn method(self, method: Method) -> RequestBuilder<N, HasMethod, U> {
        RequestBuilder::<N, HasMethod, U> {
            name: self.name,
            method: HasMethod(method),
            url: self.url,
            protocol: self.protocol,
            headers: self.headers,
            body: self.body,
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }

    pub fn url(self, url: String) -> RequestBuilder<N, M, Url> {
        RequestBuilder::<N, M, Url> {
            name: self.name,
            method: self.method,
            url: Url(url),
            protocol: self.protocol,
            headers: self.headers,
            body: self.body,
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }

    pub fn protocol(self, protocol: Protocol) -> RequestBuilder<N, M, U> {
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: Some(protocol),
            headers: self.headers,
            body: self.body,
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }

    pub fn headers(self, headers: Vec<Header>) -> RequestBuilder<N, M, U> {
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: Some(headers),
            body: self.body,
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }

    pub fn header(self, header: Header) -> RequestBuilder<N, M, U> {
        let headers = if let Some(mut headers) = self.headers {
            headers.push(header);
            headers
        } else {
            vec![]
        };
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: Some(headers),
            body: self.body,
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }

    pub fn body(self, body: String) -> RequestBuilder<N, M, U> {
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: self.headers,
            body: Some(body),
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }

    pub fn path_params(self, path_params: HashMap<String, String>) -> RequestBuilder<N, M, U> {
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: self.headers,
            body: self.body,
            path_params: Some(path_params),
            query_params: self.query_params,
        }
    }

    pub fn path_param(self, key: String, value: String) -> RequestBuilder<N, M, U> {
        let path_params = if let Some(mut params) = self.path_params {
            params.insert(key, value);
            params
        } else {
            let mut params = HashMap::new();
            params.insert(key, value);
            params
        };
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: self.headers,
            body: self.body,
            path_params: Some(path_params),
            query_params: self.query_params,
        }
    }

    pub fn query_params(self, query_params: HashMap<String, String>) -> RequestBuilder<N, M, U> {
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: self.headers,
            body: self.body,
            path_params: self.path_params,
            query_params: Some(query_params),
        }
    }

    pub fn query_param(self, key: String, value: String) -> RequestBuilder<N, M, U> {
        let query_params = if let Some(mut params) = self.query_params {
            params.insert(key, value);
            params
        } else {
            let mut params = HashMap::new();
            params.insert(key, value);
            params
        };
        RequestBuilder::<N, M, U> {
            name: self.name,
            method: self.method,
            url: self.url,
            protocol: self.protocol,
            headers: self.headers,
            body: self.body,
            path_params: self.path_params,
            query_params: Some(query_params),
        }
    }
}

#[allow(dead_code)]
impl RequestBuilder<Name, HasMethod, Url> {
    pub fn build(self) -> Request {
        Request {
            name: self.name.0,
            protocol: self.protocol,
            url: self.url.0,
            method: self.method.0,
            headers: self.headers.unwrap_or_default(),
            body: self.body,
            path_params: self.path_params,
            query_params: self.query_params,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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

#[allow(dead_code)]
impl Request {
    pub fn builder() -> RequestBuilder<NoName, NoMethod, NoUrl> {
        RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        }
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

#[cfg(test)]
mod header {
    use super::*;
    use reqwest::header::HeaderMap;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub struct Header {
        name: Box<str>,
        value: Box<str>,
    }

    impl Header {
        pub fn fold(
            headers: Result<HeaderMap, Box<dyn std::error::Error>>,
            el: &Self,
        ) -> Result<HeaderMap, Box<dyn std::error::Error>> {
            use reqwest::header::{HeaderName, HeaderValue};
            if let Ok(mut headers) = headers {
                headers.append(
                    HeaderName::from_bytes(el.name.as_bytes())?,
                    HeaderValue::from_bytes(el.value.as_bytes())?,
                );
                Ok(headers)
            } else {
                Err("Invalid Headers".into())
            }
        }
    }

    #[test]
    fn test_header_creation() {
        let header = Header {
            name: "Content-Type".into(),
            value: "application/json".into(),
        };

        assert_eq!(header.name.as_ref(), "Content-Type");
        assert_eq!(header.value.as_ref(), "application/json");
    }

    #[test]
    fn test_fold_valid_headers() {
        let header = Header {
            name: "Content-Type".into(),
            value: "application/json".into(),
        };

        let headers = HeaderMap::new();
        let result = Header::fold(Ok(headers), &header);

        assert!(result.is_ok());
        let headers = result.unwrap();
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_fold_invalid_headers() {
        let header = Header {
            name: "Content-Type".into(),
            value: "application/json".into(),
        };

        let result = Header::fold(Err("Error".into()), &header);

        assert!(result.is_err());
    }

    #[test]
    fn test_fold_invalid_header_name() {
        let header = Header {
            name: "\0Content-Type".into(),
            value: "application/json".into(),
        };

        let headers = HeaderMap::new();
        let result = Header::fold(Ok(headers), &header);

        assert!(result.is_err());
    }

    #[test]
    fn test_fold_invalid_header_value() {
        let header = Header {
            name: "Content-Type".into(),
            value: "\0application/json".into(),
        };

        let headers = HeaderMap::new();
        let result = Header::fold(Ok(headers), &header);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod request_builder {
    use super::*;

    #[test]
    fn test_name() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let builder = builder.name("TestName".to_string());
        assert_eq!(builder.name, Name("TestName".to_string()));
    }

    #[test]
    fn test_method() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let builder = builder.method(Method::Get);
        assert_eq!(builder.method, HasMethod(Method::Get));
    }

    #[test]
    fn test_url() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let builder = builder.url("http://example.com".to_string());
        assert_eq!(builder.url, Url("http://example.com".to_string()));
    }

    #[test]
    fn test_protocol() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let builder = builder.protocol(Protocol::Http);
        assert_eq!(builder.protocol, Some(Protocol::Http));
    }

    #[test]
    fn test_headers() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let builder = builder.headers(vec![Header {
            name: "Content-Type".into(),
            value: "application/json".into(),
        }]);
        assert_eq!(
            builder.headers,
            Some(vec![Header {
                name: "Content-Type".into(),
                value: "application/json".into()
            }])
        );
    }

    #[test]
    fn test_body() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let builder = builder.body("body content".to_string());
        assert_eq!(builder.body, Some("body content".to_string()));
    }

    #[test]
    fn test_path_params() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let mut params = HashMap::new();
        params.insert("key".to_string(), "value".to_string());
        let builder = builder.path_params(params.clone());
        assert_eq!(builder.path_params, Some(params));
    }

    #[test]
    fn test_query_params() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let mut params = HashMap::new();
        params.insert("key".to_string(), "value".to_string());
        let builder = builder.query_params(params.clone());
        assert_eq!(builder.query_params, Some(params));
    }

    #[test]
    fn test_build() {
        let builder = RequestBuilder {
            name: NoName,
            method: NoMethod,
            url: NoUrl,
            protocol: None,
            headers: None,
            body: None,
            path_params: None,
            query_params: None,
        };

        let request = builder
            .name("TestName".to_string())
            .method(Method::Get)
            .url("http://example.com".to_string())
            .protocol(Protocol::Http)
            .headers(vec![Header {
                name: "Content-Type".into(),
                value: "application/json".into(),
            }])
            .body("body content".to_string())
            .path_params({
                let mut params = HashMap::new();
                params.insert("key".to_string(), "value".to_string());
                params
            })
            .query_params({
                let mut params = HashMap::new();
                params.insert("key".to_string(), "value".to_string());
                params
            })
            .build();

        assert_eq!(
            request,
            Request {
                name: "TestName".to_string(),
                protocol: Some(Protocol::Http),
                url: "http://example.com".to_string(),
                method: Method::Get,
                headers: vec![Header {
                    name: "Content-Type".into(),
                    value: "application/json".into()
                }],
                body: Some("body content".to_string()),
                path_params: Some({
                    let mut params = HashMap::new();
                    params.insert("key".to_string(), "value".to_string());
                    params
                }),
                query_params: Some({
                    let mut params = HashMap::new();
                    params.insert("key".to_string(), "value".to_string());
                    params
                }),
            }
        );
    }
}
