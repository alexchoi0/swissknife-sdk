use crate::error::Result;
use crate::tool::{get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct SlackSendMessageTool;

impl Default for SlackSendMessageTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SlackSendMessageTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "slack_send_message",
            "Slack Send Message",
            "Send a message to a Slack channel",
            "social",
        )
        .with_param("webhook_url", ParameterSchema::string("Slack webhook URL").required().user_only())
        .with_param("text", ParameterSchema::string("Message text").required())
        .with_param("username", ParameterSchema::string("Bot username to display"))
        .with_param("icon_emoji", ParameterSchema::string("Icon emoji (e.g., :robot_face:)"))
        .with_output("success", OutputSchema::boolean("Whether the message was sent"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let webhook_url = get_required_string_param(&params, "webhook_url")?;
        let text = get_required_string_param(&params, "text")?;
        let username = get_string_param(&params, "username");
        let icon_emoji = get_string_param(&params, "icon_emoji");

        let client = reqwest::Client::new();

        let mut payload = serde_json::json!({ "text": text });

        if let Some(u) = username {
            payload["username"] = serde_json::json!(u);
        }
        if let Some(e) = icon_emoji {
            payload["icon_emoji"] = serde_json::json!(e);
        }

        match client.post(&webhook_url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() => Ok(ToolResponse::success(serde_json::json!({
                "success": true
            }))),
            Ok(resp) => Ok(ToolResponse::error(format!("Slack API error: {}", resp.status()))),
            Err(e) => Ok(ToolResponse::error(format!("Request failed: {}", e))),
        }
    }
}

pub struct SlackListChannelsTool;

impl Default for SlackListChannelsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SlackListChannelsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "slack_list_channels",
            "Slack List Channels",
            "List channels in a Slack workspace",
            "social",
        )
        .with_param("access_token", ParameterSchema::string("Slack Bot OAuth token").required().user_only())
        .with_output("channels", OutputSchema::array("List of channels", OutputSchema::json("Channel object")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_token = get_required_string_param(&params, "access_token")?;

        let client = reqwest::Client::new();

        match client
            .get("https://slack.com/api/conversations.list")
            .bearer_auth(&access_token)
            .send()
            .await
        {
            Ok(resp) => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                    Ok(ToolResponse::success(serde_json::json!({
                        "channels": body.get("channels").cloned().unwrap_or(serde_json::json!([])),
                        "success": true
                    })))
                } else {
                    let error = body.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error");
                    Ok(ToolResponse::error(format!("Slack API error: {}", error)))
                }
            }
            Err(e) => Ok(ToolResponse::error(format!("Request failed: {}", e))),
        }
    }
}
