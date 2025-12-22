use reqwest::Client;

use crate::{Error, Result};

const APNS_PRODUCTION: &str = "https://api.push.apple.com";
const APNS_SANDBOX: &str = "https://api.sandbox.push.apple.com";

pub struct ApnsClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) token: String,
    #[allow(dead_code)]
    pub(crate) team_id: String,
    pub(crate) bundle_id: String,
}

impl ApnsClient {
    pub fn new(
        token: impl Into<String>,
        team_id: impl Into<String>,
        bundle_id: impl Into<String>,
        sandbox: bool,
    ) -> Result<Self> {
        let token = token.into();
        let team_id = team_id.into();
        let bundle_id = bundle_id.into();

        if token.is_empty() || team_id.is_empty() || bundle_id.is_empty() {
            return Err(Error::Config(
                "Token, team ID, and bundle ID are required".into(),
            ));
        }

        let http = Client::builder()
            .http2_prior_knowledge()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP/2 client: {}", e)))?;

        let base_url = if sandbox {
            APNS_SANDBOX.to_string()
        } else {
            APNS_PRODUCTION.to_string()
        };

        Ok(Self {
            http,
            base_url,
            token,
            team_id,
            bundle_id,
        })
    }

    pub fn production(
        token: impl Into<String>,
        team_id: impl Into<String>,
        bundle_id: impl Into<String>,
    ) -> Result<Self> {
        Self::new(token, team_id, bundle_id, false)
    }

    pub fn sandbox(
        token: impl Into<String>,
        team_id: impl Into<String>,
        bundle_id: impl Into<String>,
    ) -> Result<Self> {
        Self::new(token, team_id, bundle_id, true)
    }

    pub(crate) fn device_url(&self, device_token: &str) -> String {
        format!("{}/3/device/{}", self.base_url, device_token)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("bearer {}", self.token)
    }
}
