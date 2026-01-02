use crate::security::ssrf::{validate_url_for_fetch, HTTP_TIMEOUT, MAX_RESPONSE_SIZE};
use rmcp::{
    model::{CallToolRequestParam, Tool},
    service::{RoleClient, RunningService, ServiceError},
    transport::TokioChildProcess,
    ServerHandler,
};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;
use std::sync::Arc;
use swissknife_ai_sdk::mcp::McpHost;
use swissknife_search_sdk::tavily::TavilyClient;
use swissknife_search_sdk::{SearchOptions, SearchProvider};
use tokio::process::Command;

pub struct McpClient {
    name: String,
    _service: RunningService<RoleClient, ()>,
    peer: rmcp::service::Peer<RoleClient>,
    tools: Vec<Tool>,
}

impl McpClient {
    pub async fn spawn(name: &str, command: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".into());
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        let transport = TokioChildProcess::new(cmd)?;
        let service = rmcp::service::serve_client((), transport).await?;
        let peer = service.peer().clone();

        let tools = peer.list_all_tools().await?;

        Ok(Self {
            name: name.to_string(),
            _service: service,
            peer,
            tools,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<String, ServiceError> {
        let args = arguments.and_then(|v| {
            if let serde_json::Value::Object(map) = v {
                Some(map)
            } else {
                None
            }
        });

        let tool_name: String = name.to_string();
        let result = self
            .peer
            .call_tool(CallToolRequestParam {
                name: tool_name.into(),
                arguments: args,
            })
            .await?;

        let content: Vec<String> = result
            .content
            .into_iter()
            .map(|c| match c.raw {
                rmcp::model::RawContent::Text(t) => t.text,
                rmcp::model::RawContent::Image(i) => format!("[Image: {}]", i.mime_type),
                rmcp::model::RawContent::Audio(a) => format!("[Audio: {}]", a.mime_type),
                rmcp::model::RawContent::Resource(r) => format!("[Resource: {:?}]", r.resource),
                rmcp::model::RawContent::ResourceLink(r) => format!("[ResourceLink: {:?}]", r),
            })
            .collect();

        Ok(content.join("\n"))
    }
}

#[derive(Clone)]
pub struct SdkToolServer {
    tavily: Option<Arc<TavilyClient>>,
    tool_router: ToolRouter<Self>,
}

impl SdkToolServer {
    pub fn new() -> Self {
        let tavily = std::env::var("TAVILY_API_KEY")
            .ok()
            .map(|key| Arc::new(TavilyClient::new(&key)));

        if tavily.is_some() {
            eprintln!("SDK Tools: Tavily search enabled");
        }

        Self {
            tavily,
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for SdkToolServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SdkToolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "swissknife-sdk-tools".into(),
                title: Some("Swissknife SDK Tools".into()),
                version: "0.1.0".into(),
                icons: None,
                website_url: None,
            },
            instructions: Some("Swissknife SDK tools including web search".into()),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TavilySearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WebFetchRequest {
    pub url: String,
}

#[tool_router]
impl SdkToolServer {
    #[tool(description = "Search the web using Tavily AI-powered search engine")]
    pub async fn tavily_search(
        &self,
        Parameters(req): Parameters<TavilySearchRequest>,
    ) -> Result<String, String> {
        let client = self.tavily.as_ref().ok_or_else(|| {
            "Tavily not configured. Set TAVILY_API_KEY environment variable.".to_string()
        })?;

        let options = SearchOptions {
            max_results: req.max_results,
            include_answer: true,
            ..Default::default()
        };

        let response = client
            .search(&req.query, &options)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "query": response.query,
            "answer": response.answer,
            "results": response.results.iter().map(|r| serde_json::json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.snippet,
                "content": r.content
            })).collect::<Vec<_>>()
        }))
        .map_err(|e| e.to_string())
    }

