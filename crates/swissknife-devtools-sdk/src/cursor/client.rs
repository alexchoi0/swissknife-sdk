use reqwest::Client;

pub struct CursorClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl CursorClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            base_url: "http://localhost:9000".to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }
}
