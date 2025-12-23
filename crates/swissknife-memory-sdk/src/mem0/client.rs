use reqwest::Client;

const MEM0_API_URL: &str = "https://api.mem0.ai/v1";

pub struct Mem0Client {
    api_key: String,
    base_url: String,
    client: Client,
}

impl Mem0Client {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: MEM0_API_URL.to_string(),
            client: Client::new(),
        }
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }
}
