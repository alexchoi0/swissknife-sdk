use crate::{Error, Result};
use crate::outlook::OutlookClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "email")]
use crate::email::{Email, EmailAddress, EmailResponse, EmailSender};

impl OutlookClient {
    pub async fn send_mail(&self, message: OutlookMailMessage, save_to_sent: bool) -> Result<()> {
        let body = serde_json::json!({
            "message": message,
            "saveToSentItems": save_to_sent
        });

        let response = self.client()
            .post(format!("{}/me/sendMail", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        Ok(())
    }

    pub async fn list_messages(&self, folder: Option<&str>, top: Option<u32>) -> Result<OutlookMessagesResponse> {
        let folder = folder.unwrap_or("inbox");
        let mut url = format!("{}/me/mailFolders/{}/messages", self.base_url(), folder);

        if let Some(top) = top {
            url.push_str(&format!("?$top={}", top));
        }

        let response = self.client()
            .get(&url)
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

        let messages: OutlookMessagesResponse = response.json().await?;
        Ok(messages)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<OutlookMessage> {
        let response = self.client()
            .get(format!("{}/me/messages/{}", self.base_url(), message_id))
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

        let message: OutlookMessage = response.json().await?;
        Ok(message)
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/me/messages/{}", self.base_url(), message_id))
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

    pub async fn list_folders(&self) -> Result<OutlookFoldersResponse> {
        let response = self.client()
            .get(format!("{}/me/mailFolders", self.base_url()))
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

        let folders: OutlookFoldersResponse = response.json().await?;
        Ok(folders)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutlookMailMessage {
    pub subject: String,
    pub body: OutlookBody,
    #[serde(rename = "toRecipients")]
    pub to_recipients: Vec<OutlookRecipient>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ccRecipients")]
    pub cc_recipients: Option<Vec<OutlookRecipient>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bccRecipients")]
    pub bcc_recipients: Option<Vec<OutlookRecipient>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutlookBody {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlookRecipient {
    #[serde(rename = "emailAddress")]
    pub email_address: OutlookEmailAddress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlookEmailAddress {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl OutlookMailMessage {
    pub fn new(subject: &str, to: &str) -> Self {
        Self {
            subject: subject.to_string(),
            body: OutlookBody {
                content_type: "Text".to_string(),
                content: String::new(),
            },
            to_recipients: vec![OutlookRecipient {
                email_address: OutlookEmailAddress {
                    address: to.to_string(),
                    name: None,
                },
            }],
            cc_recipients: None,
            bcc_recipients: None,
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.body = OutlookBody {
            content_type: "Text".to_string(),
            content: text.to_string(),
        };
        self
    }

    pub fn html(mut self, html: &str) -> Self {
        self.body = OutlookBody {
            content_type: "HTML".to_string(),
            content: html.to_string(),
        };
        self
    }

    pub fn add_to(mut self, to: &str, name: Option<&str>) -> Self {
        self.to_recipients.push(OutlookRecipient {
            email_address: OutlookEmailAddress {
                address: to.to_string(),
                name: name.map(String::from),
            },
        });
        self
    }

    pub fn cc(mut self, cc: &str) -> Self {
        self.cc_recipients.get_or_insert_with(Vec::new).push(OutlookRecipient {
            email_address: OutlookEmailAddress {
                address: cc.to_string(),
                name: None,
            },
        });
        self
    }

    pub fn bcc(mut self, bcc: &str) -> Self {
        self.bcc_recipients.get_or_insert_with(Vec::new).push(OutlookRecipient {
            email_address: OutlookEmailAddress {
                address: bcc.to_string(),
                name: None,
            },
        });
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutlookMessagesResponse {
    pub value: Vec<OutlookMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutlookMessage {
    pub id: String,
    pub subject: Option<String>,
    #[serde(rename = "bodyPreview")]
    pub body_preview: Option<String>,
    pub body: Option<OutlookMessageBody>,
    pub from: Option<OutlookRecipient>,
    #[serde(rename = "toRecipients")]
    pub to_recipients: Option<Vec<OutlookRecipient>>,
    #[serde(rename = "receivedDateTime")]
    pub received_date_time: Option<String>,
    #[serde(rename = "isRead")]
    pub is_read: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutlookMessageBody {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutlookFoldersResponse {
    pub value: Vec<OutlookFolder>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutlookFolder {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "totalItemCount")]
    pub total_item_count: Option<u32>,
    #[serde(rename = "unreadItemCount")]
    pub unread_item_count: Option<u32>,
}

#[cfg(feature = "email")]
#[async_trait]
impl EmailSender for OutlookClient {
    async fn send_email(&self, email: &Email) -> Result<EmailResponse> {
        let to_recipients: Vec<OutlookRecipient> = email.to.iter().map(|a| {
            OutlookRecipient {
                email_address: OutlookEmailAddress {
                    address: a.email.clone(),
                    name: a.name.clone(),
                },
            }
        }).collect();

        let content_type = if email.html.is_some() { "HTML" } else { "Text" };
        let content = email.html.clone().or(email.text.clone()).unwrap_or_default();

        let message = OutlookMailMessage {
            subject: email.subject.clone(),
            body: OutlookBody {
                content_type: content_type.to_string(),
                content,
            },
            to_recipients,
            cc_recipients: None,
            bcc_recipients: None,
        };

        self.send_mail(message, true).await?;

        Ok(EmailResponse {
            message_id: None,
            status: "sent".to_string(),
        })
    }
}
