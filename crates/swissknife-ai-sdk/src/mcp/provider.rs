use async_trait::async_trait;
use crate::Result;
use super::types::{ToolDefinition, ToolResult, ResourceDefinition, ResourceContent, PromptDefinition, PromptContent};

#[async_trait]
pub trait ToolProvider: Send + Sync {
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn call(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult>;
}

#[async_trait]
pub trait ResourceProvider: Send + Sync {
    fn resources(&self) -> Vec<ResourceDefinition>;
    async fn read(&self, uri: &str) -> Result<ResourceContent>;
}

#[async_trait]
pub trait PromptProvider: Send + Sync {
    fn prompts(&self) -> Vec<PromptDefinition>;
    async fn get(&self, name: &str, arguments: serde_json::Value) -> Result<PromptContent>;
}
