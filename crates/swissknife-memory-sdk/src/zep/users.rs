use crate::{Error, Result};
use crate::zep::ZepClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl ZepClient {
    pub async fn create_user(&self, user: CreateUserRequest) -> Result<ZepUser> {
        let response = self.client()
            .post(format!("{}/users", self.base_url()))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
            .json(&user)
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

        let result: ZepUser = response.json().await?;
        Ok(result)
    }

    pub async fn get_user(&self, user_id: &str) -> Result<ZepUser> {
        let response = self.client()
            .get(format!("{}/users/{}", self.base_url(), user_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::UserNotFound(user_id.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ZepUser = response.json().await?;
        Ok(result)
    }

    pub async fn update_user(&self, user_id: &str, user: UpdateUserRequest) -> Result<ZepUser> {
        let response = self.client()
            .patch(format!("{}/users/{}", self.base_url(), user_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
            .json(&user)
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

        let result: ZepUser = response.json().await?;
        Ok(result)
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/users/{}", self.base_url(), user_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
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

    pub async fn list_users(&self, params: Option<ListUsersParams>) -> Result<Vec<ZepUser>> {
        let mut request = self.client()
            .get(format!("{}/users", self.base_url()))
            .header("Authorization", format!("Api-Key {}", self.api_key()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(cursor) = p.cursor {
                query.push(("cursor", cursor.to_string()));
            }
            if !query.is_empty() {
                request = request.query(&query);
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: Vec<ZepUser> = response.json().await?;
        Ok(result)
    }

    pub async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<super::memory::ZepSession>> {
        let response = self.client()
            .get(format!("{}/users/{}/sessions", self.base_url(), user_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
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

        let result: Vec<super::memory::ZepSession> = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListUsersParams {
    pub limit: Option<u32>,
    pub cursor: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateUserRequest {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZepUser {
    pub uuid: Option<String>,
    pub user_id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
