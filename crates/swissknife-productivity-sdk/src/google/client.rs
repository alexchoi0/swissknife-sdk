use reqwest::Client;

pub struct GoogleClient {
    access_token: String,
    client: Client,
}

impl GoogleClient {
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
}
