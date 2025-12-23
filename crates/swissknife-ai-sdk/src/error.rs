use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {message}")]
    Api { message: String, code: Option<String> },

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Authentication required")]
    AuthRequired,

    #[error("Rate limited")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
