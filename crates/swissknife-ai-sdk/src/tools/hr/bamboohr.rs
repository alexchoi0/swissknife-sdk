use crate::error::Result;
use crate::tool::{get_i64_param, get_required_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_hr_sdk::bamboohr::BambooHrClient;
use swissknife_hr_sdk::{HrProvider, ListOptions};

pub struct BambooHRListEmployeesTool;

impl Default for BambooHRListEmployeesTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for BambooHRListEmployeesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "bamboohr_list_employees",
            "BambooHR List Employees",
            "List employees from BambooHR directory",
            "hr",
        )
        .with_param("company_domain", ParameterSchema::string("BambooHR company subdomain").required().user_only())
        .with_param("api_key", ParameterSchema::string("BambooHR API key").required().user_only())
        .with_output("employees", OutputSchema::array("List of employees", OutputSchema::json("Employee object")))
        .with_output("count", OutputSchema::number("Number of employees"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let company_domain = get_required_string_param(&params, "company_domain")?;
        let api_key = get_required_string_param(&params, "api_key")?;

        let client = BambooHrClient::new(company_domain, api_key);

        match client.get_employee_directory().await {
            Ok(employees) => Ok(ToolResponse::success(serde_json::json!({
                "employees": employees,
                "count": employees.len(),
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list employees: {}", e))),
        }
    }
}

pub struct BambooHRGetEmployeeTool;

impl Default for BambooHRGetEmployeeTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for BambooHRGetEmployeeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "bamboohr_get_employee",
            "BambooHR Get Employee",
            "Get an employee's details from BambooHR",
            "hr",
        )
        .with_param("company_domain", ParameterSchema::string("BambooHR company subdomain").required().user_only())
        .with_param("api_key", ParameterSchema::string("BambooHR API key").required().user_only())
        .with_param("employee_id", ParameterSchema::string("Employee ID").required())
        .with_output("employee", OutputSchema::json("Employee object"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let company_domain = get_required_string_param(&params, "company_domain")?;
        let api_key = get_required_string_param(&params, "api_key")?;
        let employee_id = get_required_string_param(&params, "employee_id")?;

        let client = BambooHrClient::new(company_domain, api_key);
        let fields = [
            "firstName", "lastName", "email", "workEmail", "jobTitle",
            "department", "location", "hireDate", "employmentStatus",
        ];

        match client.get_employee_with_fields(&employee_id, &fields).await {
            Ok(employee) => Ok(ToolResponse::success(serde_json::json!({
                "employee": employee,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to get employee: {}", e))),
        }
    }
}

pub struct BambooHRTimeOffTool;

impl Default for BambooHRTimeOffTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for BambooHRTimeOffTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "bamboohr_whos_out",
            "BambooHR Who's Out",
            "Get who is out of office for a date range",
            "hr",
        )
        .with_param("company_domain", ParameterSchema::string("BambooHR company subdomain").required().user_only())
        .with_param("api_key", ParameterSchema::string("BambooHR API key").required().user_only())
        .with_param("start_date", ParameterSchema::string("Start date (YYYY-MM-DD)").required())
        .with_param("end_date", ParameterSchema::string("End date (YYYY-MM-DD)").required())
        .with_output("time_off_requests", OutputSchema::array("List of time off requests", OutputSchema::json("TimeOff object")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let company_domain = get_required_string_param(&params, "company_domain")?;
        let api_key = get_required_string_param(&params, "api_key")?;
        let start_date_str = get_required_string_param(&params, "start_date")?;
        let end_date_str = get_required_string_param(&params, "end_date")?;

        let start_date = chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
            .map_err(|e| crate::error::Error::InvalidParameter(format!("Invalid start_date: {}", e)))?;
        let end_date = chrono::NaiveDate::parse_from_str(&end_date_str, "%Y-%m-%d")
            .map_err(|e| crate::error::Error::InvalidParameter(format!("Invalid end_date: {}", e)))?;

        let client = BambooHrClient::new(company_domain, api_key);

        match client.who_is_out(start_date, end_date).await {
            Ok(requests) => Ok(ToolResponse::success(serde_json::json!({
                "time_off_requests": requests,
                "count": requests.len(),
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to get time off: {}", e))),
        }
    }
}
