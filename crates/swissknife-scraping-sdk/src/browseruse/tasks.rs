use crate::{Error, Result};
use crate::browseruse::BrowserUseClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl BrowserUseClient {
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task> {
        let response = self.client()
            .post(format!("{}/v1/tasks", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: Task = response.json().await?;
        Ok(result)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        let response = self.client()
            .get(format!("{}/v1/tasks/{}", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: Task = response.json().await?;
        Ok(result)
    }

    pub async fn list_tasks(&self, params: Option<ListTasksParams>) -> Result<TasksResponse> {
        let mut request = self.client()
            .get(format!("{}/v1/tasks", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(status) = p.status {
                query.push(("status", status));
            }
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
            if !query.is_empty() {
                request = request.query(&query);
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: TasksResponse = response.json().await?;
        Ok(result)
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<Task> {
        let response = self.client()
            .post(format!("{}/v1/tasks/{}/cancel", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: Task = response.json().await?;
        Ok(result)
    }

    pub async fn run_task(&self, instruction: &str, options: Option<TaskOptions>) -> Result<TaskResult> {
        let mut body = serde_json::json!({
            "instruction": instruction
        });

        if let Some(opts) = options {
            if let Some(url) = opts.start_url {
                body["start_url"] = serde_json::Value::String(url);
            }
            if let Some(model) = opts.model {
                body["model"] = serde_json::Value::String(model);
            }
            if let Some(max_steps) = opts.max_steps {
                body["max_steps"] = serde_json::Value::Number(max_steps.into());
            }
            if let Some(timeout) = opts.timeout_ms {
                body["timeout"] = serde_json::Value::Number(timeout.into());
            }
            if let Some(variables) = opts.variables {
                body["variables"] = serde_json::to_value(variables).unwrap_or_default();
            }
        }

        let response = self.client()
            .post(format!("{}/v1/tasks/run", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: TaskResult = response.json().await?;
        Ok(result)
    }

    pub async fn extract_data(&self, url: &str, schema: &serde_json::Value) -> Result<ExtractResult> {
        let body = serde_json::json!({
            "url": url,
            "schema": schema
        });

        let response = self.client()
            .post(format!("{}/v1/extract", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ExtractResult = response.json().await?;
        Ok(result)
    }

    pub async fn extract_with_prompt(&self, url: &str, prompt: &str) -> Result<ExtractResult> {
        let body = serde_json::json!({
            "url": url,
            "prompt": prompt
        });

        let response = self.client()
            .post(format!("{}/v1/extract", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ExtractResult = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskRequest {
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct ListTasksParams {
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskOptions {
    pub start_url: Option<String>,
    pub model: Option<String>,
    pub max_steps: Option<u32>,
    pub timeout_ms: Option<u32>,
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: String,
    pub instruction: String,
    pub status: TaskStatus,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub steps: Option<Vec<TaskStep>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskStep {
    pub action: String,
    pub selector: Option<String>,
    pub value: Option<String>,
    pub thought: Option<String>,
    pub screenshot: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
    pub total: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub steps: Option<Vec<TaskStep>>,
    pub final_url: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractResult {
    pub data: serde_json::Value,
    pub url: String,
}
