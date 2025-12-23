use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "productivity")]
use swissknife_productivity_sdk as prod;

#[derive(Clone)]
pub struct ProductivityTools {
    #[cfg(feature = "notion")]
    pub notion: Option<prod::notion::NotionClient>,
    #[cfg(feature = "airtable")]
    pub airtable: Option<prod::airtable::AirtableClient>,
    #[cfg(feature = "google")]
    pub google: Option<prod::google::GoogleClient>,
}

impl ProductivityTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "notion")]
            notion: None,
            #[cfg(feature = "airtable")]
            airtable: None,
            #[cfg(feature = "google")]
            google: None,
        }
    }

    #[cfg(feature = "notion")]
    pub fn with_notion(mut self, client: prod::notion::NotionClient) -> Self {
        self.notion = Some(client);
        self
    }

    #[cfg(feature = "airtable")]
    pub fn with_airtable(mut self, client: prod::airtable::AirtableClient) -> Self {
        self.airtable = Some(client);
        self
    }

    #[cfg(feature = "google")]
    pub fn with_google(mut self, client: prod::google::GoogleClient) -> Self {
        self.google = Some(client);
        self
    }
}

impl Default for ProductivityTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NotionSearchRequest {
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NotionGetPageRequest {
    pub page_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NotionCreatePageRequest {
    pub parent_id: String,
    pub title: String,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NotionQueryDatabaseRequest {
    pub database_id: String,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
    #[serde(default)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AirtableListRecordsRequest {
    pub base_id: String,
    pub table_name: String,
    #[serde(default)]
    pub max_records: Option<u32>,
    #[serde(default)]
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AirtableCreateRecordRequest {
    pub base_id: String,
    pub table_name: String,
    pub fields: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AirtableUpdateRecordRequest {
    pub base_id: String,
    pub table_name: String,
    pub record_id: String,
    pub fields: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GoogleDriveListRequest {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GoogleCalendarEventsRequest {
    #[serde(default)]
    pub calendar_id: Option<String>,
    #[serde(default)]
    pub time_min: Option<String>,
    #[serde(default)]
    pub time_max: Option<String>,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GoogleCalendarCreateEventRequest {
    #[serde(default)]
    pub calendar_id: Option<String>,
    pub summary: String,
    pub start: String,
    pub end: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[tool_box]
impl ProductivityTools {
    #[cfg(feature = "notion")]
    #[rmcp::tool(description = "Search Notion pages and databases")]
    pub async fn notion_search(
        &self,
        #[rmcp::tool(aggr)] req: NotionSearchRequest,
    ) -> Result<String, String> {
        let client = self.notion.as_ref()
            .ok_or_else(|| "Notion client not configured".to_string())?;

        let results = client.search(&req.query).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.results
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "notion")]
    #[rmcp::tool(description = "Get a Notion page by ID")]
    pub async fn notion_get_page(
        &self,
        #[rmcp::tool(aggr)] req: NotionGetPageRequest,
    ) -> Result<String, String> {
        let client = self.notion.as_ref()
            .ok_or_else(|| "Notion client not configured".to_string())?;

        let page = client.get_page(&req.page_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&page)
            .map_err(|e| e.to_string())
    }

    #[cfg(feature = "notion")]
    #[rmcp::tool(description = "Create a new Notion page")]
    pub async fn notion_create_page(
        &self,
        #[rmcp::tool(aggr)] req: NotionCreatePageRequest,
    ) -> Result<String, String> {
        let client = self.notion.as_ref()
            .ok_or_else(|| "Notion client not configured".to_string())?;

        let request = prod::notion::CreatePageRequest {
            parent: serde_json::json!({"page_id": req.parent_id}),
            properties: serde_json::json!({
                "title": {"title": [{"text": {"content": req.title}}]}
            }),
            children: None,
            icon: None,
            cover: None,
        };

        let page = client.create_page(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": page.id,
            "url": page.url
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "notion")]
    #[rmcp::tool(description = "Query a Notion database")]
    pub async fn notion_query_database(
        &self,
        #[rmcp::tool(aggr)] req: NotionQueryDatabaseRequest,
    ) -> Result<String, String> {
        let client = self.notion.as_ref()
            .ok_or_else(|| "Notion client not configured".to_string())?;

        let request = prod::notion::QueryDatabaseRequest {
            filter: req.filter,
            sorts: None,
            start_cursor: None,
            page_size: req.page_size,
        };

        let results = client.query_database(&req.database_id, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.results,
            "has_more": results.has_more
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "airtable")]
    #[rmcp::tool(description = "List records from an Airtable table")]
    pub async fn airtable_list_records(
        &self,
        #[rmcp::tool(aggr)] req: AirtableListRecordsRequest,
    ) -> Result<String, String> {
        let client = self.airtable.as_ref()
            .ok_or_else(|| "Airtable client not configured".to_string())?;

        let params = prod::airtable::ListRecordsParams {
            fields: None,
            filter_by_formula: req.filter,
            max_records: req.max_records,
            page_size: None,
            sort: None,
            view: None,
            offset: None,
        };

        let records = client.list_records(&req.base_id, &req.table_name, Some(params)).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "records": records.records
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "airtable")]
    #[rmcp::tool(description = "Create a record in Airtable")]
    pub async fn airtable_create_record(
        &self,
        #[rmcp::tool(aggr)] req: AirtableCreateRecordRequest,
    ) -> Result<String, String> {
        let client = self.airtable.as_ref()
            .ok_or_else(|| "Airtable client not configured".to_string())?;

        let request = prod::airtable::CreateRecordRequest {
            fields: req.fields,
            typecast: Some(true),
        };

        let record = client.create_record(&req.base_id, &req.table_name, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": record.id,
            "fields": record.fields
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "airtable")]
    #[rmcp::tool(description = "Update an Airtable record")]
    pub async fn airtable_update_record(
        &self,
        #[rmcp::tool(aggr)] req: AirtableUpdateRecordRequest,
    ) -> Result<String, String> {
        let client = self.airtable.as_ref()
            .ok_or_else(|| "Airtable client not configured".to_string())?;

        let request = prod::airtable::UpdateRecordRequest {
            fields: req.fields,
            typecast: Some(true),
        };

        let record = client.update_record(&req.base_id, &req.table_name, &req.record_id, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": record.id,
            "fields": record.fields
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "google")]
    #[rmcp::tool(description = "List files in Google Drive")]
    pub async fn google_drive_list(
        &self,
        #[rmcp::tool(aggr)] req: GoogleDriveListRequest,
    ) -> Result<String, String> {
        let client = self.google.as_ref()
            .ok_or_else(|| "Google client not configured".to_string())?;

        let params = prod::google::ListFilesParams {
            q: req.query,
            page_size: req.page_size,
            page_token: None,
            order_by: None,
            fields: None,
        };

        let files = client.list_files(Some(params)).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "files": files.files
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "google")]
    #[rmcp::tool(description = "List Google Calendar events")]
    pub async fn google_calendar_events(
        &self,
        #[rmcp::tool(aggr)] req: GoogleCalendarEventsRequest,
    ) -> Result<String, String> {
        let client = self.google.as_ref()
            .ok_or_else(|| "Google client not configured".to_string())?;

        let calendar_id = req.calendar_id.unwrap_or_else(|| "primary".to_string());

        let events = client.list_events(
            &calendar_id,
            req.time_min.as_deref(),
            req.time_max.as_deref(),
            req.max_results,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "events": events.items
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "google")]
    #[rmcp::tool(description = "Create a Google Calendar event")]
    pub async fn google_calendar_create_event(
        &self,
        #[rmcp::tool(aggr)] req: GoogleCalendarCreateEventRequest,
    ) -> Result<String, String> {
        let client = self.google.as_ref()
            .ok_or_else(|| "Google client not configured".to_string())?;

        let calendar_id = req.calendar_id.unwrap_or_else(|| "primary".to_string());

        let request = prod::google::CreateEventRequest {
            summary: req.summary,
            description: req.description,
            start: serde_json::json!({"dateTime": req.start}),
            end: serde_json::json!({"dateTime": req.end}),
            attendees: None,
            location: None,
            recurrence: None,
            reminders: None,
        };

        let event = client.create_event(&calendar_id, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": event.id,
            "htmlLink": event.html_link
        })).map_err(|e| e.to_string())
    }
}
