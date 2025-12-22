use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::chat::{ChatMessage, ChatResponse, ChatSender};
use crate::{Error, Result};

use super::TelegramClient;

#[derive(Serialize)]
struct TelegramSendMessage {
    chat_id: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
}

#[derive(Deserialize)]
struct TelegramResponse {
    ok: bool,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    result: Option<TelegramMessage>,
}

#[derive(Deserialize)]
struct TelegramMessage {
    message_id: i64,
    date: i64,
    chat: TelegramChat,
}

#[derive(Deserialize)]
struct TelegramChat {
    id: i64,
}

#[async_trait]
impl ChatSender for TelegramClient {
    async fn send_message(
        &self,
        channel: &str,
        message: &ChatMessage,
    ) -> Result<ChatResponse> {
        let payload = TelegramSendMessage {
            chat_id: channel.to_string(),
            text: message.text.clone(),
            parse_mode: None,
        };

        let response = self
            .http
            .post(&self.send_message_url())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let result: TelegramResponse = response.json().await?;

        if !result.ok {
            return Err(Error::Api {
                code: 400,
                message: result.description.unwrap_or_else(|| "Unknown error".into()),
            });
        }

        let msg = result.result.ok_or_else(|| Error::Api {
            code: 500,
            message: "No message in response".into(),
        })?;

        Ok(ChatResponse {
            message_id: Some(msg.message_id.to_string()),
            channel: Some(msg.chat.id.to_string()),
            timestamp: Some(msg.date.to_string()),
        })
    }
}
