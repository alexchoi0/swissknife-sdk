use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "communication")]
use swissknife_communication_sdk as comm;

#[derive(Clone)]
pub struct CommunicationTools {
    #[cfg(feature = "slack")]
    pub slack: Option<comm::slack::SlackClient>,
    #[cfg(feature = "discord")]
    pub discord: Option<comm::discord::DiscordClient>,
    #[cfg(feature = "telegram")]
    pub telegram: Option<comm::telegram::TelegramClient>,
    #[cfg(feature = "sendgrid")]
    pub sendgrid: Option<comm::sendgrid::SendGridClient>,
    #[cfg(feature = "resend")]
    pub resend: Option<comm::resend::ResendClient>,
    #[cfg(feature = "twilio")]
    pub twilio: Option<comm::twilio::TwilioClient>,
}

impl CommunicationTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "slack")]
            slack: None,
            #[cfg(feature = "discord")]
            discord: None,
            #[cfg(feature = "telegram")]
            telegram: None,
            #[cfg(feature = "sendgrid")]
            sendgrid: None,
            #[cfg(feature = "resend")]
            resend: None,
            #[cfg(feature = "twilio")]
            twilio: None,
        }
    }

    #[cfg(feature = "slack")]
    pub fn with_slack(mut self, client: comm::slack::SlackClient) -> Self {
        self.slack = Some(client);
        self
    }

    #[cfg(feature = "discord")]
    pub fn with_discord(mut self, client: comm::discord::DiscordClient) -> Self {
        self.discord = Some(client);
        self
    }

    #[cfg(feature = "telegram")]
    pub fn with_telegram(mut self, client: comm::telegram::TelegramClient) -> Self {
        self.telegram = Some(client);
        self
    }

    #[cfg(feature = "sendgrid")]
    pub fn with_sendgrid(mut self, client: comm::sendgrid::SendGridClient) -> Self {
        self.sendgrid = Some(client);
        self
    }

    #[cfg(feature = "resend")]
    pub fn with_resend(mut self, client: comm::resend::ResendClient) -> Self {
        self.resend = Some(client);
        self
    }

    #[cfg(feature = "twilio")]
    pub fn with_twilio(mut self, client: comm::twilio::TwilioClient) -> Self {
        self.twilio = Some(client);
        self
    }
}

