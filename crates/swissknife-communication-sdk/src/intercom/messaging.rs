use crate::{Error, Result};
use crate::intercom::IntercomClient;
use serde::{Deserialize, Serialize};

impl IntercomClient {
    pub async fn send_message(&self, message: IntercomMessage) -> Result<Conversation> {
        let response = self.client()
            .post(format!("{}/messages", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Intercom-Version", "2.10")
            .json(&message)
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

        let conversation: Conversation = response.json().await?;
        Ok(conversation)
    }

    pub async fn reply_to_conversation(&self, conversation_id: &str, reply: ConversationReply) -> Result<Conversation> {
        let response = self.client()
            .post(format!("{}/conversations/{}/reply", self.base_url(), conversation_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Intercom-Version", "2.10")
            .json(&reply)
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

        let conversation: Conversation = response.json().await?;
        Ok(conversation)
    }

    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Conversation> {
        let response = self.client()
            .get(format!("{}/conversations/{}", self.base_url(), conversation_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Intercom-Version", "2.10")
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

        let conversation: Conversation = response.json().await?;
        Ok(conversation)
    }

    pub async fn list_conversations(&self, options: ListConversationsOptions) -> Result<ConversationsResponse> {
        let mut params = vec![];
        if let Some(per_page) = options.per_page {
            params.push(("per_page", per_page.to_string()));
        }
        if let Some(starting_after) = options.starting_after {
            params.push(("starting_after", starting_after));
        }

        let response = self.client()
            .get(format!("{}/conversations", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Intercom-Version", "2.10")
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

        let conversations: ConversationsResponse = response.json().await?;
        Ok(conversations)
    }

    pub async fn create_contact(&self, contact: CreateContact) -> Result<Contact> {
        let response = self.client()
            .post(format!("{}/contacts", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Intercom-Version", "2.10")
            .json(&contact)
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

        let created: Contact = response.json().await?;
        Ok(created)
    }

    pub async fn get_contact(&self, contact_id: &str) -> Result<Contact> {
        let response = self.client()
            .get(format!("{}/contacts/{}", self.base_url(), contact_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Intercom-Version", "2.10")
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

        let contact: Contact = response.json().await?;
        Ok(contact)
    }

    pub async fn search_contacts(&self, query: ContactSearchQuery) -> Result<ContactsResponse> {
        let response = self.client()
            .post(format!("{}/contacts/search", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Intercom-Version", "2.10")
            .json(&query)
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

        let contacts: ContactsResponse = response.json().await?;
        Ok(contacts)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IntercomMessage {
    pub from: MessageFrom,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<MessageTo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageFrom {
    #[serde(rename = "type")]
    pub from_type: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageTo {
    #[serde(rename = "type")]
    pub to_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationReply {
    pub message_type: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intercom_user_id: Option<String>,
}

impl ConversationReply {
    pub fn admin(admin_id: &str, body: &str) -> Self {
        Self {
            message_type: "comment".to_string(),
            body: body.to_string(),
            admin_id: Some(admin_id.to_string()),
            intercom_user_id: None,
        }
    }

    pub fn user(user_id: &str, body: &str) -> Self {
        Self {
            message_type: "comment".to_string(),
            body: body.to_string(),
            admin_id: None,
            intercom_user_id: Some(user_id.to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Conversation {
    pub id: String,
    #[serde(rename = "type")]
    pub conversation_type: String,
    pub state: Option<String>,
    pub created_at: Option<u64>,
    pub updated_at: Option<u64>,
}

#[derive(Default)]
pub struct ListConversationsOptions {
    pub per_page: Option<u32>,
    pub starting_after: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationsResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub conversations: Vec<Conversation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateContact {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

impl CreateContact {
    pub fn user(email: &str) -> Self {
        Self {
            role: "user".to_string(),
            email: Some(email.to_string()),
            phone: None,
            name: None,
            external_id: None,
        }
    }

    pub fn lead(email: &str) -> Self {
        Self {
            role: "lead".to_string(),
            email: Some(email.to_string()),
            phone: None,
            name: None,
            external_id: None,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn phone(mut self, phone: &str) -> Self {
        self.phone = Some(phone.to_string());
        self
    }

    pub fn external_id(mut self, external_id: &str) -> Self {
        self.external_id = Some(external_id.to_string());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Contact {
    pub id: String,
    #[serde(rename = "type")]
    pub contact_type: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContactSearchQuery {
    pub query: SearchQuery,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchQuery {
    pub field: String,
    pub operator: String,
    pub value: String,
}

impl ContactSearchQuery {
    pub fn by_email(email: &str) -> Self {
        Self {
            query: SearchQuery {
                field: "email".to_string(),
                operator: "=".to_string(),
                value: email.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactsResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub data: Vec<Contact>,
}
