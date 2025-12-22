use reqwest::Client;

use crate::{Error, Result};

const SENDGRID_API_BASE: &str = "https://api.sendgrid.com/v3";

pub struct SendGridClient {
    pub(crate) http: Client,
    pub(crate) api_key: String,
}

impl SendGridClient {
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let api_key = api_key.into();

        if api_key.is_empty() {
            return Err(Error::Config("API key is required".into()));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { http, api_key })
    }

    pub(crate) fn mail_send_url(&self) -> String {
        format!("{}/mail/send", SENDGRID_API_BASE)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }
}
