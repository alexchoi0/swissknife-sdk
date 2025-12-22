use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Token error: {0}")]
    Token(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Password hash error: {0}")]
    PasswordHash(String),

    #[error("TOTP error: {0}")]
    Totp(String),

    #[error("WebAuthn error: {0}")]
    WebAuthn(String),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserExists,

    #[error("Invalid state")]
    InvalidState,

    #[error("Invalid code")]
    InvalidCode,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
