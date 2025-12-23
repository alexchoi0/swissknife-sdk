use reqwest::Client;

const POLYMARKET_API_URL: &str = "https://clob.polymarket.com";
const POLYMARKET_GAMMA_API_URL: &str = "https://gamma-api.polymarket.com";

pub struct PolymarketClient {
    clob_url: String,
    gamma_url: String,
    api_key: Option<String>,
    api_secret: Option<String>,
    api_passphrase: Option<String>,
    client: Client,
}

impl PolymarketClient {
    pub fn new() -> Self {
        Self {
            clob_url: POLYMARKET_API_URL.to_string(),
            gamma_url: POLYMARKET_GAMMA_API_URL.to_string(),
            api_key: None,
            api_secret: None,
            api_passphrase: None,
            client: Client::new(),
        }
    }

    pub fn with_credentials(api_key: &str, api_secret: &str, api_passphrase: &str) -> Self {
        Self {
            clob_url: POLYMARKET_API_URL.to_string(),
            gamma_url: POLYMARKET_GAMMA_API_URL.to_string(),
            api_key: Some(api_key.to_string()),
            api_secret: Some(api_secret.to_string()),
            api_passphrase: Some(api_passphrase.to_string()),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn clob_url(&self) -> &str {
        &self.clob_url
    }

    pub(crate) fn gamma_url(&self) -> &str {
        &self.gamma_url
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    pub(crate) fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    pub(crate) fn api_secret(&self) -> Option<&str> {
        self.api_secret.as_deref()
    }

    pub(crate) fn api_passphrase(&self) -> Option<&str> {
        self.api_passphrase.as_deref()
    }
}

impl Default for PolymarketClient {
    fn default() -> Self {
        Self::new()
    }
}
