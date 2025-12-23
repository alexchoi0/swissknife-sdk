use reqwest::Client;

const TYPEFORM_API_URL: &str = "https://api.typeform.com";

pub struct TypeformClient {
    access_token: String,
    client: Client,
}

impl TypeformClient {
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
        TYPEFORM_API_URL
    }
}
