use reqwest::Client;

const BASE_URL: &str = "https://api.resend.com";

pub struct ResendClient {
    api_key: String,
    client: Client,
}

impl ResendClient {
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
        BASE_URL
    }
}