impl Default for CommunicationTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SlackMessageRequest {
    pub channel: String,
    pub text: String,
    #[serde(default)]
    pub thread_ts: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiscordMessageRequest {
    pub channel_id: String,
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TelegramMessageRequest {
    pub chat_id: String,
    pub text: String,
    #[serde(default)]
    pub parse_mode: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SendGridEmailRequest {
    pub to: String,
    pub from: String,
    pub subject: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResendEmailRequest {
    pub to: String,
    pub from: String,
    pub subject: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TwilioSmsRequest {
    pub to: String,
    pub from: String,
    pub body: String,
}

#[tool_router]
impl CommunicationTools {
    #[cfg(feature = "slack")]
    #[rmcp::tool(description = "Send a message to a Slack channel")]
    pub async fn slack_send_message(
        &self,
        #[rmcp::tool(aggr)] req: SlackMessageRequest,
    ) -> Result<String, String> {
        let client = self.slack.as_ref()
            .ok_or_else(|| "Slack client not configured".to_string())?;

        let request = comm::slack::PostMessageRequest {
            channel: req.channel,
            text: Some(req.text),
            blocks: None,
            thread_ts: req.thread_ts,
            reply_broadcast: None,
            unfurl_links: None,
            unfurl_media: None,
        };

        let response = client.post_message(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": response.ok,
            "channel": response.channel,
            "ts": response.ts
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "discord")]
    #[rmcp::tool(description = "Send a message to a Discord channel")]
    pub async fn discord_send_message(
        &self,
        #[rmcp::tool(aggr)] req: DiscordMessageRequest,
    ) -> Result<String, String> {
        let client = self.discord.as_ref()
            .ok_or_else(|| "Discord client not configured".to_string())?;

        let request = comm::discord::SendMessageRequest {
            content: Some(req.content),
            embeds: None,
            tts: None,
            message_reference: None,
        };

        let response = client.send_message(&req.channel_id, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": response.id,
            "channel_id": response.channel_id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "telegram")]
    #[rmcp::tool(description = "Send a message via Telegram")]
    pub async fn telegram_send_message(
        &self,
        #[rmcp::tool(aggr)] req: TelegramMessageRequest,
    ) -> Result<String, String> {
        let client = self.telegram.as_ref()
            .ok_or_else(|| "Telegram client not configured".to_string())?;

        let parse_mode = req.parse_mode.map(|p| match p.as_str() {
            "MarkdownV2" => comm::telegram::ParseMode::MarkdownV2,
            "HTML" => comm::telegram::ParseMode::Html,
            _ => comm::telegram::ParseMode::Markdown,
        });

        let request = comm::telegram::SendMessageRequest {
            chat_id: req.chat_id,
            text: req.text,
            parse_mode,
            disable_web_page_preview: None,
            disable_notification: None,
            reply_to_message_id: None,
        };

        let response = client.send_message(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "message_id": response.result.message_id,
            "chat_id": response.result.chat.id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "sendgrid")]
    #[rmcp::tool(description = "Send an email via SendGrid")]
    pub async fn sendgrid_send_email(
        &self,
        #[rmcp::tool(aggr)] req: SendGridEmailRequest,
    ) -> Result<String, String> {
        let client = self.sendgrid.as_ref()
            .ok_or_else(|| "SendGrid client not configured".to_string())?;

        let mut content = Vec::new();
        if let Some(text) = req.text {
            content.push(comm::sendgrid::Content {
                content_type: "text/plain".to_string(),
                value: text,
            });
        }
        if let Some(html) = req.html {
            content.push(comm::sendgrid::Content {
                content_type: "text/html".to_string(),
                value: html,
            });
        }

        if content.is_empty() {
            return Err("Either text or html content is required".to_string());
        }

        let request = comm::sendgrid::SendEmailRequest {
            from: comm::sendgrid::EmailAddress { email: req.from, name: None },
            to: vec![comm::sendgrid::EmailAddress { email: req.to, name: None }],
            subject: req.subject,
            content,
            cc: None,
            bcc: None,
            reply_to: None,
            attachments: None,
            template_id: None,
            dynamic_template_data: None,
        };

        client.send(&request).await
            .map_err(|e| e.to_string())?;

        Ok("Email sent successfully".to_string())
    }

    #[cfg(feature = "resend")]
    #[rmcp::tool(description = "Send an email via Resend")]
    pub async fn resend_send_email(
        &self,
        #[rmcp::tool(aggr)] req: ResendEmailRequest,
    ) -> Result<String, String> {
        let client = self.resend.as_ref()
            .ok_or_else(|| "Resend client not configured".to_string())?;

        let request = comm::resend::SendEmailRequest {
            from: req.from,
            to: vec![req.to],
            subject: req.subject,
            text: req.text,
            html: req.html,
            cc: None,
            bcc: None,
            reply_to: None,
            headers: None,
            attachments: None,
            tags: None,
        };

        let response = client.send(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": response.id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "twilio")]
    #[rmcp::tool(description = "Send an SMS via Twilio")]
    pub async fn twilio_send_sms(
        &self,
        #[rmcp::tool(aggr)] req: TwilioSmsRequest,
    ) -> Result<String, String> {
        let client = self.twilio.as_ref()
            .ok_or_else(|| "Twilio client not configured".to_string())?;

        let request = comm::twilio::SendSmsRequest {
            to: req.to,
            from: req.from,
            body: req.body,
            messaging_service_sid: None,
            media_url: None,
            status_callback: None,
        };

        let response = client.send_sms(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "sid": response.sid,
            "status": response.status
        })).map_err(|e| e.to_string())
    }
}
