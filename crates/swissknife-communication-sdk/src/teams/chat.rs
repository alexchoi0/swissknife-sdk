use crate::{Error, Result};
use crate::teams::TeamsClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "chat")]
use crate::chat::{ChatMessage, ChatResponse, ChatSender};

impl TeamsClient {
    pub async fn send_channel_message(&self, team_id: &str, channel_id: &str, message: TeamsMessage) -> Result<TeamsMessageResponse> {
        let response = self.client()
            .post(format!("{}/teams/{}/channels/{}/messages", self.base_url(), team_id, channel_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let msg_response: TeamsMessageResponse = response.json().await?;
        Ok(msg_response)
    }

    pub async fn send_chat_message(&self, chat_id: &str, message: TeamsMessage) -> Result<TeamsMessageResponse> {
        let response = self.client()
            .post(format!("{}/chats/{}/messages", self.base_url(), chat_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let msg_response: TeamsMessageResponse = response.json().await?;
        Ok(msg_response)
    }

    pub async fn list_teams(&self) -> Result<TeamsListResponse> {
        let response = self.client()
            .get(format!("{}/me/joinedTeams", self.base_url()))
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

        let teams: TeamsListResponse = response.json().await?;
        Ok(teams)
    }

    pub async fn list_channels(&self, team_id: &str) -> Result<ChannelsListResponse> {
        let response = self.client()
            .get(format!("{}/teams/{}/channels", self.base_url(), team_id))
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

        let channels: ChannelsListResponse = response.json().await?;
        Ok(channels)
    }

    pub async fn list_chats(&self) -> Result<ChatsListResponse> {
        let response = self.client()
            .get(format!("{}/me/chats", self.base_url()))
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

        let chats: ChatsListResponse = response.json().await?;
        Ok(chats)
    }

    pub async fn get_channel_messages(&self, team_id: &str, channel_id: &str) -> Result<MessagesListResponse> {
        let response = self.client()
            .get(format!("{}/teams/{}/channels/{}/messages", self.base_url(), team_id, channel_id))
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

        let messages: MessagesListResponse = response.json().await?;
        Ok(messages)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamsMessage {
    pub body: TeamsMessageBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamsMessageBody {
    pub content: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
}

impl TeamsMessage {
    pub fn text(content: &str) -> Self {
        Self {
            body: TeamsMessageBody {
                content: content.to_string(),
                content_type: "text".to_string(),
            },
        }
    }

    pub fn html(content: &str) -> Self {
        Self {
            body: TeamsMessageBody {
                content: content.to_string(),
                content_type: "html".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamsMessageResponse {
    pub id: String,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamsListResponse {
    pub value: Vec<Team>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelsListResponse {
    pub value: Vec<Channel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatsListResponse {
    pub value: Vec<TeamsChat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamsChat {
    pub id: String,
    #[serde(rename = "chatType")]
    pub chat_type: Option<String>,
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagesListResponse {
    pub value: Vec<TeamsMessageResponse>,
}

pub struct TeamChannelId {
    pub team_id: String,
    pub channel_id: String,
}

#[cfg(feature = "chat")]
#[async_trait]
impl ChatSender for TeamsClient {
    async fn send_message(&self, channel: &str, message: &ChatMessage) -> Result<ChatResponse> {
        let parts: Vec<&str> = channel.split('/').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidRequest("Channel format should be team_id/channel_id".to_string()));
        }

        let team_id = parts[0];
        let channel_id = parts[1];

        let teams_message = TeamsMessage::text(&message.text);
        let response = self.send_channel_message(team_id, channel_id, teams_message).await?;

        Ok(ChatResponse {
            message_id: Some(response.id),
            channel: Some(channel.to_string()),
            timestamp: response.created_date_time,
        })
    }
}
