use crate::error::Result;
use crate::tool::{get_f64_param, get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_crm_sdk::salesforce::SalesforceClient;
use swissknife_crm_sdk::{Contact, CrmProvider, Deal, ListOptions};

pub struct SalesforceCreateContactTool;

impl Default for SalesforceCreateContactTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SalesforceCreateContactTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "salesforce_create_contact",
            "Salesforce Create Contact",
            "Create a new contact in Salesforce CRM",
            "crm",
        )
        .with_param("instance_url", ParameterSchema::string("Salesforce instance URL").required().user_only())
        .with_param("client_id", ParameterSchema::string("Salesforce OAuth client ID").required().user_only())
        .with_param("client_secret", ParameterSchema::string("Salesforce OAuth client secret").required().user_only())
        .with_param("access_token", ParameterSchema::string("Salesforce access token").required().user_only())
        .with_param("email", ParameterSchema::string("Contact email address"))
        .with_param("first_name", ParameterSchema::string("Contact first name"))
        .with_param("last_name", ParameterSchema::string("Contact last name").required())
        .with_param("phone", ParameterSchema::string("Contact phone number"))
        .with_param("title", ParameterSchema::string("Contact job title"))
        .with_output("contact", OutputSchema::json("The created contact object"))
        .with_output("contact_id", OutputSchema::string("The created contact ID"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let instance_url = get_required_string_param(&params, "instance_url")?;
        let client_id = get_required_string_param(&params, "client_id")?;
        let client_secret = get_required_string_param(&params, "client_secret")?;
        let access_token = get_required_string_param(&params, "access_token")?;

        let email = get_string_param(&params, "email");
        let first_name = get_string_param(&params, "first_name");
        let last_name = get_required_string_param(&params, "last_name")?;
        let phone = get_string_param(&params, "phone");
        let title = get_string_param(&params, "title");

        let client = SalesforceClient::new(instance_url, client_id, client_secret)
            .with_access_token(access_token);

        let mut contact = Contact::new();
        if let Some(e) = email {
            contact = contact.with_email(e);
        }
        if let Some(first) = first_name {
            contact = contact.with_name(first, last_name.clone());
        } else {
            contact.last_name = Some(last_name);
        }
        if let Some(p) = phone {
            contact = contact.with_phone(p);
        }
        if let Some(t) = title {
            contact = contact.with_title(t);
        }

        match client.create_contact(&contact).await {
            Ok(created) => Ok(ToolResponse::success(serde_json::json!({
                "contact": created,
                "contact_id": created.id,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to create contact: {}", e))),
        }
    }
}

pub struct SalesforceGetContactTool;

impl Default for SalesforceGetContactTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SalesforceGetContactTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "salesforce_get_contact",
            "Salesforce Get Contact",
            "Retrieve a contact from Salesforce by ID",
            "crm",
        )
        .with_param("instance_url", ParameterSchema::string("Salesforce instance URL").required().user_only())
        .with_param("client_id", ParameterSchema::string("Salesforce OAuth client ID").required().user_only())
        .with_param("client_secret", ParameterSchema::string("Salesforce OAuth client secret").required().user_only())
        .with_param("access_token", ParameterSchema::string("Salesforce access token").required().user_only())
        .with_param("contact_id", ParameterSchema::string("The contact ID to retrieve").required())
        .with_output("contact", OutputSchema::json("The contact object"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let instance_url = get_required_string_param(&params, "instance_url")?;
        let client_id = get_required_string_param(&params, "client_id")?;
        let client_secret = get_required_string_param(&params, "client_secret")?;
        let access_token = get_required_string_param(&params, "access_token")?;
        let contact_id = get_required_string_param(&params, "contact_id")?;

        let client = SalesforceClient::new(instance_url, client_id, client_secret)
            .with_access_token(access_token);

        match client.get_contact(&contact_id).await {
            Ok(contact) => Ok(ToolResponse::success(serde_json::json!({
                "contact": contact,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to get contact: {}", e))),
        }
    }
}

pub struct SalesforceListContactsTool;

impl Default for SalesforceListContactsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SalesforceListContactsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "salesforce_list_contacts",
            "Salesforce List Contacts",
            "List contacts from Salesforce with optional filtering",
            "crm",
        )
        .with_param("instance_url", ParameterSchema::string("Salesforce instance URL").required().user_only())
        .with_param("client_id", ParameterSchema::string("Salesforce OAuth client ID").required().user_only())
        .with_param("client_secret", ParameterSchema::string("Salesforce OAuth client secret").required().user_only())
        .with_param("access_token", ParameterSchema::string("Salesforce access token").required().user_only())
        .with_param("limit", ParameterSchema::integer("Maximum number of contacts to return").with_default(serde_json::json!(100)))
        .with_param("query", ParameterSchema::string("Search query to filter contacts"))
        .with_output("contacts", OutputSchema::array("List of contacts", OutputSchema::json("Contact object")))
        .with_output("total", OutputSchema::number("Total number of matching contacts"))
        .with_output("has_more", OutputSchema::boolean("Whether there are more results"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let instance_url = get_required_string_param(&params, "instance_url")?;
        let client_id = get_required_string_param(&params, "client_id")?;
        let client_secret = get_required_string_param(&params, "client_secret")?;
        let access_token = get_required_string_param(&params, "access_token")?;
        let limit = get_i64_param(&params, "limit").map(|l| l as u32);
        let query = get_string_param(&params, "query");

        let client = SalesforceClient::new(instance_url, client_id, client_secret)
            .with_access_token(access_token);

        let options = ListOptions {
            limit,
            query,
            ..Default::default()
        };

        match client.list_contacts(&options).await {
            Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                "contacts": result.items,
                "total": result.total,
                "has_more": result.has_more,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list contacts: {}", e))),
        }
    }
}

pub struct SalesforceCreateDealTool;

impl Default for SalesforceCreateDealTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SalesforceCreateDealTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "salesforce_create_deal",
            "Salesforce Create Deal/Opportunity",
            "Create a new opportunity/deal in Salesforce",
            "crm",
        )
        .with_param("instance_url", ParameterSchema::string("Salesforce instance URL").required().user_only())
        .with_param("client_id", ParameterSchema::string("Salesforce OAuth client ID").required().user_only())
        .with_param("client_secret", ParameterSchema::string("Salesforce OAuth client secret").required().user_only())
        .with_param("access_token", ParameterSchema::string("Salesforce access token").required().user_only())
        .with_param("name", ParameterSchema::string("Deal/opportunity name").required())
        .with_param("amount", ParameterSchema::number("Deal amount"))
        .with_param("stage", ParameterSchema::string("Deal stage"))
        .with_param("close_date", ParameterSchema::string("Expected close date (YYYY-MM-DD)"))
        .with_param("account_id", ParameterSchema::string("Associated account ID"))
        .with_output("deal", OutputSchema::json("The created deal object"))
        .with_output("deal_id", OutputSchema::string("The created deal ID"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let instance_url = get_required_string_param(&params, "instance_url")?;
        let client_id = get_required_string_param(&params, "client_id")?;
        let client_secret = get_required_string_param(&params, "client_secret")?;
        let access_token = get_required_string_param(&params, "access_token")?;
        let name = get_required_string_param(&params, "name")?;
        let amount = get_f64_param(&params, "amount");
        let stage = get_string_param(&params, "stage");
        let _close_date = get_string_param(&params, "close_date");
        let account_id = get_string_param(&params, "account_id");

        let client = SalesforceClient::new(instance_url, client_id, client_secret)
            .with_access_token(access_token);

        let mut deal = Deal::new(name);
        if let Some(amt) = amount {
            deal = deal.with_amount(amt, "USD");
        }
        if let Some(s) = stage {
            deal = deal.with_stage(s);
        }
        if let Some(acc) = account_id {
            deal = deal.with_company(acc);
        }

        match client.create_deal(&deal).await {
            Ok(created) => Ok(ToolResponse::success(serde_json::json!({
                "deal": created,
                "deal_id": created.id,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to create deal: {}", e))),
        }
    }
}

pub struct SalesforceQueryTool;

impl Default for SalesforceQueryTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SalesforceQueryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "salesforce_query",
            "Salesforce SOQL Query",
            "Execute a SOQL query against Salesforce",
            "crm",
        )
        .with_param("instance_url", ParameterSchema::string("Salesforce instance URL").required().user_only())
        .with_param("client_id", ParameterSchema::string("Salesforce OAuth client ID").required().user_only())
        .with_param("client_secret", ParameterSchema::string("Salesforce OAuth client secret").required().user_only())
        .with_param("access_token", ParameterSchema::string("Salesforce access token").required().user_only())
        .with_param("query", ParameterSchema::string("SOQL query to execute").required())
        .with_output("records", OutputSchema::array("Query results", OutputSchema::json("Record object")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let instance_url = get_required_string_param(&params, "instance_url")?;
        let client_id = get_required_string_param(&params, "client_id")?;
        let client_secret = get_required_string_param(&params, "client_secret")?;
        let access_token = get_required_string_param(&params, "access_token")?;
        let query = get_required_string_param(&params, "query")?;

        let client = SalesforceClient::new(instance_url, client_id, client_secret)
            .with_access_token(access_token);

        match client.execute_soql::<serde_json::Value>(&query).await {
            Ok(records) => Ok(ToolResponse::success(serde_json::json!({
                "records": records,
                "count": records.len(),
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Query failed: {}", e))),
        }
    }
}
