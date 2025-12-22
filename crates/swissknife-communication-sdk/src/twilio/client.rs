use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;

use crate::{Error, Result};

const TWILIO_API_BASE: &str = "https://api.twilio.com/2010-04-01";

pub struct TwilioClient {
    pub(crate) http: Client,
    pub(crate) account_sid: String,
    pub(crate) auth_header: String,
}

impl TwilioClient {
    pub fn new(account_sid: impl Into<String>, auth_token: impl Into<String>) -> Result<Self> {
        let account_sid = account_sid.into();
        let auth_token = auth_token.into();

        if account_sid.is_empty() || auth_token.is_empty() {
            return Err(Error::Config(
                "Account SID and Auth Token are required".into(),
            ));
        }

        let credentials = format!("{}:{}", account_sid, auth_token);
        let auth_header = format!("Basic {}", STANDARD.encode(credentials));

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http,
            account_sid,
            auth_header,
        })
    }

    pub(crate) fn messages_url(&self) -> String {
        format!(
            "{}/Accounts/{}/Messages.json",
            TWILIO_API_BASE, self.account_sid
        )
    }

    pub(crate) fn calls_url(&self) -> String {
        format!(
            "{}/Accounts/{}/Calls.json",
            TWILIO_API_BASE, self.account_sid
        )
    }
}
