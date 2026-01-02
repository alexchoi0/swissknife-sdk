use crate::format::truncate_str;
use serde::{Deserialize, Serialize};
use serde_json::json;
use swissknife_ai_sdk::llm::{FunctionDefinition, ToolDefinition};
use swissknife_ai_sdk::memory::DuckDBMemory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryArgs {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

pub fn get_history_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "search_history".to_string(),
                description: Some("Search through past Claude Code conversation history. Searches both user prompts and assistant messages. Returns matching conversations with timestamps and project context.".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query to find in past conversations"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 20)"
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
    ]
}

pub fn execute_history(
    name: &str,
    arguments: &str,
    memory: &DuckDBMemory,
) -> Result<String, String> {
    match name {
        "search_history" => {
            let args: SearchHistoryArgs =
                serde_json::from_str(arguments).map_err(|e| format!("Invalid arguments: {}", e))?;
            search_history(&args.query, args.limit, memory)
        }
        _ => Err(format!("Unknown history tool: {}", name)),
    }
}

fn search_history(query: &str, limit: usize, memory: &DuckDBMemory) -> Result<String, String> {
    let mut results = Vec::new();

    if let Ok(prompts) = memory.search_claude_prompts(query, limit) {
        for prompt in prompts {
            let ts = chrono::DateTime::from_timestamp(prompt.timestamp / 1000, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let proj = prompt.project.as_deref().unwrap_or("unknown");
            results.push(format!(
                "[{}] Project: {}\nPrompt: {}\n",
                ts, proj, prompt.display
            ));
        }
    }

    if let Ok(messages) = memory.search_claude_messages(query, limit) {
        for msg in messages {
            let role = msg.role.as_deref().unwrap_or(&msg.message_type);
            let content = msg.content.as_deref().unwrap_or("");
            let preview: String = content.chars().take(500).collect();
            results.push(format!(
                "[Session: {}] Role: {}\nContent: {}\n",
                truncate_str(&msg.session_id, 8),
                role,
                preview
            ));
        }
    }

    if results.is_empty() {
        Ok(format!("No results found for query: {}", query))
    } else {
        Ok(format!(
            "Found {} results:\n\n{}",
            results.len(),
            results.join("\n---\n")
        ))
    }
}
