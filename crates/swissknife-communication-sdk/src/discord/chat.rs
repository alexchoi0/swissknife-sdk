use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::chat::{ChatMessage, ChatResponse, ChatSender};
use crate::{Error, Result};

use super::client::{DiscordBotClient, DiscordWebhookClient};

#[derive(Serialize)]
struct DiscordMessage {
    content: String,
}

#[derive(Serialize)]
struct DiscordWebhookMessage {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct DiscordMessageResponse {
    id: String,
    channel_id: String,
    timestamp: String,
}

#[async_trait]
impl ChatSender for DiscordBotClient {
    async fn send_message(
        &self,
        channel: &str,
        message: &ChatMessage,
    ) -> Result<ChatResponse> {
        let payload = DiscordMessage {
            content: message.text.clone(),
        };

        let response = self
            .http
            .post(&self.channel_message_url(channel))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status_code = response.status();

        if !status_code.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                code: status_code.as_u16() as i32,
                message: error_text,
            });
        }

        let result: DiscordMessageResponse = response.json().await?;

        Ok(ChatResponse {
            message_id: Some(result.id),
            channel: Some(result.channel_id),
            timestamp: Some(result.timestamp),
        })
    }
}

#[async_trait]
impl ChatSender for DiscordWebhookClient {
    async fn send_message(
        &self,
        _channel: &str,
        message: &ChatMessage,
    ) -> Result<ChatResponse> {
        let payload = DiscordWebhookMessage {
            content: message.text.clone(),
            username: message.username.clone(),
            avatar_url: message.icon_url.clone(),
        };

        let response = self
            .http
            .post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status_code = response.status();

        if !status_code.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                code: status_code.as_u16() as i32,
                message: error_text,
            });
        }

        Ok(ChatResponse {
            message_id: None,
            channel: None,
            timestamp: None,
        })
    }
}
