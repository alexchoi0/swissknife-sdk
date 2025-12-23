use reqwest::Client;

const INCIDENT_IO_API_URL: &str = "https://api.incident.io/v2";

pub struct IncidentIoClient {
    api_key: String,
    client: Client,
}

impl IncidentIoClient {
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
        INCIDENT_IO_API_URL
    }
}
