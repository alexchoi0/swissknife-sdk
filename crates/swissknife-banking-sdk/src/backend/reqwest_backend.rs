use super::{Backend, BackendConfig, HttpMethod, HttpRequest, HttpResponse};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

pub struct ReqwestBackend {
    client: Client,
    config: BackendConfig,
}

impl ReqwestBackend {
    pub fn new(config: BackendConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn with_client(client: Client, config: BackendConfig) -> Self {
        Self { client, config }
    }

    fn build_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("{}{}", self.config.base_url.trim_end_matches('/'), url)
        }
    }

    fn merge_headers(&self, request_headers: &HashMap<String, String>) -> HashMap<String, String> {
        let mut headers = self.config.default_headers.clone();
        headers.extend(request_headers.clone());
        headers
    }
}

#[async_trait]
impl Backend for ReqwestBackend {
    async fn execute(&self, request: HttpRequest) -> crate::Result<HttpResponse> {
        let url = self.build_url(&request.url);
        let headers = self.merge_headers(&request.headers);

        let mut req_builder = match request.method {
            HttpMethod::Get => self.client.get(&url),
            HttpMethod::Post => self.client.post(&url),
            HttpMethod::Put => self.client.put(&url),
            HttpMethod::Patch => self.client.patch(&url),
            HttpMethod::Delete => self.client.delete(&url),
        };

        for (key, value) in headers {
            req_builder = req_builder.header(&key, &value);
        }

        if !request.headers.contains_key("Content-Type")
            && !self.config.default_headers.contains_key("Content-Type")
        {
            req_builder = req_builder.header("Content-Type", "application/json");
        }

        if let Some(body) = request.body {
            req_builder = req_builder.body(body);
        }

        let response = req_builder.send().await?;

        let status = response.status().as_u16();
        let mut response_headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                response_headers.insert(key.to_string(), v.to_string());
            }
        }

        let body = response.text().await?;

        Ok(HttpResponse {
            status,
            headers: response_headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_absolute() {
        let backend = ReqwestBackend::new(BackendConfig::new("https://api.example.com"));
        assert_eq!(
            backend.build_url("https://other.com/path"),
            "https://other.com/path"
        );
    }

    #[test]
    fn test_build_url_relative() {
        let backend = ReqwestBackend::new(BackendConfig::new("https://api.example.com"));
        assert_eq!(
            backend.build_url("/users/123"),
            "https://api.example.com/users/123"
        );
    }

    #[test]
    fn test_build_url_relative_no_slash() {
        let backend = ReqwestBackend::new(BackendConfig::new("https://api.example.com/"));
        assert_eq!(
            backend.build_url("/users/123"),
            "https://api.example.com/users/123"
        );
    }
}
