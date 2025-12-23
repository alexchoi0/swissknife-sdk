use reqwest::Client;

const CALENDLY_API_URL: &str = "https://api.calendly.com";

pub struct CalendlyClient {
    access_token: String,
    client: Client,
}

impl CalendlyClient {
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
        CALENDLY_API_URL
    }
}
