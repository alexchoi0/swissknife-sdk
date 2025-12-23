use crate::{Error, Result};
use crate::resend::ResendClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "email")]
use crate::email::{Email, EmailResponse, EmailSender};

impl ResendClient {
    pub async fn send(&self, email: ResendEmail) -> Result<ResendResponse> {
        let response = self.client()
            .post(format!("{}/emails", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&email)
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

        let send_response: ResendResponse = response.json().await?;
        Ok(send_response)
    }

    pub async fn get_email(&self, email_id: &str) -> Result<ResendEmailInfo> {
        let response = self.client()
            .get(format!("{}/emails/{}", self.base_url(), email_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let email_info: ResendEmailInfo = response.json().await?;
        Ok(email_info)
    }

    pub async fn list_domains(&self) -> Result<DomainsResponse> {
        let response = self.client()
            .get(format!("{}/domains", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let domains: DomainsResponse = response.json().await?;
        Ok(domains)
    }

    pub async fn create_api_key(&self, name: &str, permission: Option<&str>) -> Result<ApiKeyResponse> {
        let body = serde_json::json!({
            "name": name,
            "permission": permission.unwrap_or("full_access")
        });

        let response = self.client()
            .post(format!("{}/api-keys", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
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

        let api_key: ApiKeyResponse = response.json().await?;
        Ok(api_key)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResendEmail {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<ResendTag>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResendTag {
    pub name: String,
    pub value: String,
}

impl ResendEmail {
    pub fn new(from: &str, to: &str, subject: &str) -> Self {
        Self {
            from: from.to_string(),
            to: vec![to.to_string()],
            subject: subject.to_string(),
            html: None,
            text: None,
            cc: None,
            bcc: None,
            reply_to: None,
            tags: None,
        }
    }

    pub fn html(mut self, html: &str) -> Self {
        self.html = Some(html.to_string());
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn add_to(mut self, to: &str) -> Self {
        self.to.push(to.to_string());
        self
    }

    pub fn cc(mut self, cc: Vec<String>) -> Self {
        self.cc = Some(cc);
        self
    }

    pub fn bcc(mut self, bcc: Vec<String>) -> Self {
        self.bcc = Some(bcc);
        self
    }

    pub fn reply_to(mut self, reply_to: &str) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }

    pub fn tag(mut self, name: &str, value: &str) -> Self {
        self.tags.get_or_insert_with(Vec::new).push(ResendTag {
            name: name.to_string(),
            value: value.to_string(),
        });
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResendResponse {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResendEmailInfo {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub created_at: String,
    pub last_event: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainsResponse {
    pub data: Vec<Domain>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub token: String,
}

#[cfg(feature = "email")]
#[async_trait]
impl EmailSender for ResendClient {
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

        let resend_email = ResendEmail {
            from,
            to,
            subject: email.subject.clone(),
            html: email.html.clone(),
            text: email.text.clone(),
            cc: None,
            bcc: None,
            reply_to: None,
            tags: None,
        };

        let response = self.send(resend_email).await?;

        Ok(EmailResponse {
            message_id: Some(response.id),
            status: "sent".to_string(),
        })
    }
}
