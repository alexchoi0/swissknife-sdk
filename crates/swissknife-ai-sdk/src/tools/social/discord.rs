use crate::error::Result;
use crate::tool::{get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct DiscordSendMessageTool;

impl Default for DiscordSendMessageTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for DiscordSendMessageTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "discord_send_message",
            "Discord Send Message",
            "Send a message to a Discord channel via webhook",
            "social",
        )
        .with_param("webhook_url", ParameterSchema::string("Discord webhook URL").required().user_only())
        .with_param("content", ParameterSchema::string("Message content").required())
        .with_param("username", ParameterSchema::string("Bot username to display"))
        .with_param("avatar_url", ParameterSchema::string("Avatar URL for the bot"))
        .with_output("success", OutputSchema::boolean("Whether the message was sent"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let webhook_url = get_required_string_param(&params, "webhook_url")?;
        let content = get_required_string_param(&params, "content")?;
        let username = get_string_param(&params, "username");
        let avatar_url = get_string_param(&params, "avatar_url");

        let client = reqwest::Client::new();

        let mut payload = serde_json::json!({ "content": content });

        if let Some(u) = username {
            payload["username"] = serde_json::json!(u);
        }
        if let Some(a) = avatar_url {
            payload["avatar_url"] = serde_json::json!(a);
        }

        match client.post(&webhook_url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() || resp.status() == 204 => {
                Ok(ToolResponse::success(serde_json::json!({
                    "success": true
                })))
            }
            Ok(resp) => Ok(ToolResponse::error(format!("Discord API error: {}", resp.status()))),
            Err(e) => Ok(ToolResponse::error(format!("Request failed: {}", e))),
        }
    }
}
