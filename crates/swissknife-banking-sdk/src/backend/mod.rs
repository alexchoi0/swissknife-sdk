use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

pub mod reqwest_backend;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "mock")]
pub mod fake;

pub use reqwest_backend::ReqwestBackend;

#[cfg(feature = "mock")]
pub use mock::MockBackend;

#[cfg(feature = "mock")]
pub use fake::{FakeBackend, Provider};

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
        }
    }
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::Get),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            "PATCH" => Some(HttpMethod::Patch),
            "DELETE" => Some(HttpMethod::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn ok(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    pub fn created(body: impl Into<String>) -> Self {
        Self {
            status: 201,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    pub fn no_content() -> Self {
        Self {
            status: 204,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn error(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.body)
    }
}

#[async_trait]
pub trait Backend: Send + Sync {
    async fn execute(&self, request: HttpRequest) -> crate::Result<HttpResponse>;

    async fn get(&self, url: &str) -> crate::Result<HttpResponse> {
        self.execute(HttpRequest {
            method: HttpMethod::Get,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        })
        .await
    }

    async fn get_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> crate::Result<HttpResponse> {
        self.execute(HttpRequest {
            method: HttpMethod::Get,
            url: url.to_string(),
            headers,
            body: None,
        })
        .await
    }

    async fn post<T: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> crate::Result<HttpResponse> {
        let body_json = serde_json::to_string(body)
            .map_err(|e| crate::Error::Json(e))?;

        self.execute(HttpRequest {
            method: HttpMethod::Post,
            url: url.to_string(),
            headers: HashMap::new(),
            body: Some(body_json),
        })
        .await
    }

    async fn post_with_headers<T: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &T,
        headers: HashMap<String, String>,
    ) -> crate::Result<HttpResponse> {
        let body_json = serde_json::to_string(body)
            .map_err(|e| crate::Error::Json(e))?;

        self.execute(HttpRequest {
            method: HttpMethod::Post,
            url: url.to_string(),
            headers,
            body: Some(body_json),
        })
        .await
    }

    async fn delete(&self, url: &str) -> crate::Result<HttpResponse> {
        self.execute(HttpRequest {
            method: HttpMethod::Delete,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        })
        .await
    }

    async fn delete_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> crate::Result<HttpResponse> {
        self.execute(HttpRequest {
            method: HttpMethod::Delete,
            url: url.to_string(),
            headers,
            body: None,
        })
        .await
    }
}

pub struct BackendConfig {
    pub base_url: String,
    pub default_headers: HashMap<String, String>,
}

impl BackendConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            default_headers: HashMap::new(),
        }
    }

    pub fn with_bearer_auth(mut self, token: &str) -> Self {
        self.default_headers
            .insert("Authorization".to_string(), format!("Bearer {}", token));
        self
    }

    pub fn with_basic_auth(mut self, username: &str, password: &str) -> Self {
        let credentials = format!("{}:{}", username, password);
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            credentials.as_bytes(),
        );
        self.default_headers
            .insert("Authorization".to_string(), format!("Basic {}", encoded));
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }
}
