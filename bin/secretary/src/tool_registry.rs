use swissknife_ai_sdk::llm::{FunctionDefinition, ToolDefinition};
use swissknife_ai_sdk::mcp::McpHost;

use crate::mcp_client::McpClientManager;
use crate::sdk_tools_server::SdkToolServer;
use crate::tools;

pub struct ToolRegistry {
    builtin_tools: Vec<ToolDefinition>,
    mcp_host: Option<McpHost>,
    external_mcp: McpClientManager,
}

impl ToolRegistry {
    pub fn new(enable_builtin: bool) -> Self {
        let builtin_tools = if enable_builtin {
            tools::get_tool_definitions()
        } else {
            Vec::new()
        };

        Self {
            builtin_tools,
            mcp_host: None,
            external_mcp: McpClientManager::new(),
        }
    }

    /// Start in-process MCP server with SDK tools
    pub async fn enable_sdk_mcp(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server = SdkToolServer::new();
        let mcp = McpHost::new(server).await?;

        eprintln!("In-process MCP tools ({}):", mcp.tools().len());
        for tool in mcp.tools() {
            eprintln!("  - {}", tool.name);
        }

        self.mcp_host = Some(mcp);
        Ok(())
    }

    /// Connect to external MCP server via subprocess
    pub async fn add_external_mcp(&mut self, name: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.external_mcp.add_server(name, command).await
    }

    pub fn all_tool_definitions(&self) -> Vec<ToolDefinition> {
        let mut tools = self.builtin_tools.clone();

        // Add hosted MCP tools
        if let Some(mcp) = &self.mcp_host {
            for mcp_tool in mcp.tools() {
                tools.push(ToolDefinition {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: mcp_tool.name.to_string(),
                        description: mcp_tool.description.as_ref().map(|s| s.to_string()),
                        parameters: serde_json::Value::Object((*mcp_tool.input_schema).clone()),
                    },
                });
            }
        }

        // Add external MCP tools
        for (_, mcp_tool) in self.external_mcp.all_tools() {
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

    pub async fn execute_tool(&self, name: &str, arguments: &str) -> Result<String, String> {
        // Check built-in tools first
        if self.builtin_tools.iter().any(|t| t.function.name == name) {
            return tools::execute_tool(name, arguments);
        }

        // Check hosted MCP tools
        if let Some(mcp) = &self.mcp_host {
            if mcp.tools().iter().any(|t| t.name == name) {
                let args: Option<serde_json::Map<String, serde_json::Value>> =
                    serde_json::from_str(arguments).ok();
                return mcp.call_tool(name, args).await.map_err(|e| e.to_string());
            }
        }

        // Check external MCP tools
        if self.external_mcp.find_tool(name).is_some() {
            let args: Option<serde_json::Value> = serde_json::from_str(arguments).ok();
            return self.external_mcp.call_tool(name, args).await;
        }

        Err(format!("Unknown tool: {}", name))
    }

    pub fn tool_source(&self, name: &str) -> &'static str {
        if self.builtin_tools.iter().any(|t| t.function.name == name) {
            return "builtin";
        }
        if let Some(mcp) = &self.mcp_host {
            if mcp.tools().iter().any(|t| t.name == name) {
                return "sdk-mcp";
            }
        }
        if self.external_mcp.find_tool(name).is_some() {
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
    }

    pub fn has_tools(&self) -> bool {
        !self.builtin_tools.is_empty()
            || self.mcp_host.as_ref().map(|m| !m.tools().is_empty()).unwrap_or(false)
            || !self.external_mcp.is_empty()
    }
}
