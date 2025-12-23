use reqwest::Client;

pub struct StagehandClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl StagehandClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            base_url: "https://api.browserbase.com".to_string(),
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

    pub fn local(port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", port),
            api_key: String::new(),
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
