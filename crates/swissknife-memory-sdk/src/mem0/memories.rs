use crate::{AddMemoryOptions, Error, Memory, Message, Result, SearchOptions, SearchResult};
use crate::mem0::Mem0Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Mem0Client {
    pub async fn add(&self, messages: &[Mem0Message], options: Option<AddOptions>) -> Result<AddResponse> {
        let mut body = serde_json::json!({
            "messages": messages
        });

        if let Some(opts) = options {
            if let Some(user_id) = opts.user_id {
                body["user_id"] = serde_json::Value::String(user_id);
            }
            if let Some(agent_id) = opts.agent_id {
                body["agent_id"] = serde_json::Value::String(agent_id);
            }
            if let Some(run_id) = opts.run_id {
                body["run_id"] = serde_json::Value::String(run_id);
            }
            if let Some(metadata) = opts.metadata {
                body["metadata"] = serde_json::to_value(metadata).unwrap_or_default();
            }
            if let Some(output_format) = opts.output_format {
                body["output_format"] = serde_json::Value::String(output_format);
            }
        }

        let response = self.client()
            .post(format!("{}/memories/", self.base_url()))
            .header("Authorization", format!("Token {}", self.api_key()))
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

        let result: AddResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get(&self, memory_id: &str) -> Result<Mem0Memory> {
        let response = self.client()
            .get(format!("{}/memories/{}/", self.base_url(), memory_id))
            .header("Authorization", format!("Token {}", self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::MemoryNotFound(memory_id.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: Mem0Memory = response.json().await?;
        Ok(result)
    }

    pub async fn get_all(&self, params: Option<GetAllParams>) -> Result<Vec<Mem0Memory>> {
        let mut request = self.client()
            .get(format!("{}/memories/", self.base_url()))
            .header("Authorization", format!("Token {}", self.api_key()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(user_id) = p.user_id {
                query.push(("user_id", user_id));
            }
            if let Some(agent_id) = p.agent_id {
                query.push(("agent_id", agent_id));
            }
            if let Some(run_id) = p.run_id {
                query.push(("run_id", run_id));
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

        let result: Vec<Mem0Memory> = response.json().await?;
        Ok(result)
    }

    pub async fn search(&self, query: &str, options: Option<SearchParams>) -> Result<Vec<Mem0SearchResult>> {
        let mut body = serde_json::json!({
            "query": query
        });

        if let Some(opts) = options {
            if let Some(user_id) = opts.user_id {
                body["user_id"] = serde_json::Value::String(user_id);
            }
            if let Some(agent_id) = opts.agent_id {
                body["agent_id"] = serde_json::Value::String(agent_id);
            }
            if let Some(run_id) = opts.run_id {
                body["run_id"] = serde_json::Value::String(run_id);
            }
            if let Some(limit) = opts.limit {
                body["limit"] = serde_json::Value::Number(limit.into());
            }
        }

        let response = self.client()
            .post(format!("{}/memories/search/", self.base_url()))
            .header("Authorization", format!("Token {}", self.api_key()))
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

        let result: Vec<Mem0SearchResult> = response.json().await?;
        Ok(result)
    }

    pub async fn update(&self, memory_id: &str, data: &str) -> Result<Mem0Memory> {
        let body = serde_json::json!({
            "data": data
        });

        let response = self.client()
            .put(format!("{}/memories/{}/", self.base_url(), memory_id))
            .header("Authorization", format!("Token {}", self.api_key()))
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

        let result: Mem0Memory = response.json().await?;
        Ok(result)
    }

    pub async fn delete(&self, memory_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/memories/{}/", self.base_url(), memory_id))
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

    pub async fn delete_all(&self, params: Option<DeleteAllParams>) -> Result<DeleteAllResponse> {
        let mut body = serde_json::json!({});

        if let Some(p) = params {
            if let Some(user_id) = p.user_id {
                body["user_id"] = serde_json::Value::String(user_id);
            }
            if let Some(agent_id) = p.agent_id {
                body["agent_id"] = serde_json::Value::String(agent_id);
            }
            if let Some(run_id) = p.run_id {
                body["run_id"] = serde_json::Value::String(run_id);
            }
        }

        let response = self.client()
            .delete(format!("{}/memories/", self.base_url()))
            .header("Authorization", format!("Token {}", self.api_key()))
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

        let result: DeleteAllResponse = response.json().await?;
        Ok(result)
    }

    pub async fn history(&self, memory_id: &str) -> Result<Vec<MemoryHistory>> {
        let response = self.client()
            .get(format!("{}/memories/{}/history/", self.base_url(), memory_id))
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

        let result: Vec<MemoryHistory> = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Mem0Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub output_format: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetAllParams {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct DeleteAllParams {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddResponse {
    pub results: Vec<AddedMemory>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddedMemory {
    pub id: String,
    pub memory: String,
    pub event: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mem0Memory {
    pub id: String,
    pub memory: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub hash: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mem0SearchResult {
    pub id: String,
    pub memory: String,
    pub score: f32,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteAllResponse {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryHistory {
    pub id: String,
    pub memory_id: String,
    pub prev_value: Option<String>,
    pub new_value: Option<String>,
    pub event: String,
    pub timestamp: String,
}

impl From<Mem0Memory> for Memory {
    fn from(m: Mem0Memory) -> Self {
        Self {
            id: m.id,
            content: m.memory,
            user_id: m.user_id,
            agent_id: m.agent_id,
            session_id: m.run_id,
            metadata: m.metadata.unwrap_or_default(),
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl From<Mem0SearchResult> for SearchResult {
    fn from(r: Mem0SearchResult) -> Self {
        Self {
            memory: Memory {
                id: r.id,
                content: r.memory,
                user_id: r.user_id,
                agent_id: r.agent_id,
                session_id: r.run_id,
                metadata: r.metadata.unwrap_or_default(),
                created_at: r.created_at,
                updated_at: None,
            },
            score: r.score,
        }
    }
}
