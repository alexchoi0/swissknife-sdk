use reqwest::Client;

const APOLLO_API_URL: &str = "https://api.apollo.io/v1";

pub struct ApolloClient {
    api_key: String,
    client: Client,
}

impl ApolloClient {
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
        APOLLO_API_URL
    }
}
