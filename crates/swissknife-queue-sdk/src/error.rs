use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("AWS error: {message}")]
    Aws {
        message: String,
        code: Option<String>,
    },

    #[error("Queue not found: {0}")]
    QueueNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Invalid receipt handle")]
    InvalidReceiptHandle,

    #[error("Batch request too large")]
    BatchTooLarge,
}

pub type Result<T> = std::result::Result<T, Error>;
