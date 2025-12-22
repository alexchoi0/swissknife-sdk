use async_trait::async_trait;
use serde::Deserialize;

use crate::whatsapp::{WhatsAppResponse, WhatsAppSender};
use crate::{Error, Result};

use super::TwilioClient;

#[derive(Deserialize)]
struct TwilioMessageResponse {
    sid: String,
    status: String,
    #[serde(default)]
    error_code: Option<i32>,
    #[serde(default)]
    error_message: Option<String>,
}

#[async_trait]
impl WhatsAppSender for TwilioClient {
    async fn send_whatsapp(&self, from: &str, to: &str, body: &str) -> Result<WhatsAppResponse> {
        let whatsapp_from = if from.starts_with("whatsapp:") {
            from.to_string()
        } else {
            format!("whatsapp:{}", from)
        };

        let whatsapp_to = if to.starts_with("whatsapp:") {
            to.to_string()
        } else {
            format!("whatsapp:{}", to)
        };

        let params = [
            ("From", whatsapp_from.as_str()),
            ("To", whatsapp_to.as_str()),
            ("Body", body),
        ];

        let response = self
            .http
            .post(&self.messages_url())
            .header("Authorization", &self.auth_header)
            .form(&params)
            .send()
            .await?;

        let status_code = response.status();
        let response_body: TwilioMessageResponse = response.json().await?;

        if let Some(error_code) = response_body.error_code {
            return Err(Error::Api {
                code: error_code,
                message: response_body
                    .error_message
                    .unwrap_or_else(|| format!("HTTP {}", status_code)),
            });
        }

        Ok(WhatsAppResponse {
            message_id: response_body.sid,
            status: response_body.status,
        })
    }
}
