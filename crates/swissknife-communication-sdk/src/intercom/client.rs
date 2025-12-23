use reqwest::Client;

const BASE_URL: &str = "https://api.intercom.io";

pub struct IntercomClient {
    access_token: String,
    client: Client,
}

impl IntercomClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn access_token(&self) -> &str {
        &self.access_token
    }

    pub(crate) fn base_url(&self) -> &str {
        BASE_URL
    }
}
