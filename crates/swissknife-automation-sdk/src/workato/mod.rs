use crate::{
    Action, App, AppAction, AppTrigger, AutomationProvider, Connection, ConnectionStatus,
    CreateWorkflow, Error, Execution, ExecutionError, ExecutionFilter, ExecutionStatus,
    Folder, ListOptions, ListResult, Result, Trigger, TriggerType, TriggerWorkflowRequest,
    UpdateWorkflow, Workflow, WorkflowStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://www.workato.com/api";

pub struct WorkatoClient {
    api_token: String,
    http: reqwest::Client,
}

impl WorkatoClient {
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(path.to_string()));
        }
        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API token".to_string()));
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
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API token".to_string()));
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
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API token".to_string()));
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
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if resp.status() == 401 {
            return Err(Error::Auth("Invalid API token".to_string()));
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

    pub async fn list_recipes(&self) -> Result<Vec<WorkatoRecipe>> {
        #[derive(Deserialize)]
        struct RecipesResponse {
            result: Vec<WorkatoRecipe>,
        }

        let resp: RecipesResponse = self.get("/recipes").await?;
        Ok(resp.result)
    }

    pub async fn get_recipe(&self, id: u64) -> Result<WorkatoRecipe> {
        #[derive(Deserialize)]
        struct RecipeResponse {
            result: WorkatoRecipe,
        }

        let resp: RecipeResponse = self.get(&format!("/recipes/{}", id)).await?;
        Ok(resp.result)
    }

    pub async fn start_recipe(&self, id: u64) -> Result<WorkatoRecipe> {
        #[derive(Deserialize)]
        struct StartResponse {
            result: WorkatoRecipe,
        }

        let resp: StartResponse = self.put(&format!("/recipes/{}/start", id), &serde_json::json!({})).await?;
        Ok(resp.result)
    }

    pub async fn stop_recipe(&self, id: u64) -> Result<WorkatoRecipe> {
        #[derive(Deserialize)]
        struct StopResponse {
            result: WorkatoRecipe,
        }

        let resp: StopResponse = self.put(&format!("/recipes/{}/stop", id), &serde_json::json!({})).await?;
        Ok(resp.result)
    }

    pub async fn list_jobs(&self, recipe_id: u64) -> Result<Vec<WorkatoJob>> {
        #[derive(Deserialize)]
        struct JobsResponse {
            result: Vec<WorkatoJob>,
        }

        let resp: JobsResponse = self.get(&format!("/recipes/{}/jobs", recipe_id)).await?;
        Ok(resp.result)
    }

    pub async fn get_job(&self, recipe_id: u64, job_id: u64) -> Result<WorkatoJob> {
        #[derive(Deserialize)]
        struct JobResponse {
            result: WorkatoJob,
        }

        let resp: JobResponse = self.get(&format!("/recipes/{}/jobs/{}", recipe_id, job_id)).await?;
        Ok(resp.result)
    }

    pub async fn list_connections(&self) -> Result<Vec<WorkatoConnection>> {
        #[derive(Deserialize)]
        struct ConnectionsResponse {
            result: Vec<WorkatoConnection>,
        }

        let resp: ConnectionsResponse = self.get("/connections").await?;
        Ok(resp.result)
    }

    pub async fn list_folders(&self) -> Result<Vec<WorkatoFolder>> {
        #[derive(Deserialize)]
        struct FoldersResponse {
            result: Vec<WorkatoFolder>,
        }

        let resp: FoldersResponse = self.get("/folders").await?;
        Ok(resp.result)
    }

    pub async fn list_connectors(&self) -> Result<Vec<WorkatoConnector>> {
        #[derive(Deserialize)]
        struct ConnectorsResponse {
            result: Vec<WorkatoConnector>,
        }

        let resp: ConnectorsResponse = self.get("/connectors").await?;
        Ok(resp.result)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkatoRecipe {
    pub id: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub running: bool,
    pub folder_id: Option<u64>,
    pub trigger_application: Option<String>,
    pub action_applications: Option<Vec<String>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: Option<i32>,
    pub code: Option<serde_json::Value>,
}

impl From<WorkatoRecipe> for Workflow {
    fn from(r: WorkatoRecipe) -> Self {
        let trigger = r.trigger_application.as_ref().map(|app| Trigger {
            id: r.id.to_string(),
            name: "Trigger".to_string(),
            trigger_type: TriggerType::AppEvent,
            app: Some(app.clone()),
            event: None,
            config: HashMap::new(),
        });

        let actions: Vec<Action> = r.action_applications
            .as_ref()
            .map(|apps| {
                apps.iter()
                    .enumerate()
                    .map(|(i, app)| Action {
                        id: format!("{}-{}", r.id, i),
                        name: app.clone(),
                        app: Some(app.clone()),
                        action_type: None,
                        position: Some(i as i32),
                        config: HashMap::new(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Workflow {
            id: r.id.to_string(),
            name: r.name.unwrap_or_default(),
            description: r.description,
            status: if r.running {
                WorkflowStatus::Active
            } else {
                WorkflowStatus::Inactive
            },
            trigger,
            actions,
            folder_id: r.folder_id.map(|id| id.to_string()),
            created_at: r
                .created_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updated_at: r
                .updated_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkatoJob {
    pub id: u64,
    pub recipe_id: Option<u64>,
    pub flow_run: Option<bool>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub is_poll_error: Option<bool>,
    pub error: Option<String>,
    pub error_parts: Option<serde_json::Value>,
    pub lines: Option<Vec<WorkatoJobLine>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkatoJobLine {
    pub recipe_line_number: Option<i32>,
    pub adapter_name: Option<String>,
    pub adapter_operation: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl From<WorkatoJob> for Execution {
    fn from(j: WorkatoJob) -> Self {
        let status = j.status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "succeeded" | "success" => ExecutionStatus::Success,
            "failed" | "error" => ExecutionStatus::Failed,
            "pending" | "processing" => ExecutionStatus::Running,
            "canceled" | "cancelled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Other,
        }).unwrap_or(ExecutionStatus::Other);

        let started_at = j
            .started_at
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let finished_at = j
            .completed_at
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let duration_ms = match (started_at, finished_at) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds() as u64),
            _ => None,
        };

        Execution {
            id: j.id.to_string(),
            workflow_id: j.recipe_id.map(|id| id.to_string()).unwrap_or_default(),
            status,
            started_at,
            finished_at,
            duration_ms,
            trigger_data: None,
            result: None,
            error: j.error.map(|e| ExecutionError {
                code: None,
                message: e,
                step_id: None,
                details: j.error_parts,
            }),
            steps: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkatoConnection {
    pub id: u64,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub authorization_status: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub external_id: Option<String>,
}

impl From<WorkatoConnection> for Connection {
    fn from(c: WorkatoConnection) -> Self {
        let status = c.authorization_status.as_deref().map(|s| match s.to_lowercase().as_str() {
            "success" | "authorized" => ConnectionStatus::Connected,
            "failed" | "error" => ConnectionStatus::Error,
            "expired" => ConnectionStatus::Expired,
            "pending" => ConnectionStatus::Pending,
            _ => ConnectionStatus::Other,
        }).unwrap_or(ConnectionStatus::Connected);

        Connection {
            id: c.id.to_string(),
            name: c.name.unwrap_or_default(),
            app: c.provider.unwrap_or_default(),
            status,
            account_name: c.external_id,
            created_at: c
                .created_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            expires_at: None,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkatoFolder {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
}

impl From<WorkatoFolder> for Folder {
    fn from(f: WorkatoFolder) -> Self {
        Folder {
            id: f.id.to_string(),
            name: f.name,
            parent_id: f.parent_id.map(|id| id.to_string()),
            workflow_count: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkatoConnector {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub triggers: Option<Vec<WorkatoOperation>>,
    pub actions: Option<Vec<WorkatoOperation>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkatoOperation {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl From<WorkatoConnector> for App {
    fn from(c: WorkatoConnector) -> Self {
        App {
            id: c.name.clone(),
            name: c.title.unwrap_or(c.name),
            slug: None,
            description: c.description,
            icon_url: c.icon_url,
            categories: Vec::new(),
            triggers: c.triggers.unwrap_or_default()
                .into_iter()
                .map(|t| AppTrigger {
                    id: t.name,
                    name: t.title.unwrap_or_default(),
                    description: t.description,
                    trigger_type: TriggerType::AppEvent,
                })
                .collect(),
            actions: c.actions.unwrap_or_default()
                .into_iter()
                .map(|a| AppAction {
                    id: a.name,
                    name: a.title.unwrap_or_default(),
                    description: a.description,
                })
                .collect(),
        }
    }
}

#[async_trait]
impl AutomationProvider for WorkatoClient {
    async fn list_workflows(&self, _options: ListOptions) -> Result<ListResult<Workflow>> {
        let recipes = self.list_recipes().await?;
        let total = recipes.len() as u32;

        Ok(ListResult {
            data: recipes.into_iter().map(|r| r.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_workflow(&self, id: &str) -> Result<Workflow> {
        let recipe_id: u64 = id
            .parse()
            .map_err(|_| Error::InvalidRequest("Invalid recipe ID".to_string()))?;
        let recipe = self.get_recipe(recipe_id).await?;
        Ok(recipe.into())
    }

    async fn create_workflow(&self, data: CreateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        struct CreateRequest {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            folder_id: Option<u64>,
        }

        let folder_id = data.folder_id.and_then(|id| id.parse().ok());

        let body = CreateRequest {
            name: data.name,
            description: data.description,
            folder_id,
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            result: WorkatoRecipe,
        }

        let resp: CreateResponse = self.post("/recipes", &body).await?;
        Ok(resp.result.into())
    }

    async fn update_workflow(&self, id: &str, data: UpdateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        struct UpdateRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            folder_id: Option<u64>,
        }

        let folder_id = data.folder_id.and_then(|id| id.parse().ok());

        let body = UpdateRequest {
            name: data.name,
            description: data.description,
            folder_id,
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            result: WorkatoRecipe,
        }

        let resp: UpdateResponse = self.put(&format!("/recipes/{}", id), &body).await?;
        Ok(resp.result.into())
    }

    async fn delete_workflow(&self, id: &str) -> Result<()> {
        self.delete(&format!("/recipes/{}", id)).await
    }

    async fn activate_workflow(&self, id: &str) -> Result<Workflow> {
        let recipe_id: u64 = id
            .parse()
            .map_err(|_| Error::InvalidRequest("Invalid recipe ID".to_string()))?;
        let recipe = self.start_recipe(recipe_id).await?;
        Ok(recipe.into())
    }

    async fn deactivate_workflow(&self, id: &str) -> Result<Workflow> {
        let recipe_id: u64 = id
            .parse()
            .map_err(|_| Error::InvalidRequest("Invalid recipe ID".to_string()))?;
        let recipe = self.stop_recipe(recipe_id).await?;
        Ok(recipe.into())
    }

    async fn trigger_workflow(&self, _id: &str, _data: TriggerWorkflowRequest) -> Result<Execution> {
        Err(Error::Provider(
            "Workato recipes must be triggered via their configured triggers".to_string(),
        ))
    }

    async fn list_executions(
        &self,
        filter: ExecutionFilter,
        _options: ListOptions,
    ) -> Result<ListResult<Execution>> {
        let workflow_id = filter.workflow_id.ok_or_else(|| {
            Error::InvalidRequest("workflow_id is required for Workato".to_string())
        })?;

        let recipe_id: u64 = workflow_id
            .parse()
            .map_err(|_| Error::InvalidRequest("Invalid recipe ID".to_string()))?;

        let jobs = self.list_jobs(recipe_id).await?;
        let total = jobs.len() as u32;

        Ok(ListResult {
            data: jobs.into_iter().map(|j| j.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_execution(&self, id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Workato requires recipe_id to fetch job details".to_string(),
        ))
    }

    async fn cancel_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Workato does not support cancelling jobs via API".to_string(),
        ))
    }

    async fn retry_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Workato does not support retrying jobs via API".to_string(),
        ))
    }

    async fn list_connections(&self, _options: ListOptions) -> Result<ListResult<Connection>> {
        let connections = WorkatoClient::list_connections(self).await?;
        let total = connections.len() as u32;

        Ok(ListResult {
            data: connections.into_iter().map(|c| c.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_connection(&self, id: &str) -> Result<Connection> {
        let connections = WorkatoClient::list_connections(self).await?;
        connections
            .into_iter()
            .find(|c| c.id.to_string() == id)
            .map(|c| c.into())
            .ok_or_else(|| Error::NotFound(format!("Connection {}", id)))
    }

    async fn delete_connection(&self, id: &str) -> Result<()> {
        self.delete(&format!("/connections/{}", id)).await
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
