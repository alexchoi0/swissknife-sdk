mod stdio;

#[cfg(feature = "mcp-inprocess")]
mod duplex;

#[cfg(feature = "http")]
mod http;

pub use stdio::serve_stdio;

#[cfg(feature = "mcp-inprocess")]
pub use duplex::{serve_duplex, DuplexConnection};

#[cfg(feature = "http")]
pub use http::{
    McpHttpServer,
    McpHttpServerConfig,
    ToolInfo,
    ServerInfo,
    ToolCallRequest,
    ToolCallResponse,
    SseMessage,
};
