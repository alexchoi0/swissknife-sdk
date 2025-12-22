use crate::{
    Action, App, AutomationProvider, Connection, ConnectionStatus, CreateWorkflow, Error,
    Execution, ExecutionError, ExecutionFilter, ExecutionStatus, ListOptions, ListResult, Result,
    Trigger, TriggerType, TriggerWorkflowRequest, UpdateWorkflow, Workflow, WorkflowStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.pipedream.com/v1";

pub struct PipedreamClient {
    api_key: String,
    http: reqwest::Client,
}

impl PipedreamClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(path.to_string()));
        }
        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API key".to_string()));
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API key".to_string()));
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

    async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API key".to_string()));
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API key".to_string()));
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

    pub async fn list_workflows(&self) -> Result<Vec<PdWorkflow>> {
        #[derive(Deserialize)]
        struct WorkflowsResponse {
            data: Vec<PdWorkflow>,
        }

        let resp: WorkflowsResponse = self.get("/users/me/workflows").await?;
        Ok(resp.data)
    }

    pub async fn get_workflow(&self, id: &str) -> Result<PdWorkflow> {
        #[derive(Deserialize)]
        struct WorkflowResponse {
            data: PdWorkflow,
        }

        let resp: WorkflowResponse = self.get(&format!("/workflows/{}", id)).await?;
        Ok(resp.data)
    }

    pub async fn list_workflow_events(&self, workflow_id: &str) -> Result<Vec<PdEvent>> {
        #[derive(Deserialize)]
        struct EventsResponse {
            data: Vec<PdEvent>,
        }

        let resp: EventsResponse = self
            .get(&format!("/workflows/{}/event_summaries", workflow_id))
            .await?;
        Ok(resp.data)
    }

    pub async fn trigger_workflow(&self, workflow_id: &str, data: serde_json::Value) -> Result<PdTriggerResponse> {
        self.post(&format!("/workflows/{}/trigger", workflow_id), &data)
            .await
    }

    pub async fn list_sources(&self) -> Result<Vec<PdSource>> {
        #[derive(Deserialize)]
        struct SourcesResponse {
            data: Vec<PdSource>,
        }

        let resp: SourcesResponse = self.get("/users/me/sources").await?;
        Ok(resp.data)
    }

    pub async fn list_connected_accounts(&self) -> Result<Vec<PdAccount>> {
        #[derive(Deserialize)]
        struct AccountsResponse {
            data: Vec<PdAccount>,
        }

        let resp: AccountsResponse = self.get("/users/me/accounts").await?;
        Ok(resp.data)
    }

    pub async fn list_apps(&self) -> Result<Vec<PdApp>> {
        #[derive(Deserialize)]
        struct AppsResponse {
            data: Vec<PdApp>,
        }

        let resp: AppsResponse = self.get("/apps").await?;
        Ok(resp.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdWorkflow {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub trigger: Option<PdTrigger>,
    pub steps: Option<Vec<PdStep>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdTrigger {
    pub id: Option<String>,
    pub component: Option<String>,
    pub trigger_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdStep {
    pub id: Option<String>,
    pub namespace: Option<String>,
    pub component: Option<String>,
    pub name: Option<String>,
}

impl From<PdWorkflow> for Workflow {
    fn from(w: PdWorkflow) -> Self {
        let trigger = w.trigger.map(|t| {
            let trigger_type = t
                .trigger_type
                .as_deref()
                .map(|tt| match tt {
                    "http" => TriggerType::Webhook,
                    "timer" | "schedule" => TriggerType::Schedule,
                    _ => TriggerType::AppEvent,
                })
                .unwrap_or(TriggerType::Other);

            Trigger {
                id: t.id.unwrap_or_default(),
                name: t.component.clone().unwrap_or_else(|| "Trigger".to_string()),
                trigger_type,
                app: t.component,
                event: None,
                config: HashMap::new(),
            }
        });

        let actions: Vec<Action> = w
            .steps
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, s)| Action {
                id: s.id.unwrap_or_else(|| i.to_string()),
                name: s.name.unwrap_or_else(|| s.component.clone().unwrap_or_default()),
                app: s.namespace,
                action_type: s.component,
                position: Some(i as i32),
                config: HashMap::new(),
            })
            .collect();

        Workflow {
            id: w.id,
            name: w.name.unwrap_or_default(),
            description: w.description,
            status: if w.active {
                WorkflowStatus::Active
            } else {
                WorkflowStatus::Inactive
            },
            trigger,
            actions,
            folder_id: None,
            created_at: w
                .created_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updated_at: w
                .updated_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdEvent {
    pub id: String,
    pub indexed_at_ms: Option<u64>,
    pub event: Option<serde_json::Value>,
    pub metadata: Option<PdEventMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdEventMetadata {
    pub emit_id: Option<String>,
    pub name: Option<String>,
}

impl From<PdEvent> for Execution {
    fn from(e: PdEvent) -> Self {
        let started_at = e
            .indexed_at_ms
            .map(|ms| DateTime::from_timestamp_millis(ms as i64).unwrap_or_default());

        Execution {
            id: e.id,
            workflow_id: String::new(),
            status: ExecutionStatus::Success,
            started_at,
            finished_at: started_at,
            duration_ms: None,
            trigger_data: e.event,
            result: None,
            error: None,
            steps: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdTriggerResponse {
    pub success: bool,
    pub execution_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdSource {
    pub id: String,
    pub name: Option<String>,
    pub component_id: Option<String>,
    pub active: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdAccount {
    pub id: String,
    pub name: Option<String>,
    pub app: Option<PdAccountApp>,
    pub healthy: Option<bool>,
    pub dead: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdAccountApp {
    pub id: Option<String>,
    pub name_slug: Option<String>,
}

impl From<PdAccount> for Connection {
    fn from(a: PdAccount) -> Self {
        let status = if a.dead == Some(true) {
            ConnectionStatus::Disconnected
        } else if a.healthy == Some(false) {
            ConnectionStatus::Error
        } else {
            ConnectionStatus::Connected
        };

        Connection {
            id: a.id,
            name: a.name.unwrap_or_default(),
            app: a.app.and_then(|app| app.name_slug).unwrap_or_default(),
            status,
            account_name: None,
            created_at: a
                .created_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            expires_at: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdApp {
    pub id: String,
    pub name_slug: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub img_src: Option<String>,
    pub categories: Option<Vec<String>>,
}

impl From<PdApp> for App {
    fn from(a: PdApp) -> Self {
        App {
            id: a.id,
            name: a.name.unwrap_or_default(),
            slug: a.name_slug,
            description: a.description,
            icon_url: a.img_src,
            categories: a.categories.unwrap_or_default(),
            triggers: Vec::new(),
            actions: Vec::new(),
        }
    }
}

#[async_trait]
impl AutomationProvider for PipedreamClient {
    async fn list_workflows(&self, _options: ListOptions) -> Result<ListResult<Workflow>> {
        let workflows = PipedreamClient::list_workflows(self).await?;
        let total = workflows.len() as u32;

        Ok(ListResult {
            data: workflows.into_iter().map(|w| w.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_workflow(&self, id: &str) -> Result<Workflow> {
        let workflow = PipedreamClient::get_workflow(self, id).await?;
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
            data: PdWorkflow,
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
            active: Option<bool>,
        }

        let active = data.status.map(|s| s == WorkflowStatus::Active);

        let body = UpdateRequest {
            name: data.name,
            description: data.description,
            active,
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            data: PdWorkflow,
        }

        let resp: UpdateResponse = self.put(&format!("/workflows/{}", id), &body).await?;
        Ok(resp.data.into())
    }

    async fn delete_workflow(&self, id: &str) -> Result<()> {
        self.delete(&format!("/workflows/{}", id)).await
    }

    async fn activate_workflow(&self, id: &str) -> Result<Workflow> {
        let body = serde_json::json!({ "active": true });

        #[derive(Deserialize)]
        struct UpdateResponse {
            data: PdWorkflow,
        }

        let resp: UpdateResponse = self.put(&format!("/workflows/{}", id), &body).await?;
        Ok(resp.data.into())
    }

    async fn deactivate_workflow(&self, id: &str) -> Result<Workflow> {
        let body = serde_json::json!({ "active": false });

        #[derive(Deserialize)]
        struct UpdateResponse {
            data: PdWorkflow,
        }

        let resp: UpdateResponse = self.put(&format!("/workflows/{}", id), &body).await?;
        Ok(resp.data.into())
    }

    async fn trigger_workflow(&self, id: &str, data: TriggerWorkflowRequest) -> Result<Execution> {
        let result = PipedreamClient::trigger_workflow(self, id, data.data).await?;

        Ok(Execution {
            id: result.execution_id.unwrap_or_default(),
            workflow_id: id.to_string(),
            status: if result.success {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failed
            },
            started_at: Some(Utc::now()),
            finished_at: Some(Utc::now()),
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
            Error::InvalidRequest("workflow_id is required for Pipedream".to_string())
        })?;

        let events = self.list_workflow_events(&workflow_id).await?;
        let total = events.len() as u32;

        let mut executions: Vec<Execution> = events.into_iter().map(|e| e.into()).collect();
        for exec in &mut executions {
            exec.workflow_id = workflow_id.clone();
        }

        Ok(ListResult {
            data: executions,
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Pipedream does not support fetching individual executions".to_string(),
        ))
    }

    async fn cancel_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Pipedream does not support cancelling executions".to_string(),
        ))
    }

    async fn retry_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Pipedream does not support retrying executions".to_string(),
        ))
    }

    async fn list_connections(&self, _options: ListOptions) -> Result<ListResult<Connection>> {
        let accounts = self.list_connected_accounts().await?;
        let total = accounts.len() as u32;

        Ok(ListResult {
            data: accounts.into_iter().map(|a| a.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_connection(&self, id: &str) -> Result<Connection> {
        let accounts = self.list_connected_accounts().await?;
        accounts
            .into_iter()
            .find(|a| a.id == id)
            .map(|a| a.into())
            .ok_or_else(|| Error::NotFound(format!("Connection {}", id)))
    }

    async fn delete_connection(&self, id: &str) -> Result<()> {
        self.delete(&format!("/accounts/{}", id)).await
    }

    async fn list_apps(&self) -> Result<Vec<App>> {
        let apps = PipedreamClient::list_apps(self).await?;
        Ok(apps.into_iter().map(|a| a.into()).collect())
    }

    async fn get_app(&self, id: &str) -> Result<App> {
        let apps = PipedreamClient::list_apps(self).await?;
        apps.into_iter()
            .find(|a| a.id == id)
            .map(|a| a.into())
            .ok_or_else(|| Error::NotFound(format!("App {}", id)))
    }
}
