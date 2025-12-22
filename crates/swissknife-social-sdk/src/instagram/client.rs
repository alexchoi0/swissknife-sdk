use reqwest::Client;

use crate::{Error, Result};

const GRAPH_API_BASE: &str = "https://graph.facebook.com/v18.0";

pub struct InstagramClient {
    pub(crate) http: Client,
    pub(crate) access_token: String,
    pub(crate) account_id: String,
}

impl InstagramClient {
    pub fn new(
        access_token: impl Into<String>,
        account_id: impl Into<String>,
    ) -> Result<Self> {
        let access_token = access_token.into();
        let account_id = account_id.into();

        if access_token.is_empty() || account_id.is_empty() {
            return Err(Error::Config(
                "Access token and Instagram account ID are required".into(),
            ));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http,
            access_token,
            account_id,
        })
    }

    pub(crate) fn media_url(&self) -> String {
        format!("{}/{}/media", GRAPH_API_BASE, self.account_id)
    }

    pub(crate) fn publish_url(&self) -> String {
        format!("{}/{}/media_publish", GRAPH_API_BASE, self.account_id)
    }
}
