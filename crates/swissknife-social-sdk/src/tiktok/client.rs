use reqwest::Client;

use crate::{Error, Result};

const TIKTOK_API_BASE: &str = "https://open.tiktokapis.com/v2";

pub struct TikTokClient {
    pub(crate) http: Client,
    pub(crate) access_token: String,
}

impl TikTokClient {
    pub fn new(access_token: impl Into<String>) -> Result<Self> {
        let access_token = access_token.into();

        if access_token.is_empty() {
            return Err(Error::Config("Access token is required".into()));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { http, access_token })
    }

    pub(crate) fn post_publish_url(&self) -> String {
        format!("{}/post/publish/inbox/video/init/", TIKTOK_API_BASE)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }
}
