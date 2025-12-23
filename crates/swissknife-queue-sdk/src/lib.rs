mod error;

pub use error::{Error, Result};

#[cfg(feature = "sqs")]
pub mod sqs;

#[cfg(feature = "kafka")]
pub mod kafka;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub body: String,
    pub receipt_handle: Option<String>,
    pub attributes: HashMap<String, String>,
    pub message_attributes: HashMap<String, MessageAttribute>,
    pub md5_of_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttribute {
    pub data_type: String,
    pub string_value: Option<String>,
    pub binary_value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default)]
pub struct SendMessageOptions {
    pub delay_seconds: Option<u32>,
    pub message_attributes: HashMap<String, MessageAttribute>,
    pub message_deduplication_id: Option<String>,
    pub message_group_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ReceiveMessageOptions {
    pub max_number_of_messages: Option<u32>,
    pub visibility_timeout: Option<u32>,
    pub wait_time_seconds: Option<u32>,
    pub attribute_names: Vec<String>,
    pub message_attribute_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueAttributes {
    pub approximate_number_of_messages: Option<u64>,
    pub approximate_number_of_messages_not_visible: Option<u64>,
    pub approximate_number_of_messages_delayed: Option<u64>,
    pub created_timestamp: Option<String>,
    pub last_modified_timestamp: Option<String>,
    pub visibility_timeout: Option<u32>,
    pub maximum_message_size: Option<u32>,
    pub message_retention_period: Option<u32>,
    pub delay_seconds: Option<u32>,
    pub receive_message_wait_time_seconds: Option<u32>,
    pub redrive_policy: Option<String>,
    pub fifo_queue: Option<bool>,
    pub content_based_deduplication: Option<bool>,
}

#[async_trait]
pub trait MessageQueue: Send + Sync {
    async fn send_message(&self, queue_url: &str, body: &str, options: &SendMessageOptions) -> Result<String>;
    async fn send_message_batch(&self, queue_url: &str, messages: &[(&str, SendMessageOptions)]) -> Result<Vec<String>>;
    async fn receive_messages(&self, queue_url: &str, options: &ReceiveMessageOptions) -> Result<Vec<Message>>;
    async fn delete_message(&self, queue_url: &str, receipt_handle: &str) -> Result<()>;
    async fn delete_message_batch(&self, queue_url: &str, receipt_handles: &[&str]) -> Result<()>;
    async fn change_message_visibility(&self, queue_url: &str, receipt_handle: &str, visibility_timeout: u32) -> Result<()>;
    async fn get_queue_attributes(&self, queue_url: &str) -> Result<QueueAttributes>;
    async fn purge_queue(&self, queue_url: &str) -> Result<()>;
}
