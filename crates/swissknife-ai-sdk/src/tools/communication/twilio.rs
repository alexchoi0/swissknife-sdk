use crate::error::Result;
use crate::tool::{get_required_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_communication_sdk::twilio::TwilioClient;

#[cfg(feature = "sms")]
use swissknife_communication_sdk::sms::SmsSender;

pub struct TwilioSendSmsTool;

impl Default for TwilioSendSmsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TwilioSendSmsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "twilio_send_sms",
            "Twilio Send SMS",
            "Send an SMS message using Twilio",
            "communication",
        )
        .with_param("account_sid", ParameterSchema::string("Twilio Account SID").required().user_only())
        .with_param("auth_token", ParameterSchema::string("Twilio Auth Token").required().user_only())
        .with_param("from", ParameterSchema::string("Sender phone number (E.164 format)").required())
        .with_param("to", ParameterSchema::string("Recipient phone number (E.164 format)").required())
        .with_param("body", ParameterSchema::string("Message body").required())
        .with_output("message_id", OutputSchema::string("The message SID"))
        .with_output("status", OutputSchema::string("Message status"))
    }

    #[cfg(feature = "sms")]
    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let account_sid = get_required_string_param(&params, "account_sid")?;
        let auth_token = get_required_string_param(&params, "auth_token")?;
        let from = get_required_string_param(&params, "from")?;
        let to = get_required_string_param(&params, "to")?;
        let body = get_required_string_param(&params, "body")?;

        let client = TwilioClient::new(account_sid, auth_token)
            .map_err(|e| crate::error::Error::Provider(e.to_string()))?;

        match client.send_sms(&from, &to, &body).await {
            Ok(response) => Ok(ToolResponse::success(serde_json::json!({
                "message_id": response.message_id,
                "status": response.status,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to send SMS: {}", e))),
        }
    }

    #[cfg(not(feature = "sms"))]
    async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        Ok(ToolResponse::error("SMS feature not enabled"))
    }
}
