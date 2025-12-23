mod error;

pub use error::{Error, Result};

#[cfg(feature = "mem0")]
pub mod mem0;

#[cfg(feature = "zep")]
pub mod zep;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AddMemoryOptions {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub limit: Option<u32>,
    pub threshold: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub memory: Memory,
    pub score: f32,
}

#[async_trait]
pub trait MemoryProvider: Send + Sync {
    async fn add(&self, content: &str, options: &AddMemoryOptions) -> Result<Memory>;
    async fn add_messages(&self, messages: &[Message], options: &AddMemoryOptions) -> Result<Vec<Memory>>;
    async fn get(&self, memory_id: &str) -> Result<Memory>;
    async fn get_all(&self, options: &SearchOptions) -> Result<Vec<Memory>>;
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>>;
    async fn update(&self, memory_id: &str, content: &str) -> Result<Memory>;
    async fn delete(&self, memory_id: &str) -> Result<()>;
    async fn delete_all(&self, options: &SearchOptions) -> Result<()>;
}
