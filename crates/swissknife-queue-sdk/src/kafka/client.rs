use crate::{Error, Result};
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct KafkaClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    api_secret: Option<String>,
}

impl KafkaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: None,
            api_secret: None,
        }
    }

    pub fn with_auth(mut self, api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self.api_secret = Some(api_secret.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn auth_header(&self) -> Option<String> {
        match (&self.api_key, &self.api_secret) {
            (Some(key), Some(secret)) => {
                let credentials = format!("{}:{}", key, secret);
                let encoded = base64_encode(&credentials);
                Some(format!("Basic {}", encoded))
            }
            _ => None,
        }
    }

    pub(crate) async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        request = request.header("Content-Type", "application/vnd.kafka.json.v2+json");
        request = request.header("Accept", "application/vnd.kafka.v2+json");

        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        if let Some(b) = body {
            request = request.json(b);
        }

        let response = request.send().await.map_err(|e| Error::Request {
            message: e.to_string(),
        })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: format!("HTTP {}: {}", status.as_u16(), error_text),
                code: Some(status.as_u16() as i32),
            });
        }

        response.json().await.map_err(|e| Error::Parse {
            message: e.to_string(),
        })
    }

    pub(crate) async fn request_no_response(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        request = request.header("Content-Type", "application/vnd.kafka.json.v2+json");

        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        if let Some(b) = body {
            request = request.json(b);
        }

        let response = request.send().await.map_err(|e| Error::Request {
            message: e.to_string(),
        })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: format!("HTTP {}: {}", status.as_u16(), error_text),
                code: Some(status.as_u16() as i32),
            });
        }

        Ok(())
    }
}

fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}
