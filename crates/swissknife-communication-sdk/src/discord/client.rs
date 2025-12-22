use reqwest::Client;

use crate::{Error, Result};

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

pub struct DiscordBotClient {
    pub(crate) http: Client,
    pub(crate) bot_token: String,
}

impl DiscordBotClient {
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

    pub(crate) fn channel_message_url(&self, channel_id: &str) -> String {
        format!("{}/channels/{}/messages", DISCORD_API_BASE, channel_id)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bot {}", self.bot_token)
    }
}

pub struct DiscordWebhookClient {
    pub(crate) http: Client,
    pub(crate) webhook_url: String,
}

impl DiscordWebhookClient {
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
