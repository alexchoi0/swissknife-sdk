use rmcp::{
    model::{CallToolRequestParam, Tool},
    service::{RoleClient, RunningService, ServiceError},
    transport::TokioChildProcess,
};
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
        let result = self.peer.call_tool(CallToolRequestParam {
            name: tool_name.into(),
            arguments: args,
        }).await?;

        let content: Vec<String> = result.content.into_iter().map(|c| {
            match c.raw {
                rmcp::model::RawContent::Text(t) => t.text,
                rmcp::model::RawContent::Image(i) => format!("[Image: {}]", i.mime_type),
                rmcp::model::RawContent::Audio(a) => format!("[Audio: {}]", a.mime_type),
                rmcp::model::RawContent::Resource(r) => format!("[Resource: {:?}]", r.resource),
                rmcp::model::RawContent::ResourceLink(r) => format!("[ResourceLink: {:?}]", r),
            }
        }).collect();

        Ok(content.join("\n"))
    }
}

pub struct McpClientManager {
    clients: Vec<McpClient>,
}

impl McpClientManager {
    pub fn new() -> Self {
        Self { clients: Vec::new() }
    }

    pub async fn add_server(&mut self, name: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = McpClient::spawn(name, command).await?;
        eprintln!("Connected to MCP server '{}': {} tools available", name, client.tools().len());
        for tool in client.tools() {
            eprintln!("  - {}", tool.name);
        }
        self.clients.push(client);
        Ok(())
    }

    pub fn all_tools(&self) -> Vec<(&str, &Tool)> {
        self.clients.iter()
            .flat_map(|c| c.tools().iter().map(move |t| (c.name(), t)))
            .collect()
    }

    pub fn find_tool(&self, name: &str) -> Option<(&McpClient, &Tool)> {
        for client in &self.clients {
            if let Some(tool) = client.tools().iter().find(|t| t.name == name) {
                return Some((client, tool));
            }
        }
        None
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<String, String> {
        let (client, _) = self.find_tool(name)
            .ok_or_else(|| format!("Tool '{}' not found in any MCP server", name))?;

        client.call_tool(name, arguments).await
            .map_err(|e| e.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}
