mod client;

#[cfg(feature = "chat")]
mod chat;

pub use client::{SlackClient, SlackWebhookClient};
