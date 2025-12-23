use crate::{Error, Result};
use crate::zep::ZepClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl ZepClient {
    pub async fn add_memory(&self, session_id: &str, messages: &[ZepMessage]) -> Result<ZepMemoryResponse> {
        let body = serde_json::json!({
            "messages": messages
        });

        let response = self.client()
            .post(format!("{}/sessions/{}/memory", self.base_url(), session_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
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

        let result: ZepMemoryResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_memory(&self, session_id: &str, params: Option<GetMemoryParams>) -> Result<ZepMemory> {
        let mut request = self.client()
            .get(format!("{}/sessions/{}/memory", self.base_url(), session_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(lastn) = p.lastn {
                query.push(("lastn", lastn.to_string()));
            }
            if let Some(min_rating) = p.min_rating {
                query.push(("min_rating", min_rating.to_string()));
            }
            if !query.is_empty() {
                request = request.query(&query);
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::SessionNotFound(session_id.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ZepMemory = response.json().await?;
        Ok(result)
    }

    pub async fn delete_memory(&self, session_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/sessions/{}/memory", self.base_url(), session_id))
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

    pub async fn search_memory(&self, session_id: &str, query: &str, params: Option<SearchMemoryParams>) -> Result<Vec<ZepSearchResult>> {
        let mut body = serde_json::json!({
            "text": query
        });

        if let Some(p) = params {
            if let Some(limit) = p.limit {
                body["limit"] = serde_json::Value::Number(limit.into());
            }
            if let Some(min_score) = p.min_score {
                body["min_score"] = serde_json::Value::Number(serde_json::Number::from_f64(min_score as f64).unwrap());
            }
            if let Some(search_type) = p.search_type {
                body["search_type"] = serde_json::Value::String(search_type);
            }
            if let Some(search_scope) = p.search_scope {
                body["search_scope"] = serde_json::Value::String(search_scope);
            }
        }

        let response = self.client()
            .post(format!("{}/sessions/{}/search", self.base_url(), session_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
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

        let result: Vec<ZepSearchResult> = response.json().await?;
        Ok(result)
    }

    pub async fn create_session(&self, session: CreateSessionRequest) -> Result<ZepSession> {
        let response = self.client()
            .post(format!("{}/sessions", self.base_url()))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
            .json(&session)
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

        let result: ZepSession = response.json().await?;
        Ok(result)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<ZepSession> {
        let response = self.client()
            .get(format!("{}/sessions/{}", self.base_url(), session_id))
            .header("Authorization", format!("Api-Key {}", self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::SessionNotFound(session_id.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ZepSession = response.json().await?;
        Ok(result)
    }

    pub async fn list_sessions(&self, params: Option<ListSessionsParams>) -> Result<Vec<ZepSession>> {
        let mut request = self.client()
            .get(format!("{}/sessions", self.base_url()))
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

        let result: Vec<ZepSession> = response.json().await?;
        Ok(result)
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/sessions/{}", self.base_url(), session_id))
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
}

#[derive(Debug, Clone, Serialize)]
pub struct ZepMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Default)]
pub struct GetMemoryParams {
    pub lastn: Option<u32>,
    pub min_rating: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchMemoryParams {
    pub limit: Option<u32>,
    pub min_score: Option<f32>,
    pub search_type: Option<String>,
    pub search_scope: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListSessionsParams {
    pub limit: Option<u32>,
    pub cursor: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionRequest {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZepMemoryResponse {
    pub messages: Option<Vec<ZepMessageResponse>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZepMessageResponse {
    pub uuid: Option<String>,
    pub role: String,
    pub content: String,
    pub role_type: Option<String>,
    pub token_count: Option<u32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZepMemory {
    pub messages: Option<Vec<ZepMessageResponse>>,
    pub summary: Option<ZepSummary>,
    pub facts: Option<Vec<String>>,
    pub relevant_facts: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZepSummary {
    pub uuid: Option<String>,
    pub content: String,
    pub token_count: Option<u32>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZepSearchResult {
    pub message: Option<ZepMessageResponse>,
    pub summary: Option<ZepSummary>,
    pub score: Option<f32>,
    pub dist: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZepSession {
    pub uuid: Option<String>,
    pub session_id: String,
    pub user_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
