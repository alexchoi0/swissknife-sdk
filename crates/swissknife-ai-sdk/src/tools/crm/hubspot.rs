use crate::error::Result;
use crate::tool::{get_f64_param, get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_crm_sdk::hubspot::HubSpotClient;
use swissknife_crm_sdk::{Contact, CrmProvider, Deal, ListOptions};

pub struct HubSpotCreateContactTool;

impl Default for HubSpotCreateContactTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for HubSpotCreateContactTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "hubspot_create_contact",
            "HubSpot Create Contact",
            "Create a new contact in HubSpot CRM",
            "crm",
        )
        .with_param("access_token", ParameterSchema::string("HubSpot access token").required().user_only())
        .with_param("email", ParameterSchema::string("Contact email address"))
        .with_param("first_name", ParameterSchema::string("Contact first name"))
        .with_param("last_name", ParameterSchema::string("Contact last name"))
        .with_param("phone", ParameterSchema::string("Contact phone number"))
        .with_param("company", ParameterSchema::string("Contact company name"))
        .with_param("job_title", ParameterSchema::string("Contact job title"))
        .with_output("contact", OutputSchema::json("The created contact object"))
        .with_output("contact_id", OutputSchema::string("The created contact ID"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_token = get_required_string_param(&params, "access_token")?;
        let email = get_string_param(&params, "email");
        let first_name = get_string_param(&params, "first_name");
        let last_name = get_string_param(&params, "last_name");
        let phone = get_string_param(&params, "phone");
        let company = get_string_param(&params, "company");
        let job_title = get_string_param(&params, "job_title");

        let client = HubSpotClient::new(access_token);

        let mut contact = Contact::new();
        if let Some(e) = email {
            contact = contact.with_email(e);
        }
        if let (Some(first), Some(last)) = (first_name, last_name) {
            contact = contact.with_name(first, last);
        }
        if let Some(p) = phone {
            contact = contact.with_phone(p);
        }
        if let Some(c) = company {
            contact = contact.with_company(c);
        }
        if let Some(t) = job_title {
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

pub struct HubSpotGetContactTool;

impl Default for HubSpotGetContactTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for HubSpotGetContactTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "hubspot_get_contact",
            "HubSpot Get Contact",
            "Retrieve a contact from HubSpot by ID",
            "crm",
        )
        .with_param("access_token", ParameterSchema::string("HubSpot access token").required().user_only())
        .with_param("contact_id", ParameterSchema::string("The contact ID to retrieve").required())
        .with_output("contact", OutputSchema::json("The contact object"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_token = get_required_string_param(&params, "access_token")?;
        let contact_id = get_required_string_param(&params, "contact_id")?;

        let client = HubSpotClient::new(access_token);

        match client.get_contact(&contact_id).await {
            Ok(contact) => Ok(ToolResponse::success(serde_json::json!({
                "contact": contact,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to get contact: {}", e))),
        }
    }
}

pub struct HubSpotListContactsTool;

impl Default for HubSpotListContactsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for HubSpotListContactsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "hubspot_list_contacts",
            "HubSpot List Contacts",
            "List contacts from HubSpot with pagination",
            "crm",
        )
        .with_param("access_token", ParameterSchema::string("HubSpot access token").required().user_only())
        .with_param("limit", ParameterSchema::integer("Maximum number of contacts to return").with_default(serde_json::json!(100)))
        .with_param("cursor", ParameterSchema::string("Pagination cursor for next page"))
        .with_output("contacts", OutputSchema::array("List of contacts", OutputSchema::json("Contact object")))
        .with_output("has_more", OutputSchema::boolean("Whether there are more results"))
        .with_output("next_cursor", OutputSchema::string("Cursor for next page").optional())
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_token = get_required_string_param(&params, "access_token")?;
        let limit = get_i64_param(&params, "limit").map(|l| l as u32);
        let cursor = get_string_param(&params, "cursor");

        let client = HubSpotClient::new(access_token);

        let options = ListOptions {
            limit,
            cursor,
            ..Default::default()
        };

        match client.list_contacts(&options).await {
            Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                "contacts": result.items,
                "has_more": result.has_more,
                "next_cursor": result.next_cursor,
                "total": result.total,
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list contacts: {}", e))),
        }
    }
}

pub struct HubSpotSearchContactsTool;

impl Default for HubSpotSearchContactsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for HubSpotSearchContactsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "hubspot_search_contacts",
            "HubSpot Search Contacts",
            "Search for contacts in HubSpot using a query",
            "crm",
        )
        .with_param("access_token", ParameterSchema::string("HubSpot access token").required().user_only())
        .with_param("query", ParameterSchema::string("Search query").required())
        .with_output("contacts", OutputSchema::array("List of matching contacts", OutputSchema::json("Contact object")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_token = get_required_string_param(&params, "access_token")?;
        let query = get_required_string_param(&params, "query")?;

        let client = HubSpotClient::new(access_token);

        match client.search_contacts(&query).await {
            Ok(contacts) => Ok(ToolResponse::success(serde_json::json!({
                "contacts": contacts,
                "count": contacts.len(),
                "success": true
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Search failed: {}", e))),
        }
    }
}

pub struct HubSpotCreateDealTool;

impl Default for HubSpotCreateDealTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for HubSpotCreateDealTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "hubspot_create_deal",
            "HubSpot Create Deal",
            "Create a new deal in HubSpot CRM",
            "crm",
        )
        .with_param("access_token", ParameterSchema::string("HubSpot access token").required().user_only())
        .with_param("name", ParameterSchema::string("Deal name").required())
        .with_param("amount", ParameterSchema::number("Deal amount"))
        .with_param("stage", ParameterSchema::string("Deal stage ID"))
        .with_param("pipeline", ParameterSchema::string("Pipeline ID"))
        .with_param("close_date", ParameterSchema::string("Expected close date (YYYY-MM-DD)"))
        .with_output("deal", OutputSchema::json("The created deal object"))
        .with_output("deal_id", OutputSchema::string("The created deal ID"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_token = get_required_string_param(&params, "access_token")?;
        let name = get_required_string_param(&params, "name")?;
        let amount = get_f64_param(&params, "amount");
        let stage = get_string_param(&params, "stage");
        let _pipeline = get_string_param(&params, "pipeline");

        let client = HubSpotClient::new(access_token);

        let mut deal = Deal::new(name);
        if let Some(amt) = amount {
            deal = deal.with_amount(amt, "USD");
        }
        if let Some(s) = stage {
            deal = deal.with_stage(s);
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
