use crate::error::Result;
use crate::types::{ToolSpec, ToolOutput};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolSpec;

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolOutput>;

    fn id(&self) -> String {
        self.definition().id
    }

    fn name(&self) -> String {
        self.definition().name
    }

    fn description(&self) -> String {
        self.definition().description
    }

    fn category(&self) -> String {
        self.definition().category
    }
}

pub trait ToolBuilder: Default {
    type Tool: Tool;

    fn build(self) -> Self::Tool;
}

pub fn get_string_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    params.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
}

pub fn get_required_string_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Result<String> {
    get_string_param(params, key).ok_or_else(|| crate::error::Error::MissingParameter(key.to_string()))
}

pub fn get_i64_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<i64> {
    params.get(key).and_then(|v| v.as_i64())
}

pub fn get_required_i64_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Result<i64> {
    get_i64_param(params, key).ok_or_else(|| crate::error::Error::MissingParameter(key.to_string()))
}

pub fn get_f64_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<f64> {
    params.get(key).and_then(|v| v.as_f64())
}

pub fn get_bool_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<bool> {
    params.get(key).and_then(|v| v.as_bool())
}

pub fn get_array_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<Vec<serde_json::Value>> {
    params.get(key).and_then(|v| v.as_array().cloned())
}

pub fn get_object_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    params.get(key).and_then(|v| v.as_object().cloned())
}
