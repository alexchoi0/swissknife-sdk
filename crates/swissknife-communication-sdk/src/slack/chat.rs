use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::chat::{ChatMessage, ChatResponse, ChatSender};
use crate::{Error, Result};

use super::client::{SlackClient, SlackWebhookClient};

#[derive(Serialize)]
struct SlackPostMessage {
    channel: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_emoji: Option<String>,
}

#[derive(Serialize)]
struct SlackWebhookMessage {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_emoji: Option<String>,
}

#[derive(Deserialize)]
struct SlackResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    ts: Option<String>,
}

#[async_trait]
impl ChatSender for SlackClient {
    async fn send_message(
        &self,
        channel: &str,
        message: &ChatMessage,
    ) -> Result<ChatResponse> {
        let payload = SlackPostMessage {
            channel: channel.to_string(),
            text: message.text.clone(),
            username: message.username.clone(),
            icon_url: message.icon_url.clone(),
            icon_emoji: message.icon_emoji.clone(),
        };

        let response = self
            .http
            .post(&self.post_message_url())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: SlackResponse = response.json().await?;

        if !result.ok {
            return Err(Error::Api {
                code: 400,
                message: result.error.unwrap_or_else(|| "Unknown error".into()),
            });
        }

        Ok(ChatResponse {
            message_id: result.ts.clone(),
            channel: result.channel,
            timestamp: result.ts,
        })
    }
}

#[async_trait]
impl ChatSender for SlackWebhookClient {
    async fn send_message(
        &self,
        _channel: &str,
        message: &ChatMessage,
    ) -> Result<ChatResponse> {
        let payload = SlackWebhookMessage {
            text: message.text.clone(),
            username: message.username.clone(),
            icon_url: message.icon_url.clone(),
            icon_emoji: message.icon_emoji.clone(),
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
