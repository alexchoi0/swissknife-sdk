use crate::{
    Action, App, AppAction, AppTrigger, AutomationProvider, Connection, ConnectionStatus,
    CreateWorkflow, Error, Execution, ExecutionError, ExecutionFilter, ExecutionStatus,
    ExecutionStep, ListOptions, ListResult, Result, Trigger, TriggerType, TriggerWorkflowRequest,
    UpdateWorkflow, Workflow, WorkflowStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "https://api.zapier.com/v1";
const NLA_API_BASE: &str = "https://nla.zapier.com/api/v1";

pub struct ZapierClient {
    api_key: String,
    http: reqwest::Client,
}

impl ZapierClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let resp = self
            .http
            .get(url)
            .header("X-API-Key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == 404 {
            return Err(Error::NotFound(url.to_string()));
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
        url: &str,
        body: &B,
    ) -> Result<T> {
        let resp = self
            .http
            .post(url)
            .header("X-API-Key", &self.api_key)
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

    async fn delete(&self, url: &str) -> Result<()> {
        let resp = self
            .http
            .delete(url)
            .header("X-API-Key", &self.api_key)
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

    pub async fn trigger_webhook(&self, webhook_url: &str, data: serde_json::Value) -> Result<WebhookResponse> {
        let resp = self
            .http
            .post(webhook_url)
            .json(&data)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: None,
            });
        }

        Ok(WebhookResponse {
            status: resp.status().as_u16(),
            request_id: resp
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
        })
    }

    pub async fn list_zaps(&self) -> Result<Vec<Zap>> {
        #[derive(Deserialize)]
        struct ZapsResponse {
            zaps: Vec<Zap>,
        }

        let resp: ZapsResponse = self.get(&format!("{}/zaps", API_BASE)).await?;
        Ok(resp.zaps)
    }

    pub async fn get_zap(&self, zap_id: &str) -> Result<Zap> {
        self.get(&format!("{}/zaps/{}", API_BASE, zap_id)).await
    }

    pub async fn list_nla_actions(&self) -> Result<Vec<NlaAction>> {
        #[derive(Deserialize)]
        struct NlaResponse {
            results: Vec<NlaAction>,
        }

        let resp: NlaResponse = self.get(&format!("{}/exposed/", NLA_API_BASE)).await?;
        Ok(resp.results)
    }

    pub async fn execute_nla_action(
        &self,
        action_id: &str,
        instructions: &str,
        params: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<NlaExecutionResult> {
        #[derive(Serialize)]
        struct NlaRequest {
            instructions: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            params: Option<HashMap<String, serde_json::Value>>,
        }

        let body = NlaRequest {
            instructions: instructions.to_string(),
            params,
        };

        self.post(
            &format!("{}/exposed/{}/execute/", NLA_API_BASE, action_id),
            &body,
        )
        .await
    }

    pub async fn get_zap_history(&self, zap_id: &str) -> Result<Vec<ZapRun>> {
        #[derive(Deserialize)]
        struct HistoryResponse {
            runs: Vec<ZapRun>,
        }

        let resp: HistoryResponse = self
            .get(&format!("{}/zaps/{}/history", API_BASE, zap_id))
            .await?;
        Ok(resp.runs)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Zap {
    pub id: String,
    pub title: Option<String>,
    pub is_enabled: bool,
    pub steps: Option<Vec<ZapStep>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZapStep {
    pub id: String,
    pub title: Option<String>,
    pub app: Option<String>,
    pub action: Option<String>,
    pub position: Option<i32>,
}

impl From<Zap> for Workflow {
    fn from(z: Zap) -> Self {
        let steps = z.steps.unwrap_or_default();
        let trigger = steps.first().map(|s| Trigger {
            id: s.id.clone(),
            name: s.title.clone().unwrap_or_default(),
            trigger_type: TriggerType::AppEvent,
            app: s.app.clone(),
            event: s.action.clone(),
            config: HashMap::new(),
        });

        let actions: Vec<Action> = steps
            .into_iter()
            .skip(1)
            .map(|s| Action {
                id: s.id,
                name: s.title.unwrap_or_default(),
                app: s.app,
                action_type: s.action,
                position: s.position,
                config: HashMap::new(),
            })
            .collect();

        Workflow {
            id: z.id,
            name: z.title.unwrap_or_default(),
            description: None,
            status: if z.is_enabled {
                WorkflowStatus::Active
            } else {
                WorkflowStatus::Inactive
            },
            trigger,
            actions,
            folder_id: None,
            created_at: z
                .created_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updated_at: z
                .updated_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZapRun {
    pub id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub data_in: Option<serde_json::Value>,
    pub data_out: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl From<ZapRun> for Execution {
    fn from(r: ZapRun) -> Self {
        let status = match r.status.to_lowercase().as_str() {
            "success" | "succeeded" => ExecutionStatus::Success,
            "failed" | "error" => ExecutionStatus::Failed,
            "running" | "pending" => ExecutionStatus::Running,
            "cancelled" | "stopped" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Other,
        };

        Execution {
            id: r.id,
            workflow_id: String::new(),
            status,
            started_at: r
                .started_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            finished_at: r
                .finished_at
                .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            duration_ms: None,
            trigger_data: r.data_in,
            result: r.data_out,
            error: r.error.map(|e| ExecutionError {
                code: None,
                message: e,
                step_id: None,
                details: None,
            }),
            steps: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NlaAction {
    pub id: String,
    pub description: String,
    pub operation_id: Option<String>,
    pub params: Option<HashMap<String, NlaParam>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NlaParam {
    pub description: Option<String>,
    pub param_type: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NlaExecutionResult {
    pub id: Option<String>,
    pub action_used: Option<String>,
    pub input_params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebhookResponse {
    pub status: u16,
    pub request_id: Option<String>,
}

#[async_trait]
impl AutomationProvider for ZapierClient {
    async fn list_workflows(&self, _options: ListOptions) -> Result<ListResult<Workflow>> {
        let zaps = self.list_zaps().await?;
        let total = zaps.len() as u32;

        Ok(ListResult {
            data: zaps.into_iter().map(|z| z.into()).collect(),
            total: Some(total),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_workflow(&self, id: &str) -> Result<Workflow> {
        let zap = self.get_zap(id).await?;
        Ok(zap.into())
    }

    async fn create_workflow(&self, _data: CreateWorkflow) -> Result<Workflow> {
        Err(Error::Provider(
            "Zapier workflows must be created through the Zapier UI".to_string(),
        ))
    }

    async fn update_workflow(&self, _id: &str, _data: UpdateWorkflow) -> Result<Workflow> {
        Err(Error::Provider(
            "Zapier workflows must be updated through the Zapier UI".to_string(),
        ))
    }

    async fn delete_workflow(&self, _id: &str) -> Result<()> {
        Err(Error::Provider(
            "Zapier workflows must be deleted through the Zapier UI".to_string(),
        ))
    }

    async fn activate_workflow(&self, _id: &str) -> Result<Workflow> {
        Err(Error::Provider(
            "Zapier workflow activation requires the Partner API".to_string(),
        ))
    }

    async fn deactivate_workflow(&self, _id: &str) -> Result<Workflow> {
        Err(Error::Provider(
            "Zapier workflow deactivation requires the Partner API".to_string(),
        ))
    }

    async fn trigger_workflow(&self, id: &str, data: TriggerWorkflowRequest) -> Result<Execution> {
        let nla_result = self
            .execute_nla_action(id, "Execute this action", Some(
                data.data.as_object().map(|o| {
                    o.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                }).unwrap_or_default()
            ))
            .await?;

        Ok(Execution {
            id: nla_result.id.unwrap_or_default(),
            workflow_id: id.to_string(),
            status: match nla_result.status.as_deref() {
                Some("success") => ExecutionStatus::Success,
                Some("error") => ExecutionStatus::Failed,
                _ => ExecutionStatus::Running,
            },
            started_at: Some(Utc::now()),
            finished_at: Some(Utc::now()),
            duration_ms: None,
            trigger_data: nla_result.input_params,
            result: nla_result.result,
            error: nla_result.error.map(|e| ExecutionError {
                code: None,
                message: e,
                step_id: None,
                details: None,
            }),
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
            Error::InvalidRequest("workflow_id is required for Zapier".to_string())
        })?;

        let runs = self.get_zap_history(&workflow_id).await?;
        let total = runs.len() as u32;

        let mut executions: Vec<Execution> = runs.into_iter().map(|r| r.into()).collect();
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
            "Zapier does not support fetching individual executions".to_string(),
        ))
    }

    async fn cancel_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Zapier does not support cancelling executions".to_string(),
        ))
    }

    async fn retry_execution(&self, _id: &str) -> Result<Execution> {
        Err(Error::Provider(
            "Zapier retry must be done through the Zapier UI".to_string(),
        ))
    }

    async fn list_connections(&self, _options: ListOptions) -> Result<ListResult<Connection>> {
        Err(Error::Provider(
            "Zapier connections are managed through the Zapier UI".to_string(),
        ))
    }

    async fn get_connection(&self, _id: &str) -> Result<Connection> {
        Err(Error::Provider(
            "Zapier connections are managed through the Zapier UI".to_string(),
        ))
    }

    async fn delete_connection(&self, _id: &str) -> Result<()> {
        Err(Error::Provider(
            "Zapier connections are managed through the Zapier UI".to_string(),
        ))
    }

    async fn list_apps(&self) -> Result<Vec<App>> {
        let nla_actions = self.list_nla_actions().await?;

        Ok(nla_actions
            .into_iter()
            .map(|a| App {
                id: a.id.clone(),
                name: a.description.clone(),
                slug: a.operation_id,
                description: Some(a.description),
                icon_url: None,
                categories: Vec::new(),
                triggers: Vec::new(),
                actions: vec![AppAction {
                    id: a.id,
                    name: "Execute".to_string(),
                    description: None,
                }],
            })
            .collect())
    }

    async fn get_app(&self, id: &str) -> Result<App> {
        let apps = self.list_apps().await?;
        apps.into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| Error::NotFound(format!("App {}", id)))
    }
}
