use crate::{
    Action, App, AutomationProvider, Connection, ConnectionStatus, CreateWorkflow, Error,
    Execution, ExecutionError, ExecutionFilter, ExecutionStatus, ExecutionStep, ListOptions,
    ListResult, Result, Trigger, TriggerType, TriggerWorkflowRequest, UpdateWorkflow, Workflow,
    WorkflowStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct N8nClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl N8nClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            base_url: url,
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}/api/v1{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("X-N8N-API-KEY", &self.api_key)
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
        let url = format!("{}/api/v1{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("X-N8N-API-KEY", &self.api_key)
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

    async fn patch<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}/api/v1{}", self.base_url, path);
        let resp = self
            .http
            .patch(&url)
            .header("X-N8N-API-KEY", &self.api_key)
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
        let url = format!("{}/api/v1{}", self.base_url, path);
        let resp = self
            .http
            .delete(&url)
            .header("X-N8N-API-KEY", &self.api_key)
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

    pub async fn list_workflows(&self) -> Result<Vec<N8nWorkflow>> {
        #[derive(Deserialize)]
        struct WorkflowsResponse {
            data: Vec<N8nWorkflow>,
        }

        let resp: WorkflowsResponse = self.get("/workflows").await?;
        Ok(resp.data)
    }

    pub async fn get_workflow(&self, id: &str) -> Result<N8nWorkflow> {
        self.get(&format!("/workflows/{}", id)).await
    }

    pub async fn list_executions(&self, workflow_id: Option<&str>) -> Result<Vec<N8nExecution>> {
        #[derive(Deserialize)]
        struct ExecutionsResponse {
            data: Vec<N8nExecution>,
        }

        let path = match workflow_id {
            Some(id) => format!("/executions?workflowId={}", id),
            None => "/executions".to_string(),
        };

        let resp: ExecutionsResponse = self.get(&path).await?;
        Ok(resp.data)
    }

    pub async fn get_execution(&self, id: &str) -> Result<N8nExecution> {
        self.get(&format!("/executions/{}", id)).await
    }

    pub async fn execute_workflow(&self, id: &str, data: Option<serde_json::Value>) -> Result<N8nExecution> {
        #[derive(Serialize)]
        struct ExecuteRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            data: Option<serde_json::Value>,
        }

        let body = ExecuteRequest { data };
        self.post(&format!("/workflows/{}/execute", id), &body).await
    }

    pub async fn list_credentials(&self) -> Result<Vec<N8nCredential>> {
        #[derive(Deserialize)]
        struct CredentialsResponse {
            data: Vec<N8nCredential>,
        }

        let resp: CredentialsResponse = self.get("/credentials").await?;
        Ok(resp.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N8nWorkflow {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub nodes: Option<Vec<N8nNode>>,
    pub connections: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub tags: Option<Vec<N8nTag>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N8nNode {
    pub id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub position: Option<Vec<i32>>,
    pub parameters: Option<serde_json::Value>,
    pub credentials: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct N8nTag {
    pub id: String,
    pub name: String,
}

impl From<N8nWorkflow> for Workflow {
    fn from(w: N8nWorkflow) -> Self {
        let nodes = w.nodes.unwrap_or_default();

        let trigger_node = nodes.iter().find(|n| {
            n.node_type.contains("Trigger")
                || n.node_type.contains("Webhook")
                || n.node_type.contains("Schedule")
        });

        let trigger = trigger_node.map(|n| {
            let trigger_type = if n.node_type.contains("Webhook") {
                TriggerType::Webhook
            } else if n.node_type.contains("Schedule") || n.node_type.contains("Cron") {
                TriggerType::Schedule
            } else {
                TriggerType::AppEvent
            };

            Trigger {
                id: n.id.clone().unwrap_or_default(),
                name: n.name.clone(),
                trigger_type,
                app: Some(n.node_type.clone()),
                event: None,
                config: n
                    .parameters
                    .as_ref()
                    .and_then(|p| p.as_object())
                    .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            }
        });

        let actions: Vec<Action> = nodes
            .into_iter()
            .filter(|n| !n.node_type.contains("Trigger") && !n.node_type.contains("Start"))
            .enumerate()
            .map(|(i, n)| Action {
                id: n.id.unwrap_or_else(|| i.to_string()),
                name: n.name,
                app: Some(n.node_type),
                action_type: None,
                position: Some(i as i32),
                config: n
                    .parameters
                    .and_then(|p| p.as_object().cloned())
                    .map(|o| o.into_iter().collect())
                    .unwrap_or_default(),
            })
            .collect();

        Workflow {
            id: w.id,
            name: w.name,
            description: None,
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
#[serde(rename_all = "camelCase")]
pub struct N8nExecution {
    pub id: String,
    pub workflow_id: Option<String>,
    pub finished: bool,
    pub mode: Option<String>,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub status: Option<String>,
    pub data: Option<N8nExecutionData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N8nExecutionData {
    pub result_data: Option<N8nResultData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N8nResultData {
    pub run_data: Option<HashMap<String, Vec<N8nNodeExecution>>>,
    pub error: Option<N8nError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct N8nNodeExecution {
    pub data: Option<serde_json::Value>,
    pub error: Option<N8nError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct N8nError {
    pub message: Option<String>,
    pub node: Option<String>,
}

impl From<N8nExecution> for Execution {
    fn from(e: N8nExecution) -> Self {
        let status = match e.status.as_deref() {
            Some("success") => ExecutionStatus::Success,
            Some("error") | Some("failed") => ExecutionStatus::Failed,
            Some("running") | Some("waiting") => ExecutionStatus::Running,
            Some("canceled") => ExecutionStatus::Cancelled,
            _ if e.finished => ExecutionStatus::Success,
            _ => ExecutionStatus::Running,
        };

        let error = e.data.as_ref().and_then(|d| {
            d.result_data.as_ref().and_then(|r| {
                r.error.as_ref().map(|err| ExecutionError {
                    code: None,
                    message: err.message.clone().unwrap_or_default(),
                    step_id: err.node.clone(),
                    details: None,
                })
            })
        });

        let steps: Vec<ExecutionStep> = e
            .data
            .as_ref()
            .and_then(|d| d.result_data.as_ref())
            .and_then(|r| r.run_data.as_ref())
            .map(|run_data| {
                run_data
                    .iter()
                    .map(|(name, execs)| {
                        let exec = execs.first();
                        ExecutionStep {
                            id: name.clone(),
                            action_id: None,
                            name: name.clone(),
                            status: if exec.and_then(|e| e.error.as_ref()).is_some() {
                                ExecutionStatus::Failed
                            } else {
                                ExecutionStatus::Success
                            },
                            started_at: None,
                            finished_at: None,
                            input: None,
                            output: exec.and_then(|e| e.data.clone()),
                            error: exec.and_then(|e| e.error.as_ref().and_then(|err| err.message.clone())),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let started_at = e
            .started_at
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let finished_at = e
            .stopped_at
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let duration_ms = match (started_at, finished_at) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds() as u64),
            _ => None,
        };

        Execution {
            id: e.id,
            workflow_id: e.workflow_id.unwrap_or_default(),
            status,
            started_at,
            finished_at,
            duration_ms,
            trigger_data: None,
            result: None,
            error,
            steps,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N8nCredential {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub credential_type: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<N8nCredential> for Connection {
    fn from(c: N8nCredential) -> Self {
        Connection {
            id: c.id,
            name: c.name,
            app: c.credential_type,
            status: ConnectionStatus::Connected,
            account_name: None,
            created_at: c
                .created_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            expires_at: None,
            extra: HashMap::new(),
        }
    }
}

#[async_trait]
impl AutomationProvider for N8nClient {
    async fn list_workflows(&self, _options: ListOptions) -> Result<ListResult<Workflow>> {
        let workflows = self.list_workflows().await?;
        let total = workflows.len() as u32;

        Ok(ListResult {
            data: workflows.into_iter().map(|w| w.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_workflow(&self, id: &str) -> Result<Workflow> {
        let workflow = N8nClient::get_workflow(self, id).await?;
        Ok(workflow.into())
    }

    async fn create_workflow(&self, data: CreateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        struct CreateRequest {
            name: String,
            nodes: Vec<serde_json::Value>,
            connections: serde_json::Value,
            settings: serde_json::Value,
        }

        let body = CreateRequest {
            name: data.name,
            nodes: vec![serde_json::json!({
                "name": "Start",
                "type": "n8n-nodes-base.start",
                "position": [250, 300],
                "parameters": {}
            })],
            connections: serde_json::json!({}),
            settings: serde_json::json!({}),
        };

        let workflow: N8nWorkflow = self.post("/workflows", &body).await?;
        Ok(workflow.into())
    }

    async fn update_workflow(&self, id: &str, data: UpdateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        struct UpdateRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            active: Option<bool>,
        }

        let active = data.status.map(|s| s == WorkflowStatus::Active);

        let body = UpdateRequest {
            name: data.name,
            active,
        };

        let workflow: N8nWorkflow = self.patch(&format!("/workflows/{}", id), &body).await?;
        Ok(workflow.into())
    }

    async fn delete_workflow(&self, id: &str) -> Result<()> {
        self.delete(&format!("/workflows/{}", id)).await
    }

    async fn activate_workflow(&self, id: &str) -> Result<Workflow> {
        let body = serde_json::json!({ "active": true });
        let workflow: N8nWorkflow = self.patch(&format!("/workflows/{}", id), &body).await?;
        Ok(workflow.into())
    }

    async fn deactivate_workflow(&self, id: &str) -> Result<Workflow> {
        let body = serde_json::json!({ "active": false });
        let workflow: N8nWorkflow = self.patch(&format!("/workflows/{}", id), &body).await?;
        Ok(workflow.into())
    }

    async fn trigger_workflow(&self, id: &str, data: TriggerWorkflowRequest) -> Result<Execution> {
        let execution = self.execute_workflow(id, Some(data.data)).await?;
        Ok(execution.into())
    }

    async fn list_executions(
        &self,
        filter: ExecutionFilter,
        _options: ListOptions,
    ) -> Result<ListResult<Execution>> {
        let executions = N8nClient::list_executions(self, filter.workflow_id.as_deref()).await?;
        let total = executions.len() as u32;

        Ok(ListResult {
            data: executions.into_iter().map(|e| e.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_execution(&self, id: &str) -> Result<Execution> {
        let execution = N8nClient::get_execution(self, id).await?;
        Ok(execution.into())
    }

    async fn cancel_execution(&self, id: &str) -> Result<Execution> {
        self.post(&format!("/executions/{}/stop", id), &serde_json::json!({}))
            .await
    }

    async fn retry_execution(&self, id: &str) -> Result<Execution> {
        let execution: N8nExecution = self
            .post(&format!("/executions/{}/retry", id), &serde_json::json!({}))
            .await?;
        Ok(execution.into())
    }

    async fn list_connections(&self, _options: ListOptions) -> Result<ListResult<Connection>> {
        let credentials = self.list_credentials().await?;
        let total = credentials.len() as u32;

        Ok(ListResult {
            data: credentials.into_iter().map(|c| c.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_connection(&self, id: &str) -> Result<Connection> {
        let credentials = self.list_credentials().await?;
        credentials
            .into_iter()
            .find(|c| c.id == id)
            .map(|c| c.into())
            .ok_or_else(|| Error::NotFound(format!("Connection {}", id)))
    }

    async fn delete_connection(&self, id: &str) -> Result<()> {
        self.delete(&format!("/credentials/{}", id)).await
    }

    async fn list_apps(&self) -> Result<Vec<App>> {
        Err(Error::Provider(
            "n8n does not provide an apps API".to_string(),
        ))
    }

    async fn get_app(&self, _id: &str) -> Result<App> {
        Err(Error::Provider(
            "n8n does not provide an apps API".to_string(),
        ))
    }
}
