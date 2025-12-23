use reqwest::Client;

pub struct ConfluenceClient {
    base_url: String,
    email: String,
    api_token: String,
    client: Client,
}

impl ConfluenceClient {
    pub fn new(base_url: &str, email: &str, api_token: &str) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            base_url,
            email: email.to_string(),
            api_token: api_token.to_string(),
            client: Client::new(),
        }
    }

    pub fn cloud(cloud_id: &str, email: &str, api_token: &str) -> Self {
        let base_url = format!("https://api.atlassian.com/ex/confluence/{}", cloud_id);
        Self::new(&base_url, email, api_token)
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn auth_header(&self) -> String {
        use base64::Engine;
        let credentials = format!("{}:{}", self.email, self.api_token);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        format!("Basic {}", encoded)
    }
}
