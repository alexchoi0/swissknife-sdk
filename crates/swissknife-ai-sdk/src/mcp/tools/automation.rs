use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "automation")]
use swissknife_automation_sdk as automation;

#[derive(Clone)]
pub struct AutomationTools {
    #[cfg(feature = "zapier")]
    pub zapier: Option<automation::zapier::ZapierClient>,
    #[cfg(feature = "make")]
    pub make: Option<automation::make::MakeClient>,
    #[cfg(feature = "n8n")]
    pub n8n: Option<automation::n8n::N8nClient>,
}

impl AutomationTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "zapier")]
            zapier: None,
            #[cfg(feature = "make")]
            make: None,
            #[cfg(feature = "n8n")]
            n8n: None,
        }
    }

    #[cfg(feature = "zapier")]
    pub fn with_zapier(mut self, client: automation::zapier::ZapierClient) -> Self {
        self.zapier = Some(client);
        self
    }

    #[cfg(feature = "make")]
    pub fn with_make(mut self, client: automation::make::MakeClient) -> Self {
        self.make = Some(client);
        self
    }

    #[cfg(feature = "n8n")]
    pub fn with_n8n(mut self, client: automation::n8n::N8nClient) -> Self {
        self.n8n = Some(client);
        self
    }
}

impl Default for AutomationTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZapierTriggerWebhookRequest {
    pub webhook_url: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ZapierListZapsRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MakeTriggerScenarioRequest {
    pub scenario_id: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MakeListScenariosRequest {
    #[serde(default)]
    pub folder_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MakeGetScenarioRequest {
    pub scenario_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct N8nExecuteWorkflowRequest {
    pub workflow_id: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct N8nListWorkflowsRequest {
    #[serde(default)]
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct N8nGetWorkflowRequest {
    pub workflow_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct N8nActivateWorkflowRequest {
    pub workflow_id: String,
    pub active: bool,
}

#[tool_box]
impl AutomationTools {
    #[cfg(feature = "zapier")]
    #[rmcp::tool(description = "Trigger a Zapier webhook")]
    pub async fn zapier_trigger_webhook(
        &self,
        #[rmcp::tool(aggr)] req: ZapierTriggerWebhookRequest,
    ) -> Result<String, String> {
        let client = self.zapier.as_ref()
            .ok_or_else(|| "Zapier client not configured".to_string())?;

        let result = client.trigger_webhook(&req.webhook_url, req.data).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "zapier")]
    #[rmcp::tool(description = "List Zapier Zaps")]
    pub async fn zapier_list_zaps(
        &self,
        #[rmcp::tool(aggr)] _req: ZapierListZapsRequest,
    ) -> Result<String, String> {
        let client = self.zapier.as_ref()
            .ok_or_else(|| "Zapier client not configured".to_string())?;

        let zaps = client.list_zaps().await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&zaps).map_err(|e| e.to_string())
    }

    #[cfg(feature = "make")]
    #[rmcp::tool(description = "Trigger a Make (Integromat) scenario")]
    pub async fn make_trigger_scenario(
        &self,
        #[rmcp::tool(aggr)] req: MakeTriggerScenarioRequest,
    ) -> Result<String, String> {
        let client = self.make.as_ref()
            .ok_or_else(|| "Make client not configured".to_string())?;

        let result = client.trigger_scenario(&req.scenario_id, req.data).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "make")]
    #[rmcp::tool(description = "List Make scenarios")]
    pub async fn make_list_scenarios(
        &self,
        #[rmcp::tool(aggr)] req: MakeListScenariosRequest,
    ) -> Result<String, String> {
        let client = self.make.as_ref()
            .ok_or_else(|| "Make client not configured".to_string())?;

        let scenarios = client.list_scenarios(req.folder_id.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&scenarios).map_err(|e| e.to_string())
    }

    #[cfg(feature = "make")]
    #[rmcp::tool(description = "Get a Make scenario by ID")]
    pub async fn make_get_scenario(
        &self,
        #[rmcp::tool(aggr)] req: MakeGetScenarioRequest,
    ) -> Result<String, String> {
        let client = self.make.as_ref()
            .ok_or_else(|| "Make client not configured".to_string())?;

        let scenario = client.get_scenario(&req.scenario_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&scenario).map_err(|e| e.to_string())
    }

    #[cfg(feature = "n8n")]
    #[rmcp::tool(description = "Execute an n8n workflow")]
    pub async fn n8n_execute_workflow(
        &self,
        #[rmcp::tool(aggr)] req: N8nExecuteWorkflowRequest,
    ) -> Result<String, String> {
        let client = self.n8n.as_ref()
            .ok_or_else(|| "n8n client not configured".to_string())?;

        let result = client.execute_workflow(&req.workflow_id, req.data).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "n8n")]
    #[rmcp::tool(description = "List n8n workflows")]
    pub async fn n8n_list_workflows(
        &self,
        #[rmcp::tool(aggr)] req: N8nListWorkflowsRequest,
    ) -> Result<String, String> {
        let client = self.n8n.as_ref()
            .ok_or_else(|| "n8n client not configured".to_string())?;

        let workflows = client.list_workflows(req.active).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&workflows).map_err(|e| e.to_string())
    }

    #[cfg(feature = "n8n")]
    #[rmcp::tool(description = "Get an n8n workflow by ID")]
    pub async fn n8n_get_workflow(
        &self,
        #[rmcp::tool(aggr)] req: N8nGetWorkflowRequest,
    ) -> Result<String, String> {
        let client = self.n8n.as_ref()
            .ok_or_else(|| "n8n client not configured".to_string())?;

        let workflow = client.get_workflow(&req.workflow_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&workflow).map_err(|e| e.to_string())
    }

    #[cfg(feature = "n8n")]
    #[rmcp::tool(description = "Activate or deactivate an n8n workflow")]
    pub async fn n8n_activate_workflow(
        &self,
        #[rmcp::tool(aggr)] req: N8nActivateWorkflowRequest,
    ) -> Result<String, String> {
        let client = self.n8n.as_ref()
            .ok_or_else(|| "n8n client not configured".to_string())?;

        client.activate_workflow(&req.workflow_id, req.active).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Workflow {} {}", req.workflow_id, if req.active { "activated" } else { "deactivated" }))
    }
}
