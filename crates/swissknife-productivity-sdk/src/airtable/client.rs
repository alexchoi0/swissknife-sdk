use reqwest::Client;

const AIRTABLE_API_URL: &str = "https://api.airtable.com/v0";

pub struct AirtableClient {
    api_key: String,
    client: Client,
}

impl AirtableClient {
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
        AIRTABLE_API_URL
    }
}
