use crate::Result;
use super::types::*;
use super::{ToolProvider, ResourceProvider, PromptProvider};
use std::sync::Arc;

pub struct McpRouter {
    name: String,
    version: String,
    instructions: Option<String>,
    tool_providers: Vec<Arc<dyn ToolProvider>>,
    resource_providers: Vec<Arc<dyn ResourceProvider>>,
    prompt_providers: Vec<Arc<dyn PromptProvider>>,
}

impl McpRouter {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            instructions: None,
            tool_providers: Vec::new(),
            resource_providers: Vec::new(),
            prompt_providers: Vec::new(),
        }
    }

    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    pub fn with_tool_provider<T: ToolProvider + 'static>(mut self, provider: T) -> Self {
        self.tool_providers.push(Arc::new(provider));
        self
    }

    pub fn with_resource_provider<R: ResourceProvider + 'static>(mut self, provider: R) -> Self {
        self.resource_providers.push(Arc::new(provider));
        self
    }

    pub fn with_prompt_provider<P: PromptProvider + 'static>(mut self, provider: P) -> Self {
        self.prompt_providers.push(Arc::new(provider));
        self
    }

    pub fn add_tool_provider<T: ToolProvider + 'static>(&mut self, provider: T) {
        self.tool_providers.push(Arc::new(provider));
    }

    pub fn add_resource_provider<R: ResourceProvider + 'static>(&mut self, provider: R) {
        self.resource_providers.push(Arc::new(provider));
    }

    pub fn add_prompt_provider<P: PromptProvider + 'static>(&mut self, provider: P) {
        self.prompt_providers.push(Arc::new(provider));
    }

    pub fn server_info(&self) -> ServerInfo {
        ServerInfo {
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }

    pub fn instructions(&self) -> Option<String> {
        self.instructions.clone()
    }

    pub fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: if self.tool_providers.is_empty() {
                None
            } else {
                Some(ToolsCapability { list_changed: Some(true) })
            },
            resources: if self.resource_providers.is_empty() {
                None
            } else {
                Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(true),
                })
            },
            prompts: if self.prompt_providers.is_empty() {
                None
            } else {
                Some(PromptsCapability { list_changed: Some(true) })
            },
            logging: Some(LoggingCapability {}),
        }
    }

    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tool_providers
            .iter()
            .flat_map(|p| p.tools())
            .collect()
    }

    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        for provider in &self.tool_providers {
            for tool in provider.tools() {
                if tool.name == name {
                    return provider.call(name, arguments).await;
                }
            }
        }

        Ok(ToolResult::error(format!("Tool not found: {}", name)))
    }

    pub fn list_resources(&self) -> Vec<ResourceDefinition> {
        self.resource_providers
            .iter()
            .flat_map(|p| p.resources())
            .collect()
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContent> {
        for provider in &self.resource_providers {
            for resource in provider.resources() {
                if resource.uri == uri || uri.starts_with(&resource.uri) {
                    return provider.read(uri).await;
                }
            }
        }

        Err(crate::Error::Api {
            message: format!("Resource not found: {}", uri),
            code: Some("NOT_FOUND".to_string()),
        })
    }

    pub fn list_prompts(&self) -> Vec<PromptDefinition> {
        self.prompt_providers
            .iter()
            .flat_map(|p| p.prompts())
            .collect()
    }

    pub async fn get_prompt(&self, name: &str, arguments: serde_json::Value) -> Result<PromptContent> {
        for provider in &self.prompt_providers {
            for prompt in provider.prompts() {
                if prompt.name == name {
                    return provider.get(name, arguments).await;
                }
            }
        }

        Err(crate::Error::Api {
            message: format!("Prompt not found: {}", name),
            code: Some("NOT_FOUND".to_string()),
        })
    }
}

impl Default for McpRouter {
    fn default() -> Self {
        Self::new("swissknife-mcp", env!("CARGO_PKG_VERSION"))
    }
}
