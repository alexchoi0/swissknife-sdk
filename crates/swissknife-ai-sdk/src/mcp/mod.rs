pub mod server;
pub mod tools;

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

pub use rmcp::tool_box;
