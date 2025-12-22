use reqwest::Client;

use crate::{Error, Result};

const GRAPH_API_BASE: &str = "https://graph.facebook.com/v18.0";

pub struct FacebookClient {
    pub(crate) http: Client,
    pub(crate) access_token: String,
    pub(crate) page_id: String,
}

impl FacebookClient {
    pub fn new(
        access_token: impl Into<String>,
        page_id: impl Into<String>,
    ) -> Result<Self> {
        let access_token = access_token.into();
        let page_id = page_id.into();

        if access_token.is_empty() || page_id.is_empty() {
            return Err(Error::Config(
                "Access token and Page ID are required".into(),
            ));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http,
            access_token,
            page_id,
        })
    }

    pub(crate) fn feed_url(&self) -> String {
        format!("{}/{}/feed", GRAPH_API_BASE, self.page_id)
    }

    pub(crate) fn photos_url(&self) -> String {
        format!("{}/{}/photos", GRAPH_API_BASE, self.page_id)
    }

    pub(crate) fn videos_url(&self) -> String {
        format!("{}/{}/videos", GRAPH_API_BASE, self.page_id)
    }
}
