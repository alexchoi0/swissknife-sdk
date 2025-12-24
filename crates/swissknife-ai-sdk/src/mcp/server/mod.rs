mod stdio;

#[cfg(feature = "http")]
mod http;

pub use stdio::serve_stdio;

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
