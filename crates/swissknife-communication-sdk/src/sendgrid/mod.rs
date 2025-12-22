mod client;

#[cfg(feature = "email")]
mod email;

pub use client::SendGridClient;
