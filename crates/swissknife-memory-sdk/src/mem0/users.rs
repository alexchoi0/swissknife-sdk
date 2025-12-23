use crate::{Error, Result};
use crate::mem0::Mem0Client;
use serde::Deserialize;

impl Mem0Client {
    pub async fn list_users(&self) -> Result<Vec<User>> {
        let response = self.client()
            .get(format!("{}/users/", self.base_url()))
            .header("Authorization", format!("Token {}", self.api_key()))
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

        let result: UsersResponse = response.json().await?;
        Ok(result.results)
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/users/{}/", self.base_url(), user_id))
            .header("Authorization", format!("Token {}", self.api_key()))
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsersResponse {
    pub results: Vec<User>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub total_memories: Option<u32>,
}
