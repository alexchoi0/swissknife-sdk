mod client;
mod server;

pub use client::McpClient;
pub use server::SdkToolServer;

use rmcp::model::Tool;
use std::collections::HashMap;
use swissknife_ai_sdk::mcp::McpHost;

pub enum ToolSource {
    Sdk,
    External(usize),
}

pub struct McpManager {
    external_clients: Vec<McpClient>,
    sdk_host: Option<McpHost>,
    tool_index: HashMap<String, ToolSource>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            external_clients: Vec::new(),
            sdk_host: None,
            tool_index: HashMap::new(),
        }
    }

    fn rebuild_index(&mut self) {
        self.tool_index.clear();
        if let Some(host) = &self.sdk_host {
            for tool in host.tools() {
                self.tool_index.insert(tool.name.to_string(), ToolSource::Sdk);
            }
        }
        for (idx, client) in self.external_clients.iter().enumerate() {
            for tool in client.tools() {
                self.tool_index.insert(tool.name.to_string(), ToolSource::External(idx));
            }
        }
    }

    pub fn find_tool(&self, name: &str) -> Option<&ToolSource> {
        self.tool_index.get(name)
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
        self.rebuild_index();
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
        self.rebuild_index();
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
        self.external_clients
            .iter()
            .find(|client| client.tools().iter().any(|t| t.name == name))
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
