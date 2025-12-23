use reqwest::Client;

const CLAY_API_URL: &str = "https://api.clay.com/v1";

pub struct ClayClient {
    api_key: String,
    client: Client,
}

impl ClayClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    pub(crate) fn base_url(&self) -> &str {
        CLAY_API_URL
    }
}
