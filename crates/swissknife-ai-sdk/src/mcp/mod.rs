pub mod server;
pub mod types;
pub mod provider;
pub mod providers;

#[cfg(feature = "mcp-tools")]
pub mod tools;

#[cfg(feature = "mcp-inprocess")]
mod host;

#[cfg(feature = "cli")]
pub mod cli;

pub use rmcp::{
    ServerHandler,
    ServiceExt,
    tool,
    model::{
        CallToolResult,
        Content,
        Tool,
        ServerCapabilities,
        ServerInfo,
        Implementation,
    },
    schemars,
};

pub use rmcp::tool_router;

#[cfg(feature = "mcp-inprocess")]
pub use host::McpHost;
pub use provider::{ToolProvider, ResourceProvider, PromptProvider};
