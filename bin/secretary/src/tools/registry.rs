use super::builtin::{execute_builtin, get_builtin_definitions};
use super::history::{execute_history, get_history_definitions};
use super::mcp::McpManager;
use swissknife_ai_sdk::llm::{FunctionDefinition, ToolDefinition};
use swissknife_ai_sdk::memory::DuckDBMemory;

pub struct ToolRegistry {
    builtin_tools: Vec<ToolDefinition>,
    history_tools: Vec<ToolDefinition>,
    mcp_manager: McpManager,
}

impl ToolRegistry {
    pub fn new(enable_builtin: bool, enable_history: bool) -> Self {
        let builtin_tools = if enable_builtin {
            get_builtin_definitions()
        } else {
            Vec::new()
        };

        let history_tools = if enable_history {
            get_history_definitions()
        } else {
            Vec::new()
        };

        Self {
            builtin_tools,
            history_tools,
            mcp_manager: McpManager::new(),
        }
    }

    pub async fn enable_sdk_mcp(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.mcp_manager.enable_sdk_tools().await
    }

    pub async fn add_external_mcp(
        &mut self,
        name: &str,
        command: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mcp_manager.add_external_server(name, command).await
    }

    pub fn all_tool_definitions(&self) -> Vec<ToolDefinition> {
        let mut tools = self.builtin_tools.clone();
        tools.extend(self.history_tools.clone());

        for mcp_tool in self.mcp_manager.sdk_tools() {
            tools.push(ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: mcp_tool.name.to_string(),
                    description: mcp_tool.description.as_ref().map(|s| s.to_string()),
                    parameters: serde_json::Value::Object((*mcp_tool.input_schema).clone()),
                },
            });
        }

        for (_, mcp_tool) in self.mcp_manager.external_tools() {
            tools.push(ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: mcp_tool.name.to_string(),
                    description: mcp_tool.description.as_ref().map(|s| s.to_string()),
                    parameters: serde_json::Value::Object((*mcp_tool.input_schema).clone()),
                },
            });
        }

        tools
    }

    pub async fn execute_tool(
        &self,
        name: &str,
        arguments: &str,
        memory: &DuckDBMemory,
    ) -> Result<String, String> {
        if self.builtin_tools.iter().any(|t| t.function.name == name) {
            return execute_builtin(name, arguments);
        }

        if self.history_tools.iter().any(|t| t.function.name == name) {
            return execute_history(name, arguments, memory);
        }

        if self.mcp_manager.find_sdk_tool(name) {
            let args: Option<serde_json::Map<String, serde_json::Value>> =
                serde_json::from_str(arguments).ok();
            return self.mcp_manager.call_sdk_tool(name, args).await;
        }

        if self.mcp_manager.find_external_tool(name).is_some() {
            let args: Option<serde_json::Value> = serde_json::from_str(arguments).ok();
            return self.mcp_manager.call_external_tool(name, args).await;
        }

        Err(format!("Unknown tool: {}", name))
    }

    pub fn tool_source(&self, name: &str) -> &'static str {
        if self.builtin_tools.iter().any(|t| t.function.name == name) {
            return "builtin";
        }
        if self.history_tools.iter().any(|t| t.function.name == name) {
            return "history";
        }
        if self.mcp_manager.find_sdk_tool(name) {
            return "sdk-mcp";
        }
        if self.mcp_manager.find_external_tool(name).is_some() {
            return "ext-mcp";
        }
        "unknown"
    }

    pub fn print_available_tools(&self) {
        if !self.builtin_tools.is_empty() {
            eprintln!("Built-in tools ({}):", self.builtin_tools.len());
            for tool in &self.builtin_tools {
                eprintln!("  - {}", tool.function.name);
            }
        }
        if !self.history_tools.is_empty() {
            eprintln!("History tools ({}):", self.history_tools.len());
            for tool in &self.history_tools {
                eprintln!("  - {}", tool.function.name);
            }
        }
    }

    pub fn has_tools(&self) -> bool {
        !self.builtin_tools.is_empty()
            || !self.history_tools.is_empty()
            || !self.mcp_manager.sdk_tools().is_empty()
            || !self.mcp_manager.external_tools().is_empty()
    }
}
