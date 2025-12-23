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

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
