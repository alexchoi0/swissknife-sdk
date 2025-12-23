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

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Market not found: {0}")]
    MarketNotFound(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),

    #[error("Rate limited")]
    RateLimited,
}

pub type Result<T> = std::result::Result<T, Error>;
