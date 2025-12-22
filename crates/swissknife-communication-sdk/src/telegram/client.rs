use reqwest::Client;

use crate::{Error, Result};

const TELEGRAM_API_BASE: &str = "https://api.telegram.org";

pub struct TelegramClient {
    pub(crate) http: Client,
    pub(crate) bot_token: String,
}

impl TelegramClient {
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

    pub(crate) fn send_message_url(&self) -> String {
        format!("{}/bot{}/sendMessage", TELEGRAM_API_BASE, self.bot_token)
    }
}
