use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SecretaryError {
    #[error("configuration error: {message}")]
    Config { message: String, key: Option<String> },

    #[error("database error: {context}")]
    Database {
        context: String,
        #[source]
        source: swissknife_ai_sdk::Error,
    },

    #[error("API error: {context}")]
    Api {
        context: String,
        model: Option<String>,
        #[source]
        source: swissknife_ai_sdk::Error,
    },

    #[error("embedding error: {context}")]
    Embedding {
        context: String,
        query_preview: Option<String>,
        #[source]
        source: swissknife_ai_sdk::Error,
    },

    #[error("MCP error: {message}")]
    Mcp {
        message: String,
        server: Option<String>,
        tool: Option<String>,
    },

    #[error("tool execution failed: {tool_name}")]
    Tool {
        tool_name: String,
        details: String,
        arguments_preview: Option<String>,
    },

    #[error("IO error: {context}")]
    Io {
        context: String,
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },

    #[error("serialization error: {context}")]
    Serialization {
        context: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("TOML error: {context}")]
    Toml {
        context: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("security violation: {violation}")]
    Security {
        violation: String,
        path: Option<String>,
        url: Option<String>,
    },

    #[error("session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("operation timed out: {operation} (elapsed: {elapsed:?}, limit: {limit:?})")]
    Timeout {
        operation: String,
        elapsed: Duration,
        limit: Duration,
    },

    #[error("DNS resolution failed for {host}")]
    DnsResolution {
        host: String,
        #[source]
        source: std::io::Error,
    },

    #[error("path security violation: {violation}")]
    PathTraversal {
        violation: String,
        path: PathBuf,
        resolved: Option<PathBuf>,
    },

    #[error("symlink escape: {path} -> {target}")]
    SymlinkEscape { path: PathBuf, target: PathBuf },

    #[error("hardlink to sensitive file: {path} (inode: {inode})")]
    HardlinkEscape { path: PathBuf, inode: u64 },

    #[error("HTTP redirect blocked: {from} -> {to}")]
    RedirectBlocked { from: String, to: String },
}

