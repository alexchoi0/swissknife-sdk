use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {message}")]
    Api {
        message: String,
        code: Option<String>,
    },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Actor not found: {0}")]
    ActorNotFound(String),

    #[error("Run failed: {0}")]
    RunFailed(String),
}

pub type Result<T> = std::result::Result<T, Error>;
