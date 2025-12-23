use crate::error::Result;
use crate::tool::{get_object_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_productivity_sdk::{DocumentProvider, DatabaseProvider, CalendarProvider};

pub struct NotionGetPageTool;

impl Default for NotionGetPageTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for NotionGetPageTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "notion_get_page",
            "Notion Get Page",
            "Get a page from Notion by ID",
            "productivity",
        )
        .with_param("api_key", ParameterSchema::string("Notion API key").required().user_only())
        .with_param("page_id", ParameterSchema::string("Page ID").required())
        .with_output("page", OutputSchema::json("Page content and properties"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let page_id = get_required_string_param(&params, "page_id")?;

        #[cfg(feature = "notion")]
        {
            use swissknife_productivity_sdk::notion::NotionClient;
            let client = NotionClient::new(&api_key);
            match client.get_document(&page_id).await {
                Ok(doc) => Ok(ToolResponse::success(serde_json::json!({
                    "page": {
                        "id": doc.id,
                        "title": doc.title,
                        "content": doc.content,
                        "markdown": doc.markdown,
                        "url": doc.url,
                        "properties": doc.properties,
                    }
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to get page: {}", e))),
            }
        }
        #[cfg(not(feature = "notion"))]
        {
            let _ = (api_key, page_id);
            Ok(ToolResponse::error("Notion feature not enabled"))
        }
    }
}

pub struct NotionCreatePageTool;

impl Default for NotionCreatePageTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for NotionCreatePageTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "notion_create_page",
            "Notion Create Page",
            "Create a new page in Notion",
            "productivity",
        )
        .with_param("api_key", ParameterSchema::string("Notion API key").required().user_only())
        .with_param("parent_id", ParameterSchema::string("Parent page or database ID"))
        .with_param("title", ParameterSchema::string("Page title").required())
        .with_param("content", ParameterSchema::string("Page content in markdown"))
        .with_output("page", OutputSchema::json("Created page"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let parent_id = get_string_param(&params, "parent_id");
        let title = get_required_string_param(&params, "title")?;
        let content = get_string_param(&params, "content");

        #[cfg(feature = "notion")]
        {
            use swissknife_productivity_sdk::notion::NotionClient;
            let client = NotionClient::new(&api_key);
            match client.create_document(parent_id.as_deref(), &title, content.as_deref()).await {
                Ok(doc) => Ok(ToolResponse::success(serde_json::json!({
                    "page": {
                        "id": doc.id,
                        "title": doc.title,
                        "url": doc.url,
                    }
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to create page: {}", e))),
            }
        }
        #[cfg(not(feature = "notion"))]
        {
            let _ = (api_key, parent_id, title, content);
            Ok(ToolResponse::error("Notion feature not enabled"))
        }
    }
}

pub struct NotionSearchTool;

impl Default for NotionSearchTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for NotionSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "notion_search",
            "Notion Search",
            "Search for pages and databases in Notion",
            "productivity",
        )
        .with_param("api_key", ParameterSchema::string("Notion API key").required().user_only())
        .with_param("query", ParameterSchema::string("Search query").required())
        .with_output("results", OutputSchema::array("Search results", OutputSchema::json("Result")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let query = get_required_string_param(&params, "query")?;

        #[cfg(feature = "notion")]
        {
            use swissknife_productivity_sdk::notion::NotionClient;
            let client = NotionClient::new(&api_key);
            match client.search(&query).await {
                Ok(results) => Ok(ToolResponse::success(serde_json::json!({
                    "results": results.iter().map(|r| serde_json::json!({
                        "id": r.id,
                        "title": r.title,
                        "url": r.url,
                    })).collect::<Vec<_>>(),
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Search failed: {}", e))),
            }
        }
        #[cfg(not(feature = "notion"))]
        {
            let _ = (api_key, query);
            Ok(ToolResponse::error("Notion feature not enabled"))
        }
    }
}

pub struct NotionQueryDatabaseTool;

impl Default for NotionQueryDatabaseTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for NotionQueryDatabaseTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "notion_query_database",
            "Notion Query Database",
            "Query a Notion database with filters",
            "productivity",
        )
        .with_param("api_key", ParameterSchema::string("Notion API key").required().user_only())
        .with_param("database_id", ParameterSchema::string("Database ID").required())
        .with_param("filter", ParameterSchema::json("Filter object"))
        .with_output("entries", OutputSchema::array("Database entries", OutputSchema::json("Entry")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let database_id = get_required_string_param(&params, "database_id")?;
        let _filter = get_object_param(&params, "filter");

        #[cfg(feature = "notion")]
        {
            use swissknife_productivity_sdk::notion::NotionClient;
            use swissknife_productivity_sdk::QueryFilter;
            let client = NotionClient::new(&api_key);
            let query_filter = QueryFilter::default();
            match client.query_database(&database_id, &query_filter).await {
                Ok(entries) => Ok(ToolResponse::success(serde_json::json!({
                    "entries": entries.iter().map(|e| serde_json::json!({
                        "id": e.id,
                        "properties": e.properties,
                    })).collect::<Vec<_>>(),
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Query failed: {}", e))),
            }
        }
        #[cfg(not(feature = "notion"))]
        {
            let _ = (api_key, database_id, _filter);
            Ok(ToolResponse::error("Notion feature not enabled"))
        }
    }
}

pub struct AirtableListRecordsTool;

impl Default for AirtableListRecordsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for AirtableListRecordsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "airtable_list_records",
            "Airtable List Records",
            "List records from an Airtable table",
            "productivity",
        )
        .with_param("api_key", ParameterSchema::string("Airtable API key").required().user_only())
        .with_param("base_id", ParameterSchema::string("Airtable base ID").required())
        .with_param("table_name", ParameterSchema::string("Table name").required())
        .with_param("filter", ParameterSchema::string("Filter formula"))
        .with_output("records", OutputSchema::array("Table records", OutputSchema::json("Record")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let base_id = get_required_string_param(&params, "base_id")?;
        let table_name = get_required_string_param(&params, "table_name")?;
        let _filter = get_string_param(&params, "filter");

        #[cfg(feature = "airtable")]
        {
            use swissknife_productivity_sdk::airtable::AirtableClient;
            use swissknife_productivity_sdk::QueryFilter;
            let client = AirtableClient::new(&api_key, &base_id);
            let query_filter = QueryFilter::default();
            match client.query_database(&table_name, &query_filter).await {
                Ok(records) => Ok(ToolResponse::success(serde_json::json!({
                    "records": records.iter().map(|r| serde_json::json!({
                        "id": r.id,
                        "properties": r.properties,
                    })).collect::<Vec<_>>(),
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to list records: {}", e))),
            }
        }
        #[cfg(not(feature = "airtable"))]
        {
            let _ = (api_key, base_id, table_name, _filter);
            Ok(ToolResponse::error("Airtable feature not enabled"))
        }
    }
}

pub struct GoogleCalendarListEventsTool;

impl Default for GoogleCalendarListEventsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GoogleCalendarListEventsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "google_calendar_list_events",
            "Google Calendar List Events",
            "List events from Google Calendar",
            "productivity",
        )
        .with_param("access_token", ParameterSchema::string("Google OAuth access token").required().user_only())
        .with_param("calendar_id", ParameterSchema::string("Calendar ID").with_default(serde_json::json!("primary")))
        .with_param("start_time", ParameterSchema::string("Start time (ISO 8601)").required())
        .with_param("end_time", ParameterSchema::string("End time (ISO 8601)").required())
        .with_output("events", OutputSchema::array("Calendar events", OutputSchema::json("Event")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_token = get_required_string_param(&params, "access_token")?;
        let calendar_id = get_string_param(&params, "calendar_id").unwrap_or_else(|| "primary".to_string());
        let start_time = get_required_string_param(&params, "start_time")?;
        let end_time = get_required_string_param(&params, "end_time")?;

        #[cfg(feature = "google")]
        {
            use swissknife_productivity_sdk::google::GoogleCalendarClient;
            use chrono::DateTime;
            let client = GoogleCalendarClient::new(&access_token);
            let start = DateTime::parse_from_rfc3339(&start_time)
                .map_err(|_| crate::Error::InvalidParameter("start_time".into()))?
                .with_timezone(&chrono::Utc);
            let end = DateTime::parse_from_rfc3339(&end_time)
                .map_err(|_| crate::Error::InvalidParameter("end_time".into()))?
                .with_timezone(&chrono::Utc);
            match client.list_events(&calendar_id, start, end).await {
                Ok(events) => Ok(ToolResponse::success(serde_json::json!({
                    "events": events.iter().map(|e| serde_json::json!({
                        "id": e.id,
                        "title": e.title,
                        "description": e.description,
                        "start_time": e.start_time.to_rfc3339(),
                        "end_time": e.end_time.to_rfc3339(),
                        "location": e.location,
                        "attendees": e.attendees,
                    })).collect::<Vec<_>>(),
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to list events: {}", e))),
            }
        }
        #[cfg(not(feature = "google"))]
        {
            let _ = (access_token, calendar_id, start_time, end_time);
            Ok(ToolResponse::error("Google feature not enabled"))
        }
    }
}
