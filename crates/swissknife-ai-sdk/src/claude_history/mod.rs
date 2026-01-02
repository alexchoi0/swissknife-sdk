mod parser;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use parser::{discover_sessions, parse_history_jsonl, parse_session_jsonl, parse_todos_dir};

#[cfg(feature = "duckdb")]
use crate::memory::DuckDBMemory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudePrompt {
    pub display: String,
    pub timestamp: i64,
    pub project: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessageContent {
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub text: Option<String>,
    pub thinking: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessageInner {
    pub role: Option<String>,
    pub content: Option<serde_json::Value>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub uuid: String,
    #[serde(rename = "parentUuid")]
    pub parent_uuid: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
    pub timestamp: Option<String>,
    pub message: Option<ClaudeMessageInner>,
    pub cwd: Option<String>,
    #[serde(rename = "gitBranch")]
    pub git_branch: Option<String>,
    pub summary: Option<String>,
}

impl ClaudeMessage {
    pub fn role(&self) -> Option<&str> {
        self.message.as_ref().and_then(|m| m.role.as_deref())
    }

    pub fn content_text(&self) -> Option<String> {
        let content = self.message.as_ref()?.content.as_ref()?;

        if let Some(text) = content.as_str() {
            return Some(text.to_string());
        }

        if let Some(arr) = content.as_array() {
            let texts: Vec<String> = arr
                .iter()
                .filter_map(|item| {
                    if let Some(obj) = item.as_object() {
                        if obj.get("type").and_then(|t| t.as_str()) == Some("text") {
                            return obj.get("text").and_then(|t| t.as_str()).map(String::from);
                        }
                    }
                    None
                })
                .collect();
            if !texts.is_empty() {
                return Some(texts.join("\n"));
            }
        }

        None
    }

    pub fn thinking_text(&self) -> Option<String> {
        let content = self.message.as_ref()?.content.as_ref()?;

        if let Some(arr) = content.as_array() {
            let texts: Vec<String> = arr
                .iter()
                .filter_map(|item| {
                    if let Some(obj) = item.as_object() {
                        if obj.get("type").and_then(|t| t.as_str()) == Some("thinking") {
                            return obj.get("thinking").and_then(|t| t.as_str()).map(String::from);
                        }
                    }
                    None
                })
                .collect();
            if !texts.is_empty() {
                return Some(texts.join("\n"));
            }
        }

        None
    }

    pub fn tool_uses(&self) -> Vec<ClaudeToolUse> {
        let Some(content) = self.message.as_ref().and_then(|m| m.content.as_ref()) else {
            return Vec::new();
        };

        let Some(arr) = content.as_array() else {
            return Vec::new();
        };

        arr.iter()
            .filter_map(|item| {
                let obj = item.as_object()?;
                if obj.get("type").and_then(|t| t.as_str()) != Some("tool_use") {
                    return None;
                }
                Some(ClaudeToolUse {
                    id: obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    input: obj.get("input").cloned().unwrap_or(serde_json::Value::Null),
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeTodo {
    pub session_id: String,
    pub agent_id: String,
    pub content: String,
    pub status: String,
    #[serde(rename = "activeForm")]
    pub active_form: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RawTodoItem {
    pub content: String,
    pub status: String,
    #[serde(rename = "activeForm")]
    pub active_form: String,
}

pub struct ClaudeHistoryImporter {
    #[cfg(feature = "duckdb")]
    memory: DuckDBMemory,
    claude_dir: PathBuf,
}

impl ClaudeHistoryImporter {
    #[cfg(feature = "duckdb")]
    pub fn new(memory: DuckDBMemory) -> Self {
        let claude_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude");
        Self { memory, claude_dir }
    }

    #[cfg(feature = "duckdb")]
    pub fn with_claude_dir(memory: DuckDBMemory, claude_dir: PathBuf) -> Self {
        Self { memory, claude_dir }
    }

    pub fn history_file(&self) -> PathBuf {
        self.claude_dir.join("history.jsonl")
    }

    pub fn projects_dir(&self) -> PathBuf {
        self.claude_dir.join("projects")
    }

    pub fn todos_dir(&self) -> PathBuf {
        self.claude_dir.join("todos")
    }

    pub fn read_prompts(&self) -> crate::Result<Vec<ClaudePrompt>> {
        let path = self.history_file();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;

        let prompts: Vec<ClaudePrompt> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(prompts)
    }

    pub fn read_session_messages(&self, project_path: &str, session_id: &str) -> crate::Result<Vec<ClaudeMessage>> {
        let project_dir_name = project_path.replace('/', "-");
        let session_file = self.projects_dir()
            .join(&project_dir_name)
            .join(format!("{}.jsonl", session_id));

        if !session_file.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&session_file)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;

        let messages: Vec<ClaudeMessage> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(messages)
    }

    pub fn list_projects(&self) -> crate::Result<Vec<String>> {
        let projects_dir = self.projects_dir();
        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&projects_dir)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;

        let projects: Vec<String> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().ok()?.is_dir() {
                    entry.file_name().to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(projects)
    }

    pub fn list_sessions(&self, project_path: &str) -> crate::Result<Vec<String>> {
        let project_dir_name = project_path.replace('/', "-");
        let project_dir = self.projects_dir().join(&project_dir_name);

        if !project_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&project_dir)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;

        let sessions: Vec<String> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_str()?.to_string();
                if name.ends_with(".jsonl") {
                    Some(name.trim_end_matches(".jsonl").to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(sessions)
    }

    pub fn read_todos(&self) -> crate::Result<Vec<ClaudeTodo>> {
        let todos_dir = self.todos_dir();
        if !todos_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&todos_dir)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;

        let mut all_todos = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let filename = match path.file_stem().and_then(|s| s.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            let parts: Vec<&str> = filename.splitn(2, "-agent-").collect();
            if parts.len() != 2 {
                continue;
            }

            let session_id = parts[0].to_string();
            let agent_id = parts[1].to_string();

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let items: Vec<RawTodoItem> = match serde_json::from_str(&content) {
                Ok(items) => items,
                Err(_) => continue,
            };

            for item in items {
                all_todos.push(ClaudeTodo {
                    session_id: session_id.clone(),
                    agent_id: agent_id.clone(),
                    content: item.content,
                    status: item.status,
                    active_form: item.active_form,
                });
            }
        }

        Ok(all_todos)
    }

    #[cfg(feature = "duckdb")]
    pub fn memory(&self) -> &DuckDBMemory {
        &self.memory
    }
}
