use reqwest::Client;

const BASE_URL: &str = "https://www.googleapis.com/youtube/v3";

pub struct YouTubeClient {
    api_key: Option<String>,
    access_token: Option<String>,
    client: Client,
}

impl YouTubeClient {
    pub fn with_api_key(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_string()),
            access_token: None,
            client: Client::new(),
        }
    }

    pub fn with_oauth(access_token: &str) -> Self {
        Self {
            api_key: None,
            access_token: Some(access_token.to_string()),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    pub(crate) fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    pub(crate) fn base_url(&self) -> &str {
        BASE_URL
    }

    pub(crate) fn add_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.access_token {
            request.header("Authorization", format!("Bearer {}", token))
        } else if let Some(key) = &self.api_key {
            request.query(&[("key", key)])
        } else {
            request
        }
    }
}
