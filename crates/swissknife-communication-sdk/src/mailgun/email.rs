use crate::{Error, Result};
use crate::mailgun::MailgunClient;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg(feature = "email")]
use crate::email::{Email, EmailResponse, EmailSender};

impl MailgunClient {
    pub async fn send_message(&self, message: MailgunMessage) -> Result<MailgunSendResponse> {
        let mut form = reqwest::multipart::Form::new()
            .text("from", message.from)
            .text("to", message.to.join(","))
            .text("subject", message.subject);

        if let Some(text) = message.text {
            form = form.text("text", text);
        }
        if let Some(html) = message.html {
            form = form.text("html", html);
        }
        if !message.cc.is_empty() {
            form = form.text("cc", message.cc.join(","));
        }
        if !message.bcc.is_empty() {
            form = form.text("bcc", message.bcc.join(","));
        }
        if let Some(reply_to) = message.reply_to {
            form = form.text("h:Reply-To", reply_to);
        }

        let response = self.client()
            .post(format!("{}/{}/messages", self.base_url(), self.domain()))
            .basic_auth("api", Some(self.api_key()))
            .multipart(form)
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

        let send_response: MailgunSendResponse = response.json().await?;
        Ok(send_response)
    }

    pub async fn get_events(&self, options: EventsOptions) -> Result<EventsResponse> {
        let mut params = vec![];
        if let Some(begin) = options.begin {
            params.push(("begin", begin));
        }
        if let Some(end) = options.end {
            params.push(("end", end));
        }
        if let Some(limit) = options.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(event) = options.event {
            params.push(("event", event));
        }

        let response = self.client()
            .get(format!("{}/{}/events", self.base_url(), self.domain()))
            .basic_auth("api", Some(self.api_key()))
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

        let events: EventsResponse = response.json().await?;
        Ok(events)
    }

    pub async fn validate_email(&self, email: &str) -> Result<ValidationResponse> {
        let response = self.client()
            .get("https://api.mailgun.net/v4/address/validate")
            .basic_auth("api", Some(self.api_key()))
            .query(&[("address", email)])
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

        let validation: ValidationResponse = response.json().await?;
        Ok(validation)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MailgunMessage {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub text: Option<String>,
    pub html: Option<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub reply_to: Option<String>,
}

impl MailgunMessage {
    pub fn new(from: &str, to: &str, subject: &str) -> Self {
        Self {
            from: from.to_string(),
            to: vec![to.to_string()],
            subject: subject.to_string(),
            ..Default::default()
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn html(mut self, html: &str) -> Self {
        self.html = Some(html.to_string());
        self
    }

    pub fn add_to(mut self, to: &str) -> Self {
        self.to.push(to.to_string());
        self
    }

    pub fn cc(mut self, cc: &str) -> Self {
        self.cc.push(cc.to_string());
        self
    }

    pub fn bcc(mut self, bcc: &str) -> Self {
        self.bcc.push(bcc.to_string());
        self
    }

    pub fn reply_to(mut self, reply_to: &str) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MailgunSendResponse {
    pub id: Option<String>,
    pub message: String,
}

#[derive(Default)]
pub struct EventsOptions {
    pub begin: Option<String>,
    pub end: Option<String>,
    pub limit: Option<u32>,
    pub event: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsResponse {
    pub items: Vec<MailgunEvent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MailgunEvent {
    pub event: String,
    pub timestamp: Option<f64>,
    pub recipient: Option<String>,
    pub message: Option<EventMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventMessage {
    pub headers: Option<EventHeaders>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventHeaders {
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidationResponse {
    pub is_valid: bool,
    pub address: String,
    pub risk: Option<String>,
    pub reason: Option<Vec<String>>,
}

#[cfg(feature = "email")]
#[async_trait]
impl EmailSender for MailgunClient {
    async fn send_email(&self, email: &Email) -> Result<EmailResponse> {
        let from = if let Some(ref name) = email.from.name {
            format!("{} <{}>", name, email.from.email)
        } else {
            email.from.email.clone()
        };

        let to: Vec<String> = email.to.iter().map(|a| {
            if let Some(ref name) = a.name {
                format!("{} <{}>", name, a.email)
            } else {
                a.email.clone()
            }
        }).collect();

        let mut message = MailgunMessage {
            from,
            to,
            subject: email.subject.clone(),
            text: email.text.clone(),
            html: email.html.clone(),
            ..Default::default()
        };

        let response = self.send_message(message).await?;

        Ok(EmailResponse {
            message_id: response.id,
            status: "sent".to_string(),
        })
    }
}
