use rmcp::{
    model::{CallToolRequestParam, Tool},
    service::{RoleClient, Peer},
    ServerHandler,
};

use super::server::{serve_duplex, DuplexConnection};

pub struct McpHost {
    _server_handle: tokio::task::JoinHandle<()>,
    peer: Peer<RoleClient>,
    tools: Vec<Tool>,
}

impl McpHost {
    pub async fn new<S: ServerHandler + Send + 'static>(
        server: S,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let DuplexConnection { server_handle, peer } = serve_duplex(server).await?;
        let tools = peer.list_all_tools().await?;

        Ok(Self {
            _server_handle: server_handle,
            peer,
            tools,
        })
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<String, rmcp::service::ServiceError> {
        let result = self.peer.call_tool(CallToolRequestParam {
            name: name.to_string().into(),
            arguments,
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
