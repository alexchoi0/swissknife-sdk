use crate::{
    Action, App, AutomationProvider, Connection, ConnectionStatus, CreateWorkflow, Error,
    Execution, ExecutionError, ExecutionFilter, ExecutionStatus, ListOptions, ListResult, Result,
    Trigger, TriggerType, TriggerWorkflowRequest, UpdateWorkflow, Workflow, WorkflowStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.tray.io/core/v1";

pub struct TrayClient {
    access_token: String,
    http: reqwest::Client,
}

impl TrayClient {
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(path.to_string()));
        }
        if resp.status() == 401 {
            return Err(Error::Auth("Invalid access token".to_string()));
        }
        if resp.status() == 429 {
            return Err(Error::RateLimited);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(resp.json().await?)
    }

    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid access token".to_string()));
        }
        if resp.status() == 429 {
            return Err(Error::RateLimited);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(resp.json().await?)
    }

    async fn patch<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid access token".to_string()));
        }
        if resp.status() == 429 {
            return Err(Error::RateLimited);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(resp.json().await?)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid access token".to_string()));
        }
        if resp.status() == 429 {
            return Err(Error::RateLimited);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(())
    }

    pub async fn list_solutions(&self) -> Result<Vec<TraySolution>> {
        #[derive(Deserialize)]
        struct SolutionsResponse {
            data: Vec<TraySolution>,
        }

        let resp: SolutionsResponse = self.get("/solutions").await?;
        Ok(resp.data)
    }

    pub async fn get_solution(&self, id: &str) -> Result<TraySolution> {
        #[derive(Deserialize)]
        struct SolutionResponse {
            data: TraySolution,
        }

        let resp: SolutionResponse = self.get(&format!("/solutions/{}", id)).await?;
        Ok(resp.data)
    }

    pub async fn list_workflows(&self) -> Result<Vec<TrayWorkflow>> {
        #[derive(Deserialize)]
        struct WorkflowsResponse {
            data: Vec<TrayWorkflow>,
        }

        let resp: WorkflowsResponse = self.get("/workflows").await?;
        Ok(resp.data)
    }

    pub async fn get_workflow(&self, id: &str) -> Result<TrayWorkflow> {
        #[derive(Deserialize)]
        struct WorkflowResponse {
            data: TrayWorkflow,
        }

        let resp: WorkflowResponse = self.get(&format!("/workflows/{}", id)).await?;
        Ok(resp.data)
    }

    pub async fn list_workflow_logs(&self, workflow_id: &str) -> Result<Vec<TrayExecution>> {
        #[derive(Deserialize)]
        struct LogsResponse {
            data: Vec<TrayExecution>,
        }

        let resp: LogsResponse = self
            .get(&format!("/workflows/{}/logs", workflow_id))
            .await?;
        Ok(resp.data)
    }

    pub async fn run_workflow(&self, workflow_id: &str, data: serde_json::Value) -> Result<TrayRunResult> {
        self.post(&format!("/workflows/{}/run", workflow_id), &data)
            .await
    }

    pub async fn list_authentications(&self) -> Result<Vec<TrayAuthentication>> {
        #[derive(Deserialize)]
        struct AuthsResponse {
            data: Vec<TrayAuthentication>,
        }

        let resp: AuthsResponse = self.get("/authentications").await?;
        Ok(resp.data)
    }

    pub async fn list_connectors(&self) -> Result<Vec<TrayConnector>> {
        #[derive(Deserialize)]
        struct ConnectorsResponse {
            data: Vec<TrayConnector>,
        }

        let resp: ConnectorsResponse = self.get("/connectors").await?;
        Ok(resp.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraySolution {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayWorkflow {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub trigger_type: Option<String>,
    pub trigger_url: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub steps: Option<Vec<TrayStep>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayStep {
    pub id: String,
    pub name: Option<String>,
    pub connector_name: Option<String>,
    pub operation_name: Option<String>,
}

impl From<TrayWorkflow> for Workflow {
    fn from(w: TrayWorkflow) -> Self {
        let trigger_type = w.trigger_type.as_deref().map(|t| match t {
            "webhook" | "manual" => TriggerType::Webhook,
            "scheduled" | "cron" => TriggerType::Schedule,
            "polling" => TriggerType::Polling,
            _ => TriggerType::Other,
        }).unwrap_or(TriggerType::Other);

        let trigger = Some(Trigger {
            id: w.id.clone(),
            name: "Trigger".to_string(),
            trigger_type,
            app: None,
            event: w.trigger_type.clone(),
            config: w.trigger_url.map(|url| {
                let mut config = HashMap::new();
                config.insert("url".to_string(), serde_json::Value::String(url));
                config
            }).unwrap_or_default(),
        });

        let actions: Vec<Action> = w.steps.unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, s)| Action {
                id: s.id,
                name: s.name.unwrap_or_else(|| s.operation_name.clone().unwrap_or_default()),
                app: s.connector_name,
                action_type: s.operation_name,
                position: Some(i as i32),
                config: HashMap::new(),
            })
            .collect();

        Workflow {
            id: w.id,
            name: w.name.unwrap_or_default(),
            description: w.description,
            status: if w.enabled {
                WorkflowStatus::Active
            } else {
                WorkflowStatus::Inactive
            },
            trigger,
            actions,
            folder_id: None,
            created_at: w
                .created
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updated_at: w
                .updated
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayExecution {
    pub id: String,
    pub workflow_id: Option<String>,
    pub status: Option<String>,
    pub started: Option<String>,
    pub finished: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<TrayError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrayError {
    pub message: Option<String>,
    pub step_id: Option<String>,
}

impl From<TrayExecution> for Execution {
    fn from(e: TrayExecution) -> Self {
        let status = e.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "success" | "succeeded" | "completed" => ExecutionStatus::Success,
            "failed" | "error" => ExecutionStatus::Failed,
            "running" | "in_progress" => ExecutionStatus::Running,
            "cancelled" | "canceled" => ExecutionStatus::Cancelled,
            "waiting" => ExecutionStatus::Waiting,
            _ => ExecutionStatus::Other,
        }).unwrap_or(ExecutionStatus::Other);

        Execution {
            id: e.id,
            workflow_id: e.workflow_id.unwrap_or_default(),
            status,
            started_at: e
                .started
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            finished_at: e
                .finished
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            duration_ms: e.duration_ms,
            trigger_data: None,
            result: None,
            error: e.error.map(|err| ExecutionError {
                code: None,
                message: err.message.unwrap_or_default(),
                step_id: err.step_id,
                details: None,
            }),
            steps: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayRunResult {
    pub execution_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayAuthentication {
    pub id: String,
    pub name: Option<String>,
    pub connector_name: Option<String>,
    pub status: Option<String>,
    pub created: Option<String>,
}

impl From<TrayAuthentication> for Connection {
    fn from(a: TrayAuthentication) -> Self {
        let status = a.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "active" | "valid" => ConnectionStatus::Connected,
            "expired" => ConnectionStatus::Expired,
            "error" | "invalid" => ConnectionStatus::Error,
            _ => ConnectionStatus::Other,
        }).unwrap_or(ConnectionStatus::Connected);

        Connection {
            id: a.id,
            name: a.name.unwrap_or_default(),
            app: a.connector_name.unwrap_or_default(),
            status,
            account_name: None,
            created_at: a
                .created
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            expires_at: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayConnector {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub version: Option<String>,
}

impl From<TrayConnector> for App {
    fn from(c: TrayConnector) -> Self {
        App {
            id: c.name.clone(),
            name: c.title.unwrap_or(c.name),
            slug: None,
            description: c.description,
            icon_url: c.icon_url,
            categories: Vec::new(),
            triggers: Vec::new(),
            actions: Vec::new(),
        }
    }
}

#[async_trait]
impl AutomationProvider for TrayClient {
    async fn list_workflows(&self, _options: ListOptions) -> Result<ListResult<Workflow>> {
        let workflows = TrayClient::list_workflows(self).await?;
        let total = workflows.len() as u32;

        Ok(ListResult {
            data: workflows.into_iter().map(|w| w.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_workflow(&self, id: &str) -> Result<Workflow> {
        let workflow = TrayClient::get_workflow(self, id).await?;
        Ok(workflow.into())
    }

    async fn create_workflow(&self, data: CreateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        struct CreateRequest {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
        }

        let body = CreateRequest {
            name: data.name,
            description: data.description,
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            data: TrayWorkflow,
        }

        let resp: CreateResponse = self.post("/workflows", &body).await?;
        Ok(resp.data.into())
    }

    async fn update_workflow(&self, id: &str, data: UpdateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        struct UpdateRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            enabled: Option<bool>,
        }

        let enabled = data.status.map(|s| s == WorkflowStatus::Active);

        let body = UpdateRequest {
            name: data.name,
            description: data.description,
            enabled,
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            data: TrayWorkflow,
        }

        let resp: UpdateResponse = self.patch(&format!("/workflows/{}", id), &body).await?;
        Ok(resp.data.into())
    }

    async fn delete_workflow(&self, id: &str) -> Result<()> {
        self.delete(&format!("/workflows/{}", id)).await
    }

    async fn activate_workflow(&self, id: &str) -> Result<Workflow> {
        let body = serde_json::json!({ "enabled": true });

        #[derive(Deserialize)]
        struct UpdateResponse {
            data: TrayWorkflow,
        }

        let resp: UpdateResponse = self.patch(&format!("/workflows/{}", id), &body).await?;
        Ok(resp.data.into())
    }

    async fn deactivate_workflow(&self, id: &str) -> Result<Workflow> {
        let body = serde_json::json!({ "enabled": false });

        #[derive(Deserialize)]
        struct UpdateResponse {
            data: TrayWorkflow,
        }

        let resp: UpdateResponse = self.patch(&format!("/workflows/{}", id), &body).await?;
        Ok(resp.data.into())
    }

    async fn trigger_workflow(&self, id: &str, data: TriggerWorkflowRequest) -> Result<Execution> {
        let result = self.run_workflow(id, data.data).await?;

        Ok(Execution {
            id: result.execution_id.unwrap_or_default(),
            workflow_id: id.to_string(),
            status: match result.status.as_deref() {
                Some("success") => ExecutionStatus::Success,
                Some("error") | Some("failed") => ExecutionStatus::Failed,
                _ => ExecutionStatus::Running,
            },
            started_at: Some(Utc::now()),
            finished_at: None,
            duration_ms: None,
            trigger_data: None,
            result: None,
            error: None,
            steps: Vec::new(),
            extra: HashMap::new(),
        })
    }

    async fn list_executions(
        &self,
        filter: ExecutionFilter,
        _options: ListOptions,
    ) -> Result<ListResult<Execution>> {
        let workflow_id = filter.workflow_id.ok_or_else(|| {
            Error::InvalidRequest("workflow_id is required for Tray.io".to_string())
        })?;

        let logs = self.list_workflow_logs(&workflow_id).await?;
        let total = logs.len() as u32;

        Ok(ListResult {
            data: logs.into_iter().map(|l| l.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Tray.io does not support fetching individual executions".to_string(),
        ))
    }

    async fn cancel_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Tray.io does not support cancelling executions via API".to_string(),
        ))
    }

    async fn retry_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Tray.io does not support retrying executions via API".to_string(),
        ))
    }

    async fn list_connections(&self, _options: ListOptions) -> Result<ListResult<Connection>> {
        let auths = self.list_authentications().await?;
        let total = auths.len() as u32;

        Ok(ListResult {
            data: auths.into_iter().map(|a| a.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_connection(&self, id: &str) -> Result<Connection> {
        let auths = self.list_authentications().await?;
        auths
            .into_iter()
            .find(|a| a.id == id)
            .map(|a| a.into())
            .ok_or_else(|| Error::NotFound(format!("Connection {}", id)))
    }

    async fn delete_connection(&self, id: &str) -> Result<()> {
        self.delete(&format!("/authentications/{}", id)).await
    }

    async fn list_apps(&self) -> Result<Vec<App>> {
        let connectors = self.list_connectors().await?;
        Ok(connectors.into_iter().map(|c| c.into()).collect())
    }

    async fn get_app(&self, id: &str) -> Result<App> {
        let connectors = self.list_connectors().await?;
        connectors
            .into_iter()
            .find(|c| c.name == id)
            .map(|c| c.into())
            .ok_or_else(|| Error::NotFound(format!("App {}", id)))
    }
}
