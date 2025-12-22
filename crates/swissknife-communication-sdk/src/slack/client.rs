use reqwest::Client;

use crate::{Error, Result};

const SLACK_API_BASE: &str = "https://slack.com/api";

pub struct SlackClient {
    pub(crate) http: Client,
    pub(crate) bot_token: String,
}

impl SlackClient {
    pub fn new(bot_token: impl Into<String>) -> Result<Self> {
        let bot_token = bot_token.into();

        if bot_token.is_empty() {
            return Err(Error::Config("Bot token is required".into()));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { http, bot_token })
    }

    pub fn from_webhook(webhook_url: impl Into<String>) -> Result<SlackWebhookClient> {
        SlackWebhookClient::new(webhook_url)
    }

    pub(crate) fn post_message_url(&self) -> String {
        format!("{}/chat.postMessage", SLACK_API_BASE)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.bot_token)
    }
}

pub struct SlackWebhookClient {
    pub(crate) http: Client,
    pub(crate) webhook_url: String,
}

impl SlackWebhookClient {
    pub fn new(webhook_url: impl Into<String>) -> Result<Self> {
        let webhook_url = webhook_url.into();

        if webhook_url.is_empty() {
            return Err(Error::Config("Webhook URL is required".into()));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { http, webhook_url })
    }
}
