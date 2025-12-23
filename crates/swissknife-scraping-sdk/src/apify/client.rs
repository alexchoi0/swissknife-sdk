use reqwest::Client;

const APIFY_API_URL: &str = "https://api.apify.com/v2";

pub struct ApifyClient {
    api_token: String,
    client: Client,
}

impl ApifyClient {
    pub fn new(api_token: &str) -> Self {
        Self {
            api_token: api_token.to_string(),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        APIFY_API_URL
    }

    pub(crate) fn api_token(&self) -> &str {
        &self.api_token
    }
}
