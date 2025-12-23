use crate::error::Result;
use crate::tool::{get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_communication_sdk::sendgrid::SendGridClient;

#[cfg(feature = "email")]
use swissknife_communication_sdk::email::{Email, EmailAddress, EmailSender};

pub struct SendGridSendEmailTool;

impl Default for SendGridSendEmailTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SendGridSendEmailTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "sendgrid_send_email",
            "SendGrid Send Email",
            "Send an email using SendGrid",
            "communication",
        )
        .with_param("api_key", ParameterSchema::string("SendGrid API key").required().user_only())
        .with_param("from_email", ParameterSchema::string("Sender email address").required())
        .with_param("from_name", ParameterSchema::string("Sender name"))
        .with_param("to_email", ParameterSchema::string("Recipient email address").required())
        .with_param("to_name", ParameterSchema::string("Recipient name"))
        .with_param("subject", ParameterSchema::string("Email subject").required())
        .with_param("text", ParameterSchema::string("Plain text content"))
        .with_param("html", ParameterSchema::string("HTML content"))
        .with_output("message_id", OutputSchema::string("The message ID").optional())
        .with_output("status", OutputSchema::string("Send status"))
    }

    #[cfg(feature = "email")]
    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let from_email = get_required_string_param(&params, "from_email")?;
        let from_name = get_string_param(&params, "from_name");
        let to_email = get_required_string_param(&params, "to_email")?;
        let to_name = get_string_param(&params, "to_name");
        let subject = get_required_string_param(&params, "subject")?;
        let text = get_string_param(&params, "text");
        let html = get_string_param(&params, "html");

        if text.is_none() && html.is_none() {
            return Ok(ToolResponse::error("Either text or html content is required"));
        }

        let client = SendGridClient::new(api_key)
            .map_err(|e| crate::error::Error::Provider(e.to_string()))?;

        let from = if let Some(name) = from_name {
            EmailAddress::with_name(from_email, name)
        } else {
            EmailAddress::new(from_email)
        };

        let to = if let Some(name) = to_name {
            EmailAddress::with_name(to_email, name)
        } else {
            EmailAddress::new(to_email)
        };

        let mut email = Email::new(from, to, subject);
        if let Some(t) = text {
            email = email.text(t);
        }
        if let Some(h) = html {
            email = email.html(h);
        }

        match client.send_email(&email).await {
            Ok(response) => Ok(ToolResponse::success(serde_json::json!({
                "message_id": response.message_id,
                "status": response.status,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to send email: {}", e))),
        }
    }

    #[cfg(not(feature = "email"))]
    async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        Ok(ToolResponse::error("Email feature not enabled"))
    }
}
