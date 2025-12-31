use serde_json::json;
use swissknife_ai_sdk::llm::{FunctionDefinition, ToolDefinition};
use swissknife_search_sdk::{SearchOptions, SearchProvider};
use swissknife_search_sdk::tavily::TavilyClient;

use crate::mcp_client::McpClientManager;
use crate::tools;

pub struct ToolRegistry {
    builtin_tools: Vec<ToolDefinition>,
    sdk_tools: Option<SdkTools>,
    mcp_manager: McpClientManager,
}

pub struct SdkTools {
    tavily: Option<TavilyClient>,
}

impl SdkTools {
    pub fn new() -> Self {
        let tavily = std::env::var("TAVILY_API_KEY")
            .ok()
            .map(|key| TavilyClient::new(&key));

        Self { tavily }
    }

    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        let mut tools = Vec::new();

        if self.tavily.is_some() {
            tools.push(ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: "tavily_search".to_string(),
                    description: Some("Search the web using Tavily AI-powered search".to_string()),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query"
                            },
                            "max_results": {
                                "type": "integer",
                                "description": "Maximum number of results (default: 5)"
                            }
                        },
                        "required": ["query"]
                    }),
                },
            });
        }

        tools
    }

    pub async fn execute(&self, name: &str, arguments: &str) -> Result<String, String> {
        match name {
            "tavily_search" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    query: String,
                    max_results: Option<u32>,
                }
                let args: Args = serde_json::from_str(arguments)
                    .map_err(|e| format!("Invalid arguments: {}", e))?;

                let client = self.tavily.as_ref()
                    .ok_or("Tavily not configured")?;

                let options = SearchOptions {
                    max_results: args.max_results,
                    ..Default::default()
                };

                let response = client.search(&args.query, &options).await
                    .map_err(|e| e.to_string())?;

                serde_json::to_string_pretty(&json!({
                    "answer": response.answer,
                    "results": response.results.iter().map(|r| json!({
                        "title": r.title,
                        "url": r.url,
                        "content": r.content
                    })).collect::<Vec<_>>()
                })).map_err(|e| e.to_string())
            }
            _ => Err(format!("Unknown SDK tool: {}", name)),
        }
    }
}

impl ToolRegistry {
    pub fn new(enable_builtin: bool, enable_sdk: bool) -> Self {
        let builtin_tools = if enable_builtin {
            tools::get_tool_definitions()
        } else {
            Vec::new()
        };

        let sdk_tools = if enable_sdk {
            Some(SdkTools::new())
        } else {
            None
        };

        Self {
            builtin_tools,
            sdk_tools,
            mcp_manager: McpClientManager::new(),
        }
    }

    pub async fn add_mcp_server(&mut self, name: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.mcp_manager.add_server(name, command).await
    }

    pub fn all_tool_definitions(&self) -> Vec<ToolDefinition> {
        let mut tools = self.builtin_tools.clone();

        if let Some(sdk) = &self.sdk_tools {
            tools.extend(sdk.tool_definitions());
        }

        for (server_name, mcp_tool) in self.mcp_manager.all_tools() {
            tools.push(ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: mcp_tool.name.to_string(),
                    description: mcp_tool.description.as_ref().map(|s| s.to_string()),
                    parameters: serde_json::Value::Object((*mcp_tool.input_schema).clone()),
                },
            });
            let _ = server_name; // We could prefix tool names with server name if needed
        }

        tools
    }

    pub async fn execute_tool(&self, name: &str, arguments: &str) -> Result<String, String> {
        // Check built-in tools first
        if self.builtin_tools.iter().any(|t| t.function.name == name) {
            return tools::execute_tool(name, arguments);
        }

        // Check SDK tools
        if let Some(sdk) = &self.sdk_tools {
            if sdk.tool_definitions().iter().any(|t| t.function.name == name) {
                return sdk.execute(name, arguments).await;
            }
        }

        // Check MCP tools
        if self.mcp_manager.find_tool(name).is_some() {
            let args: Option<serde_json::Value> = serde_json::from_str(arguments).ok();
            return self.mcp_manager.call_tool(name, args).await;
        }

        Err(format!("Unknown tool: {}", name))
    }

    pub fn tool_source(&self, name: &str) -> &'static str {
        if self.builtin_tools.iter().any(|t| t.function.name == name) {
            return "builtin";
        }
        if let Some(sdk) = &self.sdk_tools {
            if sdk.tool_definitions().iter().any(|t| t.function.name == name) {
                return "sdk";
            }
        }
        if self.mcp_manager.find_tool(name).is_some() {
            return "mcp";
        }
        "unknown"
    }

    pub fn print_available_tools(&self) {
        let builtin_count = self.builtin_tools.len();
        let sdk_count = self.sdk_tools.as_ref().map(|s| s.tool_definitions().len()).unwrap_or(0);
        let mcp_count = self.mcp_manager.all_tools().len();

        if builtin_count > 0 {
            eprintln!("Built-in tools ({}):", builtin_count);
            for tool in &self.builtin_tools {
                eprintln!("  - {}", tool.function.name);
            }
        }

        if sdk_count > 0 {
            eprintln!("SDK tools ({}):", sdk_count);
            if let Some(sdk) = &self.sdk_tools {
                for tool in sdk.tool_definitions() {
                    eprintln!("  - {}", tool.function.name);
                }
            }
        }

        if mcp_count > 0 {
            eprintln!("MCP tools ({}):", mcp_count);
            for (server, tool) in self.mcp_manager.all_tools() {
                eprintln!("  - {} (from {})", tool.name, server);
            }
        }
    }

    pub fn has_tools(&self) -> bool {
        !self.builtin_tools.is_empty()
            || self.sdk_tools.as_ref().map(|s| !s.tool_definitions().is_empty()).unwrap_or(false)
            || !self.mcp_manager.is_empty()
    }
}
