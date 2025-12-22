use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error: {code} - {message}")]
    Api { code: i32, message: String },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Rate limited: retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Media upload failed: {0}")]
    MediaUpload(String),
}

pub type Result<T> = std::result::Result<T, Error>;
