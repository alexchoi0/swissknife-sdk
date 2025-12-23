use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error: {code} - {message}")]
    Api { code: String, message: String },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Link token expired")]
    LinkTokenExpired,

    #[error("Institution not supported: {0}")]
    InstitutionNotSupported(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Consent required")]
    ConsentRequired,

    #[error("Consent expired")]
    ConsentExpired,

    #[error("MFA required: {0}")]
    MfaRequired(String),

    #[error("Provider error: {0}")]
    Provider(String),
}

pub type Result<T> = std::result::Result<T, Error>;
