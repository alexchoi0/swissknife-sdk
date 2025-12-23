use crate::error::Result;
use crate::tool::{get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct PlaidGetAccountsTool;

impl Default for PlaidGetAccountsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for PlaidGetAccountsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "plaid_get_accounts",
            "Plaid Get Accounts",
            "Retrieve linked bank accounts from Plaid",
            "banking",
        )
        .with_param("client_id", ParameterSchema::string("Plaid client ID").required().user_only())
        .with_param("secret", ParameterSchema::string("Plaid secret").required().user_only())
        .with_param("access_token", ParameterSchema::string("Plaid access token for the user").required().user_only())
        .with_param("environment", ParameterSchema::string("Plaid environment (sandbox, development, production)").with_default(serde_json::json!("sandbox")))
        .with_output("accounts", OutputSchema::array("List of bank accounts", OutputSchema::json("Account object")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let client_id = get_required_string_param(&params, "client_id")?;
        let secret = get_required_string_param(&params, "secret")?;
        let access_token = get_required_string_param(&params, "access_token")?;
        let environment = get_string_param(&params, "environment").unwrap_or_else(|| "sandbox".into());

        let base_url = match environment.as_str() {
            "production" => "https://production.plaid.com",
            "development" => "https://development.plaid.com",
            _ => "https://sandbox.plaid.com",
        };

        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "client_id": client_id,
            "secret": secret,
            "access_token": access_token
        });

        match client
            .post(format!("{}/accounts/get", base_url))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                if body.get("error_code").is_some() {
                    let error_msg = body.get("error_message")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Unknown error");
                    Ok(ToolResponse::error(format!("Plaid error: {}", error_msg)))
                } else {
                    Ok(ToolResponse::success(serde_json::json!({
                        "accounts": body.get("accounts").cloned().unwrap_or(serde_json::json!([])),
                        "item": body.get("item").cloned(),
                        "success": true
                    })))
                }
            }
            Err(e) => Ok(ToolResponse::error(format!("Request failed: {}", e))),
        }
    }
}

pub struct PlaidGetTransactionsTool;

impl Default for PlaidGetTransactionsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for PlaidGetTransactionsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "plaid_get_transactions",
            "Plaid Get Transactions",
            "Retrieve transactions from linked bank accounts",
            "banking",
        )
        .with_param("client_id", ParameterSchema::string("Plaid client ID").required().user_only())
        .with_param("secret", ParameterSchema::string("Plaid secret").required().user_only())
        .with_param("access_token", ParameterSchema::string("Plaid access token for the user").required().user_only())
        .with_param("start_date", ParameterSchema::string("Start date (YYYY-MM-DD)").required())
        .with_param("end_date", ParameterSchema::string("End date (YYYY-MM-DD)").required())
        .with_param("environment", ParameterSchema::string("Plaid environment (sandbox, development, production)").with_default(serde_json::json!("sandbox")))
        .with_output("transactions", OutputSchema::array("List of transactions", OutputSchema::json("Transaction object")))
        .with_output("total_transactions", OutputSchema::number("Total number of transactions"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let client_id = get_required_string_param(&params, "client_id")?;
        let secret = get_required_string_param(&params, "secret")?;
        let access_token = get_required_string_param(&params, "access_token")?;
        let start_date = get_required_string_param(&params, "start_date")?;
        let end_date = get_required_string_param(&params, "end_date")?;
        let environment = get_string_param(&params, "environment").unwrap_or_else(|| "sandbox".into());

        let base_url = match environment.as_str() {
            "production" => "https://production.plaid.com",
            "development" => "https://development.plaid.com",
            _ => "https://sandbox.plaid.com",
        };

        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "client_id": client_id,
            "secret": secret,
            "access_token": access_token,
            "start_date": start_date,
            "end_date": end_date
        });

        match client
            .post(format!("{}/transactions/get", base_url))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                if body.get("error_code").is_some() {
                    let error_msg = body.get("error_message")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Unknown error");
                    Ok(ToolResponse::error(format!("Plaid error: {}", error_msg)))
                } else {
                    Ok(ToolResponse::success(serde_json::json!({
                        "transactions": body.get("transactions").cloned().unwrap_or(serde_json::json!([])),
                        "total_transactions": body.get("total_transactions"),
                        "accounts": body.get("accounts").cloned(),
                        "success": true
                    })))
                }
            }
            Err(e) => Ok(ToolResponse::error(format!("Request failed: {}", e))),
        }
    }
}
