use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "hr")]
use swissknife_hr_sdk as hr;

#[derive(Clone)]
pub struct HrTools {
    #[cfg(feature = "bamboohr")]
    pub bamboohr: Option<hr::bamboohr::BambooHRClient>,
    #[cfg(feature = "gusto")]
    pub gusto: Option<hr::gusto::GustoClient>,
    #[cfg(feature = "workday")]
    pub workday: Option<hr::workday::WorkdayClient>,
}

impl HrTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "bamboohr")]
            bamboohr: None,
            #[cfg(feature = "gusto")]
            gusto: None,
            #[cfg(feature = "workday")]
            workday: None,
        }
    }

    #[cfg(feature = "bamboohr")]
    pub fn with_bamboohr(mut self, client: hr::bamboohr::BambooHRClient) -> Self {
        self.bamboohr = Some(client);
        self
    }

    #[cfg(feature = "gusto")]
    pub fn with_gusto(mut self, client: hr::gusto::GustoClient) -> Self {
        self.gusto = Some(client);
        self
    }

    #[cfg(feature = "workday")]
    pub fn with_workday(mut self, client: hr::workday::WorkdayClient) -> Self {
        self.workday = Some(client);
        self
    }
}

impl Default for HrTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BambooHRGetEmployeeRequest {
    pub employee_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BambooHRListEmployeesRequest {
    #[serde(default)]
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BambooHRGetDirectoryRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BambooHRGetTimeOffRequest {
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub employee_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GustoListEmployeesRequest {
    pub company_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GustoGetEmployeeRequest {
    pub employee_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GustoListPayrollsRequest {
    pub company_id: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkdayGetWorkerRequest {
    pub worker_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkdaySearchWorkersRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[tool_router]
impl HrTools {
    #[cfg(feature = "bamboohr")]
    #[rmcp::tool(description = "Get an employee from BambooHR")]
    pub async fn bamboohr_get_employee(
        &self,
        #[rmcp::tool(aggr)] req: BambooHRGetEmployeeRequest,
    ) -> Result<String, String> {
        let client = self.bamboohr.as_ref()
            .ok_or_else(|| "BambooHR client not configured".to_string())?;

        let employee = client.get_employee(&req.employee_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&employee).map_err(|e| e.to_string())
    }

    #[cfg(feature = "bamboohr")]
    #[rmcp::tool(description = "List employees from BambooHR")]
    pub async fn bamboohr_list_employees(
        &self,
        #[rmcp::tool(aggr)] req: BambooHRListEmployeesRequest,
    ) -> Result<String, String> {
        let client = self.bamboohr.as_ref()
            .ok_or_else(|| "BambooHR client not configured".to_string())?;

        let employees = client.list_employees(req.fields.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&employees).map_err(|e| e.to_string())
    }

    #[cfg(feature = "bamboohr")]
    #[rmcp::tool(description = "Get the company directory from BambooHR")]
    pub async fn bamboohr_get_directory(
        &self,
        #[rmcp::tool(aggr)] _req: BambooHRGetDirectoryRequest,
    ) -> Result<String, String> {
        let client = self.bamboohr.as_ref()
            .ok_or_else(|| "BambooHR client not configured".to_string())?;

        let directory = client.get_directory().await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&directory).map_err(|e| e.to_string())
    }

    #[cfg(feature = "bamboohr")]
    #[rmcp::tool(description = "Get time off requests from BambooHR")]
    pub async fn bamboohr_get_time_off(
        &self,
        #[rmcp::tool(aggr)] req: BambooHRGetTimeOffRequest,
    ) -> Result<String, String> {
        let client = self.bamboohr.as_ref()
            .ok_or_else(|| "BambooHR client not configured".to_string())?;

        let requests = client.get_time_off_requests(
            &req.start_date,
            &req.end_date,
            req.employee_id.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&requests).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gusto")]
    #[rmcp::tool(description = "List employees from Gusto")]
    pub async fn gusto_list_employees(
        &self,
        #[rmcp::tool(aggr)] req: GustoListEmployeesRequest,
    ) -> Result<String, String> {
        let client = self.gusto.as_ref()
            .ok_or_else(|| "Gusto client not configured".to_string())?;

        let employees = client.list_employees(&req.company_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&employees).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gusto")]
    #[rmcp::tool(description = "Get an employee from Gusto")]
    pub async fn gusto_get_employee(
        &self,
        #[rmcp::tool(aggr)] req: GustoGetEmployeeRequest,
    ) -> Result<String, String> {
        let client = self.gusto.as_ref()
            .ok_or_else(|| "Gusto client not configured".to_string())?;

        let employee = client.get_employee(&req.employee_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&employee).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gusto")]
    #[rmcp::tool(description = "List payrolls from Gusto")]
    pub async fn gusto_list_payrolls(
        &self,
        #[rmcp::tool(aggr)] req: GustoListPayrollsRequest,
    ) -> Result<String, String> {
        let client = self.gusto.as_ref()
            .ok_or_else(|| "Gusto client not configured".to_string())?;

        let payrolls = client.list_payrolls(
            &req.company_id,
            req.start_date.as_deref(),
            req.end_date.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&payrolls).map_err(|e| e.to_string())
    }

    #[cfg(feature = "workday")]
    #[rmcp::tool(description = "Get a worker from Workday")]
    pub async fn workday_get_worker(
        &self,
        #[rmcp::tool(aggr)] req: WorkdayGetWorkerRequest,
    ) -> Result<String, String> {
        let client = self.workday.as_ref()
            .ok_or_else(|| "Workday client not configured".to_string())?;

        let worker = client.get_worker(&req.worker_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&worker).map_err(|e| e.to_string())
    }

    #[cfg(feature = "workday")]
    #[rmcp::tool(description = "Search for workers in Workday")]
    pub async fn workday_search_workers(
        &self,
        #[rmcp::tool(aggr)] req: WorkdaySearchWorkersRequest,
    ) -> Result<String, String> {
        let client = self.workday.as_ref()
            .ok_or_else(|| "Workday client not configured".to_string())?;

        let workers = client.search_workers(&req.query, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&workers).map_err(|e| e.to_string())
    }
}