    #[tool(description = "Fetch content from a URL")]
    pub async fn web_fetch(
        &self,
        Parameters(req): Parameters<WebFetchRequest>,
    ) -> Result<String, String> {
        let (validated_url, pinned_addr) = validate_url_for_fetch(&req.url).await?;

        let host = validated_url
            .host_str()
            .ok_or_else(|| "URL must have a host".to_string())?;

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(HTTP_TIMEOUT)
            .resolve(host, pinned_addr)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(validated_url.as_str())
            .header("User-Agent", "Mozilla/5.0 (compatible; Secretary/1.0)")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();

        if status.is_redirection() {
            if let Some(location) = response.headers().get("location") {
                let location_str = location
                    .to_str()
                    .unwrap_or("<invalid location header>");
                return Ok(format!(
                    "Status: {} (Redirect)\nRedirects are disabled for security. Location header: {}",
                    status, location_str
                ));
            }
            return Ok(format!(
                "Status: {} (Redirect)\nRedirects are disabled for security.",
                status
            ));
        }

        let content_length = response
            .content_length()
            .unwrap_or(0) as usize;

        if content_length > MAX_RESPONSE_SIZE {
            return Err(format!(
                "Response too large: {} bytes (max {} bytes)",
                content_length, MAX_RESPONSE_SIZE
            ));
        }

        let bytes = response.bytes().await.map_err(|e| e.to_string())?;

        if bytes.len() > MAX_RESPONSE_SIZE {
            return Err(format!(
                "Response too large: {} bytes (max {} bytes)",
                bytes.len(),
                MAX_RESPONSE_SIZE
            ));
        }

        let text = String::from_utf8_lossy(&bytes);

        if text.len() > 10000 {
            Ok(format!(
                "Status: {}\nContent (truncated):\n{}",
                status,
                &text[..10000]
            ))
        } else {
            Ok(format!("Status: {}\nContent:\n{}", status, text))
        }
    }
}

pub struct McpManager {
    external_clients: Vec<McpClient>,
    sdk_host: Option<McpHost>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            external_clients: Vec::new(),
            sdk_host: None,
        }
    }

    pub async fn enable_sdk_tools(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server = SdkToolServer::new();
        let mcp = McpHost::new(server).await?;

        eprintln!("In-process MCP tools ({}):", mcp.tools().len());
        for tool in mcp.tools() {
            eprintln!("  - {}", tool.name);
        }

        self.sdk_host = Some(mcp);
        Ok(())
    }

    pub async fn add_external_server(
        &mut self,
        name: &str,
        command: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = McpClient::spawn(name, command).await?;
        eprintln!(
            "Connected to MCP server '{}': {} tools available",
            name,
            client.tools().len()
        );
        for tool in client.tools() {
            eprintln!("  - {}", tool.name);
        }
        self.external_clients.push(client);
        Ok(())
    }

    pub fn sdk_tools(&self) -> Vec<&Tool> {
        self.sdk_host
            .as_ref()
            .map(|h| h.tools().iter().collect())
            .unwrap_or_default()
    }

    pub fn external_tools(&self) -> Vec<(&str, &Tool)> {
        self.external_clients
            .iter()
            .flat_map(|c| c.tools().iter().map(move |t| (c.name(), t)))
            .collect()
    }

    pub fn find_sdk_tool(&self, name: &str) -> bool {
        self.sdk_host
            .as_ref()
            .map(|h| h.tools().iter().any(|t| t.name == name))
            .unwrap_or(false)
    }

    pub fn find_external_tool(&self, name: &str) -> Option<&McpClient> {
        for client in &self.external_clients {
            if client.tools().iter().any(|t| t.name == name) {
                return Some(client);
            }
        }
        None
    }

    pub async fn call_sdk_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<String, String> {
        let host = self.sdk_host.as_ref().ok_or("SDK tools not enabled")?;
        host.call_tool(name, arguments)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn call_external_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<String, String> {
        let client = self
            .find_external_tool(name)
            .ok_or_else(|| format!("Tool '{}' not found in any MCP server", name))?;

        client.call_tool(name, arguments).await.map_err(|e| e.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.sdk_host.is_none() && self.external_clients.is_empty()
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}
