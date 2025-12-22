use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error: {message}")]
    Api { message: String, code: Option<String> },

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Provider error: {0}")]
    Provider(String),
}

pub type Result<T> = std::result::Result<T, Error>;
