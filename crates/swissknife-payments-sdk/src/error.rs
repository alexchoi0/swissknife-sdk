use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Payment declined: {0}")]
    Declined(String),

    #[error("Invalid card: {0}")]
    InvalidCard(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Webhook verification failed: {0}")]
    WebhookVerification(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
