use reqwest::Client;

const KALSHI_API_URL: &str = "https://trading-api.kalshi.com/trade-api/v2";
const KALSHI_DEMO_API_URL: &str = "https://demo-api.kalshi.co/trade-api/v2";

pub struct KalshiClient {
    base_url: String,
    token: Option<String>,
    client: Client,
}

impl KalshiClient {
    pub fn new() -> Self {
        Self {
            base_url: KALSHI_API_URL.to_string(),
            token: None,
            client: Client::new(),
        }
    }

    pub fn demo() -> Self {
        Self {
            base_url: KALSHI_DEMO_API_URL.to_string(),
            token: None,
            client: Client::new(),
        }
    }

    pub fn with_token(token: &str) -> Self {
        Self {
            base_url: KALSHI_API_URL.to_string(),
            token: Some(token.to_string()),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = Some(token.to_string());
    }
}

impl Default for KalshiClient {
    fn default() -> Self {
        Self::new()
    }
}
