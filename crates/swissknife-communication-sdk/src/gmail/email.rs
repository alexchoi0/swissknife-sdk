use crate::{Error, Result};
use crate::gmail::GmailClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "email")]
use crate::email::{Email, EmailResponse, EmailSender};

impl GmailClient {
    pub async fn send_raw(&self, user_id: &str, raw_message: &str) -> Result<GmailMessage> {
        let encoded = base64::engine::general_purpose::URL_SAFE.encode(raw_message);

        let response = self.client()
            .post(format!("{}/users/{}/messages/send", self.base_url(), user_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&serde_json::json!({ "raw": encoded }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let message: GmailMessage = response.json().await?;
        Ok(message)
    }

    pub async fn list_messages(&self, user_id: &str, query: Option<&str>, max_results: Option<u32>) -> Result<Vec<GmailMessage>> {
        let mut params = vec![];
        if let Some(q) = query {
            params.push(("q", q.to_string()));
        }
        if let Some(max) = max_results {
            params.push(("maxResults", max.to_string()));
        }

        let response = self.client()
            .get(format!("{}/users/{}/messages", self.base_url(), user_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let list_response: GmailListResponse = response.json().await?;
        Ok(list_response.messages.unwrap_or_default())
    }

    pub async fn get_message(&self, user_id: &str, message_id: &str) -> Result<GmailMessageFull> {
        let response = self.client()
            .get(format!("{}/users/{}/messages/{}", self.base_url(), user_id, message_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("format", "full")])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let message: GmailMessageFull = response.json().await?;
        Ok(message)
    }

    pub async fn delete_message(&self, user_id: &str, message_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/users/{}/messages/{}", self.base_url(), user_id, message_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        Ok(())
    }

    pub async fn list_labels(&self, user_id: &str) -> Result<Vec<GmailLabel>> {
        let response = self.client()
            .get(format!("{}/users/{}/labels", self.base_url(), user_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let list_response: GmailLabelsResponse = response.json().await?;
        Ok(list_response.labels.unwrap_or_default())
    }
}

use base64::Engine;

fn build_raw_email(from: &str, to: &[&str], subject: &str, text: Option<&str>, html: Option<&str>) -> String {
    let boundary = "boundary_swissknife";
    let to_header = to.join(", ");

    let mut raw = format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\n",
        from, to_header, subject
    );

    if text.is_some() && html.is_some() {
        raw.push_str(&format!("Content-Type: multipart/alternative; boundary=\"{}\"\r\n\r\n", boundary));
        if let Some(t) = text {
            raw.push_str(&format!("--{}\r\nContent-Type: text/plain; charset=\"UTF-8\"\r\n\r\n{}\r\n", boundary, t));
        }
        if let Some(h) = html {
            raw.push_str(&format!("--{}\r\nContent-Type: text/html; charset=\"UTF-8\"\r\n\r\n{}\r\n", boundary, h));
        }
        raw.push_str(&format!("--{}--", boundary));
    } else if let Some(h) = html {
        raw.push_str("Content-Type: text/html; charset=\"UTF-8\"\r\n\r\n");
        raw.push_str(h);
    } else if let Some(t) = text {
        raw.push_str("Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n");
        raw.push_str(t);
    }

    raw
}

#[derive(Debug, Clone, Deserialize)]
pub struct GmailMessage {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GmailMessageFull {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(rename = "labelIds")]
    pub label_ids: Option<Vec<String>>,
    pub snippet: Option<String>,
    pub payload: Option<GmailPayload>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GmailPayload {
    pub headers: Option<Vec<GmailHeader>>,
    pub body: Option<GmailBody>,
    pub parts: Option<Vec<GmailPart>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GmailHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GmailBody {
    pub size: Option<u64>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GmailPart {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub body: Option<GmailBody>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GmailLabel {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub label_type: Option<String>,
}

#[derive(Deserialize)]
struct GmailListResponse {
    messages: Option<Vec<GmailMessage>>,
}

#[derive(Deserialize)]
struct GmailLabelsResponse {
    labels: Option<Vec<GmailLabel>>,
}

#[cfg(feature = "email")]
#[async_trait]
impl EmailSender for GmailClient {
    async fn send_email(&self, email: &Email) -> Result<EmailResponse> {
        let from = if let Some(ref name) = email.from.name {
            format!("{} <{}>", name, email.from.email)
        } else {
            email.from.email.clone()
        };

        let to: Vec<&str> = email.to.iter().map(|a| a.email.as_str()).collect();

        let raw = build_raw_email(
            &from,
            &to,
            &email.subject,
            email.text.as_deref(),
            email.html.as_deref(),
        );

        let message = self.send_raw("me", &raw).await?;

        Ok(EmailResponse {
            message_id: Some(message.id),
            status: "sent".to_string(),
        })
    }
}
