use crate::{
    Action, App, AppAction, AppTrigger, AutomationProvider, Connection, ConnectionStatus,
    CreateWorkflow, Error, Execution, ExecutionError, ExecutionFilter, ExecutionStatus,
    ExecutionStep, Folder, ListOptions, ListResult, Result, Trigger, TriggerType,
    TriggerWorkflowRequest, UpdateWorkflow, Workflow, WorkflowStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://us1.make.com/api/v2";

pub struct MakeClient {
    api_token: String,
    team_id: String,
    zone: String,
    http: reqwest::Client,
}

impl MakeClient {
    pub fn new(api_token: impl Into<String>, team_id: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            team_id: team_id.into(),
            zone: "us1".to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_zone(mut self, zone: impl Into<String>) -> Self {
        self.zone = zone.into();
        self
    }

    fn base_url(&self) -> String {
        format!("https://{}.make.com/api/v2", self.zone)
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url(), path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_token))
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
        let url = format!("{}{}", self.base_url(), path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Token {}", self.api_token))
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

    async fn patch<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url(), path);
        let resp = self
            .http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.api_token))
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
        let url = format!("{}{}", self.base_url(), path);
        let resp = self
            .http
            .delete(&url)
            .header("Authorization", format!("Token {}", self.api_token))
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

    pub async fn list_scenarios(&self) -> Result<Vec<MakeScenario>> {
        #[derive(Deserialize)]
        struct ScenariosResponse {
            scenarios: Vec<MakeScenario>,
        }

        let resp: ScenariosResponse = self
            .get(&format!("/teams/{}/scenarios", self.team_id))
            .await?;
        Ok(resp.scenarios)
    }

    pub async fn get_scenario(&self, scenario_id: u64) -> Result<MakeScenario> {
        #[derive(Deserialize)]
        struct ScenarioResponse {
            scenario: MakeScenario,
        }

        let resp: ScenarioResponse = self.get(&format!("/scenarios/{}", scenario_id)).await?;
        Ok(resp.scenario)
    }

    pub async fn list_scenario_logs(&self, scenario_id: u64) -> Result<Vec<MakeExecution>> {
        #[derive(Deserialize)]
        struct LogsResponse {
            logs: Vec<MakeExecution>,
        }

        let resp: LogsResponse = self
            .get(&format!("/scenarios/{}/logs", scenario_id))
            .await?;
        Ok(resp.logs)
    }

    pub async fn run_scenario(&self, scenario_id: u64, data: Option<serde_json::Value>) -> Result<MakeRunResult> {
        #[derive(Serialize)]
        struct RunRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            data: Option<serde_json::Value>,
        }

        let body = RunRequest { data };

        self.post(&format!("/scenarios/{}/run", scenario_id), &body)
            .await
    }

    pub async fn list_connections(&self) -> Result<Vec<MakeConnection>> {
        #[derive(Deserialize)]
        struct ConnectionsResponse {
            connections: Vec<MakeConnection>,
        }

        let resp: ConnectionsResponse = self
            .get(&format!("/teams/{}/connections", self.team_id))
            .await?;
        Ok(resp.connections)
    }

    pub async fn list_folders(&self) -> Result<Vec<MakeFolder>> {
        #[derive(Deserialize)]
        struct FoldersResponse {
            folders: Vec<MakeFolder>,
        }

        let resp: FoldersResponse = self
            .get(&format!("/teams/{}/folders", self.team_id))
            .await?;
        Ok(resp.folders)
    }

    pub async fn list_apps(&self) -> Result<Vec<MakeApp>> {
        #[derive(Deserialize)]
        struct AppsResponse {
            apps: Vec<MakeApp>,
        }

        let resp: AppsResponse = self.get("/apps").await?;
        Ok(resp.apps)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeScenario {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: bool,
    pub is_paused: bool,
    pub scheduling: Option<MakeScheduling>,
    pub folder_id: Option<u64>,
    pub created: Option<String>,
    pub last_edit: Option<String>,
    pub blueprint: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeScheduling {
    pub scheduling_type: Option<String>,
    pub interval: Option<u64>,
}

impl From<MakeScenario> for Workflow {
    fn from(s: MakeScenario) -> Self {
        let status = if !s.is_enabled {
            WorkflowStatus::Inactive
        } else if s.is_paused {
            WorkflowStatus::Paused
        } else {
            WorkflowStatus::Active
        };

        let trigger_type = s
            .scheduling
            .as_ref()
            .and_then(|sc| sc.scheduling_type.as_ref())
            .map(|t| match t.as_str() {
                "immediately" => TriggerType::Webhook,
                "indefinitely" | "interval" => TriggerType::Schedule,
                _ => TriggerType::Other,
            })
            .unwrap_or(TriggerType::Other);

        let trigger = Some(Trigger {
            id: s.id.to_string(),
            name: "Trigger".to_string(),
            trigger_type,
            app: None,
            event: None,
            config: HashMap::new(),
        });

        Workflow {
            id: s.id.to_string(),
            name: s.name,
            description: s.description,
            status,
            trigger,
            actions: Vec::new(),
            folder_id: s.folder_id.map(|id| id.to_string()),
            created_at: s
                .created
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updated_at: s
                .last_edit
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeExecution {
    pub id: u64,
    pub scenario_id: u64,
    pub status: String,
    pub timestamp: Option<String>,
    pub duration: Option<u64>,
    pub operations: Option<u64>,
    pub transfer: Option<u64>,
    pub error: Option<MakeError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MakeError {
    pub message: Option<String>,
    pub module: Option<String>,
}

impl From<MakeExecution> for Execution {
    fn from(e: MakeExecution) -> Self {
        let status = match e.status.to_lowercase().as_str() {
            "success" => ExecutionStatus::Success,
            "error" | "failed" => ExecutionStatus::Failed,
            "running" | "processing" => ExecutionStatus::Running,
            "waiting" => ExecutionStatus::Waiting,
            _ => ExecutionStatus::Other,
        };

        Execution {
            id: e.id.to_string(),
            workflow_id: e.scenario_id.to_string(),
            status,
            started_at: e
                .timestamp
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            finished_at: None,
            duration_ms: e.duration,
            trigger_data: None,
            result: None,
            error: e.error.map(|err| ExecutionError {
                code: None,
                message: err.message.unwrap_or_default(),
                step_id: err.module,
                details: None,
            }),
            steps: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeRunResult {
    pub execution_id: Option<u64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeConnection {
    pub id: u64,
    pub name: String,
    pub account_name: Option<String>,
    pub account_label: Option<String>,
    pub package_name: Option<String>,
    pub expire: Option<String>,
    pub editable: Option<bool>,
    pub uid: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl From<MakeConnection> for Connection {
    fn from(c: MakeConnection) -> Self {
        let status = c
            .expire
            .as_ref()
            .and_then(|e| DateTime::parse_from_rfc3339(e).ok())
            .map(|exp| {
                if exp.with_timezone(&Utc) < Utc::now() {
                    ConnectionStatus::Expired
                } else {
                    ConnectionStatus::Connected
                }
            })
            .unwrap_or(ConnectionStatus::Connected);

        Connection {
            id: c.id.to_string(),
            name: c.name,
            app: c.package_name.unwrap_or_default(),
            status,
            account_name: c.account_name.or(c.account_label),
            created_at: None,
            expires_at: c
                .expire
                .and_then(|e| DateTime::parse_from_rfc3339(&e).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MakeFolder {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
}

impl From<MakeFolder> for Folder {
    fn from(f: MakeFolder) -> Self {
        Folder {
            id: f.id.to_string(),
            name: f.name,
            parent_id: f.parent_id.map(|id| id.to_string()),
            workflow_count: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MakeApp {
    pub name: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub icon_url: Option<String>,
    pub categories: Option<Vec<String>>,
}

impl From<MakeApp> for App {
    fn from(a: MakeApp) -> Self {
        App {
            id: a.name.clone(),
            name: a.label.unwrap_or(a.name),
            slug: None,
            description: a.description,
            icon_url: a.icon_url,
            categories: a.categories.unwrap_or_default(),
            triggers: Vec::new(),
            actions: Vec::new(),
        }
    }
}

#[async_trait]
impl AutomationProvider for MakeClient {
    async fn list_workflows(&self, _options: ListOptions) -> Result<ListResult<Workflow>> {
        let scenarios = self.list_scenarios().await?;
        let total = scenarios.len() as u32;

        Ok(ListResult {
            data: scenarios.into_iter().map(|s| s.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_workflow(&self, id: &str) -> Result<Workflow> {
        let scenario_id: u64 = id
            .parse()
            .map_err(|_| Error::InvalidRequest("Invalid scenario ID".to_string()))?;
        let scenario = self.get_scenario(scenario_id).await?;
        Ok(scenario.into())
    }

    async fn create_workflow(&self, data: CreateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateRequest {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            folder_id: Option<u64>,
            team_id: String,
        }

        let folder_id = data.folder_id.and_then(|id| id.parse().ok());

        let body = CreateRequest {
            name: data.name,
            description: data.description,
            folder_id,
            team_id: self.team_id.clone(),
        };

        #[derive(Deserialize)]
        struct CreateResponse {
            scenario: MakeScenario,
        }

        let resp: CreateResponse = self.post("/scenarios", &body).await?;
        Ok(resp.scenario.into())
    }

    async fn update_workflow(&self, id: &str, data: UpdateWorkflow) -> Result<Workflow> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            folder_id: Option<u64>,
        }

        let body = UpdateRequest {
            name: data.name,
            description: data.description,
            folder_id: data.folder_id.and_then(|id| id.parse().ok()),
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            scenario: MakeScenario,
        }

        let resp: UpdateResponse = self.patch(&format!("/scenarios/{}", id), &body).await?;
        Ok(resp.scenario.into())
    }

    async fn delete_workflow(&self, id: &str) -> Result<()> {
        self.delete(&format!("/scenarios/{}", id)).await
    }

    async fn activate_workflow(&self, id: &str) -> Result<Workflow> {
        #[derive(Deserialize)]
        struct ActivateResponse {
            scenario: MakeScenario,
        }

        let resp: ActivateResponse = self
            .post(&format!("/scenarios/{}/start", id), &serde_json::json!({}))
            .await?;
        Ok(resp.scenario.into())
    }

    async fn deactivate_workflow(&self, id: &str) -> Result<Workflow> {
        #[derive(Deserialize)]
        struct DeactivateResponse {
            scenario: MakeScenario,
        }

        let resp: DeactivateResponse = self
            .post(&format!("/scenarios/{}/stop", id), &serde_json::json!({}))
            .await?;
        Ok(resp.scenario.into())
    }

    async fn trigger_workflow(&self, id: &str, data: TriggerWorkflowRequest) -> Result<Execution> {
        let scenario_id: u64 = id
            .parse()
            .map_err(|_| Error::InvalidRequest("Invalid scenario ID".to_string()))?;

        let result = self.run_scenario(scenario_id, Some(data.data)).await?;

        Ok(Execution {
            id: result.execution_id.map(|id| id.to_string()).unwrap_or_default(),
            workflow_id: id.to_string(),
            status: match result.status.as_deref() {
                Some("success") => ExecutionStatus::Success,
                Some("error") => ExecutionStatus::Failed,
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
            Error::InvalidRequest("workflow_id is required for Make".to_string())
        })?;

        let scenario_id: u64 = workflow_id
            .parse()
            .map_err(|_| Error::InvalidRequest("Invalid scenario ID".to_string()))?;

        let logs = self.list_scenario_logs(scenario_id).await?;
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
            "Make does not support fetching individual executions".to_string(),
        ))
    }

    async fn cancel_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Make does not support cancelling executions via API".to_string(),
        ))
    }

    async fn retry_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Make does not support retrying executions via API".to_string(),
        ))
    }

    async fn list_connections(&self, _options: ListOptions) -> Result<ListResult<Connection>> {
        let connections = self.list_connections().await?;
        let total = connections.len() as u32;

        Ok(ListResult {
            data: connections.into_iter().map(|c| c.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_connection(&self, id: &str) -> Result<Connection> {
        let connections = self.list_connections().await?;
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
        let apps = self.list_apps().await?;
        Ok(apps.into_iter().map(|a| a.into()).collect())
    }

    async fn get_app(&self, id: &str) -> Result<App> {
        let apps = self.list_apps().await?;
        apps.into_iter()
            .find(|a| a.name == id)
            .map(|a| a.into())
            .ok_or_else(|| Error::NotFound(format!("App {}", id)))
    }
}
