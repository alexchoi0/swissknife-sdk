mod error;

pub use error::{Error, Result};

#[cfg(feature = "zapier")]
pub mod zapier;

#[cfg(feature = "make")]
pub mod make;

#[cfg(feature = "n8n")]
pub mod n8n;

#[cfg(feature = "pipedream")]
pub mod pipedream;

#[cfg(feature = "tray")]
pub mod tray;

#[cfg(feature = "workato")]
pub mod workato;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: WorkflowStatus,
    pub trigger: Option<Trigger>,
    pub actions: Vec<Action>,
    pub folder_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    Active,
    Inactive,
    Draft,
    Paused,
    Error,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub name: String,
    pub trigger_type: TriggerType,
    pub app: Option<String>,
    pub event: Option<String>,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    Webhook,
    Schedule,
    Polling,
    AppEvent,
    Manual,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub app: Option<String>,
    pub action_type: Option<String>,
    pub position: Option<i32>,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub trigger_data: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<ExecutionError>,
    pub steps: Vec<ExecutionStep>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Running,
    Success,
    Failed,
    Cancelled,
    Waiting,
    Retrying,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: String,
    pub action_id: Option<String>,
    pub name: String,
    pub status: ExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    pub code: Option<String>,
    pub message: String,
    pub step_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub app: String,
    pub status: ConnectionStatus,
    pub account_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Expired,
    Error,
    Pending,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    pub triggers: Vec<AppTrigger>,
    pub actions: Vec<AppAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTrigger {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAction {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub workflow_id: String,
    pub received_at: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
    pub processed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub workflow_count: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub cursor: Option<String>,
    pub status: Option<WorkflowStatus>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListResult<T> {
    pub data: Vec<T>,
    pub total: Option<u32>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateWorkflow {
    pub name: String,
    pub description: Option<String>,
    pub trigger: Option<CreateTrigger>,
    pub actions: Vec<CreateAction>,
    pub folder_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTrigger {
    pub trigger_type: TriggerType,
    pub app: Option<String>,
    pub event: Option<String>,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAction {
    pub app: String,
    pub action_type: String,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateWorkflow {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<WorkflowStatus>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerWorkflowRequest {
    pub data: serde_json::Value,
    pub async_execution: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFilter {
    pub workflow_id: Option<String>,
    pub status: Option<ExecutionStatus>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait AutomationProvider: Send + Sync {
    async fn list_workflows(&self, options: ListOptions) -> Result<ListResult<Workflow>>;
    async fn get_workflow(&self, id: &str) -> Result<Workflow>;
    async fn create_workflow(&self, data: CreateWorkflow) -> Result<Workflow>;
    async fn update_workflow(&self, id: &str, data: UpdateWorkflow) -> Result<Workflow>;
    async fn delete_workflow(&self, id: &str) -> Result<()>;

    async fn activate_workflow(&self, id: &str) -> Result<Workflow>;
    async fn deactivate_workflow(&self, id: &str) -> Result<Workflow>;

    async fn trigger_workflow(&self, id: &str, data: TriggerWorkflowRequest) -> Result<Execution>;

    async fn list_executions(&self, filter: ExecutionFilter, options: ListOptions) -> Result<ListResult<Execution>>;
    async fn get_execution(&self, id: &str) -> Result<Execution>;
    async fn cancel_execution(&self, id: &str) -> Result<Execution>;
    async fn retry_execution(&self, id: &str) -> Result<Execution>;

    async fn list_connections(&self, options: ListOptions) -> Result<ListResult<Connection>>;
    async fn get_connection(&self, id: &str) -> Result<Connection>;
    async fn delete_connection(&self, id: &str) -> Result<()>;

    async fn list_apps(&self) -> Result<Vec<App>>;
    async fn get_app(&self, id: &str) -> Result<App>;
}
