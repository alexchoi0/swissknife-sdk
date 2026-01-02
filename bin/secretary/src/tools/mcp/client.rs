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
        let parts = shell_words::split(command)
            .map_err(|e| format!("Invalid command syntax: {}", e))?;

        if parts.is_empty() {
            return Err("Empty command".into());
        }

        let mut cmd = Command::new(&parts[0]);
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
