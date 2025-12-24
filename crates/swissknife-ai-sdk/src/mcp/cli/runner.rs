use crate::mcp::cli::args::{Cli, ServerMode, ToolCategory};
use std::net::SocketAddr;

#[cfg(feature = "http")]
use crate::mcp::server::{McpHttpServer, McpHttpServerConfig};

pub async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if cli.verbose {
        println!("Starting MCP server...");
        println!("  Mode: {:?}", cli.mode);
        println!("  Tools: {:?}", cli.tools);
    }

    match cli.mode {
        ServerMode::Http => run_http(cli).await,
        ServerMode::Stdio => run_stdio(cli).await,
    }
}

#[cfg(feature = "http")]
async fn run_http(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = McpHttpServerConfig {
        name: cli.name.clone(),
        version: cli.version.clone(),
        cors_origins: if cli.cors_origins.is_empty() {
            vec!["*".to_string()]
        } else {
            cli.cors_origins.clone()
        },
    };

    let server = McpHttpServer::new(config);

    register_tools(&server, &cli).await;

    let addr: SocketAddr = cli.addr().parse()?;
    server.serve(addr).await
}

#[cfg(not(feature = "http"))]
async fn run_http(_cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("HTTP server requires the 'http' feature".into())
}

async fn run_stdio(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::mcp::server::serve_stdio;

    let service = StdioService::new(&cli);
    serve_stdio(service).await.map_err(|e| e.into())
}

#[cfg(feature = "http")]
async fn register_tools(server: &McpHttpServer, cli: &Cli) {
    if cli.has_category(ToolCategory::Search) {
        register_search_tools(server).await;
    }
    if cli.has_category(ToolCategory::Devtools) {
        register_devtools_tools(server).await;
    }
    if cli.has_category(ToolCategory::Communication) {
        register_communication_tools(server).await;
    }
    if cli.has_category(ToolCategory::Database) {
        register_database_tools(server).await;
    }
    if cli.has_category(ToolCategory::Llm) {
        register_llm_tools(server).await;
    }

    if cli.verbose {
        println!("Tools registered successfully");
    }
}

#[cfg(feature = "http")]
async fn register_search_tools(server: &McpHttpServer) {
    server
        .register_tool(
            "web_search",
            "Search the web using configured search provider",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
            |args| async move {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing query parameter")?;

                Ok(serde_json::json!({
                    "query": query,
                    "results": [],
                    "message": "Search provider not configured"
                }))
            },
        )
        .await;
}

#[cfg(feature = "http")]
async fn register_devtools_tools(server: &McpHttpServer) {
    server
        .register_tool(
            "github_get_repo",
            "Get information about a GitHub repository",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "owner": {
                        "type": "string",
                        "description": "Repository owner"
                    },
                    "repo": {
                        "type": "string",
                        "description": "Repository name"
                    }
                },
                "required": ["owner", "repo"]
            }),
            |args| async move {
                let owner = args
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing owner parameter")?;
                let repo = args
                    .get("repo")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing repo parameter")?;

                Ok(serde_json::json!({
                    "owner": owner,
                    "repo": repo,
                    "message": "GitHub client not configured"
                }))
            },
        )
        .await;
}

#[cfg(feature = "http")]
async fn register_communication_tools(server: &McpHttpServer) {
    server
        .register_tool(
            "send_email",
            "Send an email using configured email provider",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Recipient email address"
                    },
                    "subject": {
                        "type": "string",
                        "description": "Email subject"
                    },
                    "body": {
                        "type": "string",
                        "description": "Email body"
                    }
                },
                "required": ["to", "subject", "body"]
            }),
            |args| async move {
                let to = args
                    .get("to")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'to' parameter")?;

                Ok(serde_json::json!({
                    "to": to,
                    "message": "Email provider not configured"
                }))
            },
        )
        .await;

    server
        .register_tool(
            "send_slack_message",
            "Send a message to Slack",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "channel": {
                        "type": "string",
                        "description": "Slack channel ID or name"
                    },
                    "message": {
                        "type": "string",
                        "description": "Message content"
                    }
                },
                "required": ["channel", "message"]
            }),
            |args| async move {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing channel parameter")?;

                Ok(serde_json::json!({
                    "channel": channel,
                    "message": "Slack client not configured"
                }))
            },
        )
        .await;
}

#[cfg(feature = "http")]
async fn register_database_tools(server: &McpHttpServer) {
    server
        .register_tool(
            "query_database",
            "Execute a database query",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "SQL query to execute"
                    },
                    "database": {
                        "type": "string",
                        "description": "Database identifier"
                    }
                },
                "required": ["query"]
            }),
            |args| async move {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing query parameter")?;

                Ok(serde_json::json!({
                    "query": query,
                    "message": "Database client not configured"
                }))
            },
        )
        .await;
}

#[cfg(feature = "http")]
async fn register_llm_tools(server: &McpHttpServer) {
    server
        .register_tool(
            "llm_complete",
            "Generate text completion using an LLM",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "The prompt to complete"
                    },
                    "model": {
                        "type": "string",
                        "description": "Model to use (e.g., gpt-4, claude-3)"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Maximum tokens to generate",
                        "default": 1000
                    }
                },
                "required": ["prompt"]
            }),
            |args| async move {
                let prompt = args
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing prompt parameter")?;

                Ok(serde_json::json!({
                    "prompt": prompt,
                    "message": "LLM provider not configured"
                }))
            },
        )
        .await;
}

use rmcp::{ServerHandler, model::ServerInfo};

pub struct StdioService {
    name: String,
    version: String,
}

impl StdioService {
    pub fn new(cli: &Cli) -> Self {
        Self {
            name: cli.name.clone(),
            version: cli.version.clone(),
        }
    }
}

impl ServerHandler for StdioService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            ..Default::default()
        }
    }
}