impl SecretaryError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: None,
        }
    }

    pub fn config_key(message: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            key: Some(key.into()),
        }
    }

    pub fn database(context: impl Into<String>, source: swissknife_ai_sdk::Error) -> Self {
        Self::Database {
            context: context.into(),
            source,
        }
    }

    pub fn api(context: impl Into<String>, source: swissknife_ai_sdk::Error) -> Self {
        Self::Api {
            context: context.into(),
            model: None,
            source,
        }
    }

    pub fn api_with_model(
        context: impl Into<String>,
        model: impl Into<String>,
        source: swissknife_ai_sdk::Error,
    ) -> Self {
        Self::Api {
            context: context.into(),
            model: Some(model.into()),
            source,
        }
    }

    pub fn embedding(context: impl Into<String>, source: swissknife_ai_sdk::Error) -> Self {
        Self::Embedding {
            context: context.into(),
            query_preview: None,
            source,
        }
    }

    pub fn embedding_with_preview(
        context: impl Into<String>,
        query_preview: impl Into<String>,
        source: swissknife_ai_sdk::Error,
    ) -> Self {
        Self::Embedding {
            context: context.into(),
            query_preview: Some(query_preview.into()),
            source,
        }
    }

    pub fn tool(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Tool {
            tool_name: name.into(),
            details: details.into(),
            arguments_preview: None,
        }
    }

    pub fn tool_with_args(
        name: impl Into<String>,
        details: impl Into<String>,
        arguments_preview: impl Into<String>,
    ) -> Self {
        Self::Tool {
            tool_name: name.into(),
            details: details.into(),
            arguments_preview: Some(arguments_preview.into()),
        }
    }

    pub fn mcp(message: impl Into<String>) -> Self {
        Self::Mcp {
            message: message.into(),
            server: None,
            tool: None,
        }
    }

    pub fn mcp_server(message: impl Into<String>, server: impl Into<String>) -> Self {
        Self::Mcp {
            message: message.into(),
            server: Some(server.into()),
            tool: None,
        }
    }

    pub fn mcp_tool(
        message: impl Into<String>,
        server: impl Into<String>,
        tool: impl Into<String>,
    ) -> Self {
        Self::Mcp {
            message: message.into(),
            server: Some(server.into()),
            tool: Some(tool.into()),
        }
    }

    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            path: None,
            source,
        }
    }

    pub fn io_path(
        context: impl Into<String>,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io {
            context: context.into(),
            path: Some(path.into()),
            source,
        }
    }

    pub fn security(violation: impl Into<String>) -> Self {
        Self::Security {
            violation: violation.into(),
            path: None,
            url: None,
        }
    }

    pub fn security_path(violation: impl Into<String>, path: impl Into<String>) -> Self {
        Self::Security {
            violation: violation.into(),
            path: Some(path.into()),
            url: None,
        }
    }

    pub fn security_url(violation: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Security {
            violation: violation.into(),
            path: None,
            url: Some(url.into()),
        }
    }

    pub fn session_not_found(session_id: impl Into<String>) -> Self {
        Self::SessionNotFound {
            session_id: session_id.into(),
        }
    }

    pub fn timeout(operation: impl Into<String>, elapsed: Duration, limit: Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            elapsed,
            limit,
        }
    }

    pub fn dns(host: impl Into<String>, source: std::io::Error) -> Self {
        Self::DnsResolution {
            host: host.into(),
            source,
        }
    }

    pub fn path_traversal(violation: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::PathTraversal {
            violation: violation.into(),
            path: path.into(),
            resolved: None,
        }
    }

    pub fn path_traversal_resolved(
        violation: impl Into<String>,
        path: impl Into<PathBuf>,
        resolved: impl Into<PathBuf>,
    ) -> Self {
        Self::PathTraversal {
            violation: violation.into(),
            path: path.into(),
            resolved: Some(resolved.into()),
        }
    }

    pub fn symlink_escape(path: impl Into<PathBuf>, target: impl Into<PathBuf>) -> Self {
        Self::SymlinkEscape {
            path: path.into(),
            target: target.into(),
        }
    }

    pub fn hardlink_escape(path: impl Into<PathBuf>, inode: u64) -> Self {
        Self::HardlinkEscape {
            path: path.into(),
            inode,
        }
    }

    pub fn redirect_blocked(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::RedirectBlocked {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl From<std::io::Error> for SecretaryError {
    fn from(source: std::io::Error) -> Self {
        Self::Io {
            context: "IO operation failed".to_string(),
            path: None,
            source,
        }
    }
}

impl From<serde_json::Error> for SecretaryError {
    fn from(source: serde_json::Error) -> Self {
        Self::Serialization {
            context: "JSON serialization failed".to_string(),
            source,
        }
    }
}

impl From<toml::de::Error> for SecretaryError {
    fn from(source: toml::de::Error) -> Self {
        Self::Toml {
            context: "TOML parsing failed".to_string(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, SecretaryError>;

pub trait ResultExt<T> {
    fn context(self, ctx: impl Into<String>) -> Result<T>;
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> ResultExt<T> for std::result::Result<T, std::io::Error> {
    fn context(self, ctx: impl Into<String>) -> Result<T> {
        self.map_err(|source| SecretaryError::Io {
            context: ctx.into(),
            path: None,
            source,
        })
    }

    fn with_path(self, path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| SecretaryError::Io {
            context: "IO operation failed".to_string(),
            path: Some(path.into()),
            source,
        })
    }
}

impl<T> ResultExt<T> for std::result::Result<T, serde_json::Error> {
    fn context(self, ctx: impl Into<String>) -> Result<T> {
        self.map_err(|source| SecretaryError::Serialization {
            context: ctx.into(),
            source,
        })
    }

    fn with_path(self, _path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| SecretaryError::Serialization {
            context: "JSON serialization failed".to_string(),
            source,
        })
    }
}

impl<T> ResultExt<T> for std::result::Result<T, toml::de::Error> {
    fn context(self, ctx: impl Into<String>) -> Result<T> {
        self.map_err(|source| SecretaryError::Toml {
            context: ctx.into(),
            source,
        })
    }

    fn with_path(self, _path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| SecretaryError::Toml {
            context: "TOML parsing failed".to_string(),
            source,
        })
    }
}
