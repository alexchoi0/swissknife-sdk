use reqwest::Client;

const BASE_URL: &str = "https://oauth.reddit.com";
const AUTH_URL: &str = "https://www.reddit.com/api/v1/access_token";

pub struct RedditClient {
    access_token: String,
    client: Client,
    user_agent: String,
}

impl RedditClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
            client: Client::new(),
            user_agent: "swissknife-sdk/0.1.0".to_string(),
        }
    }

    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn access_token(&self) -> &str {
        &self.access_token
    }

    pub(crate) fn base_url(&self) -> &str {
        BASE_URL
    }

    pub(crate) fn user_agent(&self) -> &str {
        &self.user_agent
    }
}
