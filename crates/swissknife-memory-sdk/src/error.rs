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

    #[error("Memory not found: {0}")]
    MemoryNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, Error>;
