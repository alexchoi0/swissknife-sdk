use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Message,
    ToolCall,
    ToolResult,
    Thinking,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Message => "message",
            ActionType::ToolCall => "tool_call",
            ActionType::ToolResult => "tool_result",
            ActionType::Thinking => "thinking",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "message" => Some(ActionType::Message),
            "tool_call" => Some(ActionType::ToolCall),
            "tool_result" => Some(ActionType::ToolResult),
            "thinking" => Some(ActionType::Thinking),
            _ => None,
        }
    }
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub session_id: String,
    pub sequence: i64,
    pub action_type: ActionType,
    pub role: Option<String>,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<String>,
    pub tool_call_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: String,
    pub action_id: String,
    pub embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub action: Action,
    pub score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryConfig {
    pub db_path: Option<String>,
    pub embedding_dim: usize,
}

impl MemoryConfig {
    pub fn new() -> Self {
        Self {
            db_path: None,
            embedding_dim: 1024,
        }
    }

    pub fn with_db_path(mut self, path: impl Into<String>) -> Self {
        self.db_path = Some(path.into());
        self
    }

    pub fn with_embedding_dim(mut self, dim: usize) -> Self {
        self.embedding_dim = dim;
        self
    }
}
