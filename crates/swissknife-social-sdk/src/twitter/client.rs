use reqwest::Client;

use crate::{Error, Result};

const TWITTER_API_BASE: &str = "https://api.twitter.com/2";

pub struct TwitterClient {
    pub(crate) http: Client,
    pub(crate) bearer_token: String,
}

impl TwitterClient {
    pub fn new(bearer_token: impl Into<String>) -> Result<Self> {
        let bearer_token = bearer_token.into();

        if bearer_token.is_empty() {
            return Err(Error::Config("Bearer token is required".into()));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { http, bearer_token })
    }

    pub(crate) fn tweets_url(&self) -> String {
        format!("{}/tweets", TWITTER_API_BASE)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.bearer_token)
    }
}
