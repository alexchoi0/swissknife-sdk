use async_trait::async_trait;
use serde::Serialize;

use crate::email::{Email, EmailResponse, EmailSender};
use crate::{Error, Result};

use super::SendGridClient;

#[derive(Serialize)]
struct SendGridEmail {
    personalizations: Vec<Personalization>,
    from: EmailAddr,
    subject: String,
    content: Vec<Content>,
}

#[derive(Serialize)]
struct Personalization {
    to: Vec<EmailAddr>,
}

#[derive(Serialize)]
struct EmailAddr {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    value: String,
}

impl From<&crate::email::EmailAddress> for EmailAddr {
    fn from(addr: &crate::email::EmailAddress) -> Self {
        Self {
            email: addr.email.clone(),
            name: addr.name.clone(),
        }
    }
}

#[async_trait]
impl EmailSender for SendGridClient {
    async fn send_email(&self, email: &Email) -> Result<EmailResponse> {
        let mut content = Vec::new();

        if let Some(text) = &email.text {
            content.push(Content {
                content_type: "text/plain".to_string(),
                value: text.clone(),
            });
        }

        if let Some(html) = &email.html {
            content.push(Content {
                content_type: "text/html".to_string(),
                value: html.clone(),
            });
        }

        if content.is_empty() {
            return Err(Error::Config(
                "Email must have either text or html content".into(),
            ));
        }

        let sg_email = SendGridEmail {
            personalizations: vec![Personalization {
                to: email.to.iter().map(EmailAddr::from).collect(),
            }],
            from: EmailAddr::from(&email.from),
            subject: email.subject.clone(),
            content,
        };

        let response = self
            .http
            .post(&self.mail_send_url())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&sg_email)
            .send()
            .await?;

        let status_code = response.status();
        let message_id = response
            .headers()
            .get("X-Message-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if !status_code.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                code: status_code.as_u16() as i32,
                message: error_text,
            });
        }

        Ok(EmailResponse {
            message_id,
            status: "accepted".to_string(),
        })
    }
}
