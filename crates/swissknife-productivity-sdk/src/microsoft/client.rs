use reqwest::Client;

const GRAPH_API_URL: &str = "https://graph.microsoft.com/v1.0";

pub struct MicrosoftClient {
    access_token: String,
    client: Client,
}

impl MicrosoftClient {
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
        GRAPH_API_URL
    }
}
