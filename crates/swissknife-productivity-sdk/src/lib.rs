mod error;

pub use error::{Error, Result};

#[cfg(feature = "notion")]
pub mod notion;

#[cfg(feature = "google")]
pub mod google;

#[cfg(feature = "airtable")]
pub mod airtable;

#[cfg(feature = "calendly")]
pub mod calendly;

#[cfg(feature = "confluence")]
pub mod confluence;

#[cfg(feature = "microsoft")]
pub mod microsoft;

#[cfg(feature = "typeform")]
pub mod typeform;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub markdown: Option<String>,
    pub url: Option<String>,
    pub parent_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    #[serde(flatten)]
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub properties: HashMap<String, PropertySchema>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    pub name: String,
    pub property_type: PropertyType,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Title,
    RichText,
    Number,
    Select,
    MultiSelect,
    Date,
    Checkbox,
    Url,
    Email,
    Phone,
    Formula,
    Relation,
    Rollup,
    CreatedTime,
    CreatedBy,
    LastEditedTime,
    LastEditedBy,
    Files,
    People,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseEntry {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub is_all_day: bool,
    pub recurrence: Option<String>,
    pub calendar_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub timezone: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: Option<u64>,
    pub url: Option<String>,
    pub parent_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    pub filters: Vec<FilterCondition>,
    pub sorts: Vec<SortCondition>,
}

#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub property: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    IsEmpty,
    IsNotEmpty,
}

#[derive(Debug, Clone)]
pub struct SortCondition {
    pub property: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[async_trait]
pub trait DocumentProvider: Send + Sync {
    async fn get_document(&self, id: &str) -> Result<Document>;
    async fn create_document(&self, parent_id: Option<&str>, title: &str, content: Option<&str>) -> Result<Document>;
    async fn update_document(&self, id: &str, title: Option<&str>, content: Option<&str>) -> Result<Document>;
    async fn delete_document(&self, id: &str) -> Result<()>;
    async fn search(&self, query: &str) -> Result<Vec<Document>>;
}

#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    async fn get_database(&self, id: &str) -> Result<Database>;
    async fn query_database(&self, id: &str, filter: &QueryFilter) -> Result<Vec<DatabaseEntry>>;
    async fn create_entry(&self, database_id: &str, properties: HashMap<String, serde_json::Value>) -> Result<DatabaseEntry>;
    async fn update_entry(&self, entry_id: &str, properties: HashMap<String, serde_json::Value>) -> Result<DatabaseEntry>;
    async fn delete_entry(&self, entry_id: &str) -> Result<()>;
}

#[async_trait]
pub trait CalendarProvider: Send + Sync {
    async fn list_calendars(&self) -> Result<Vec<Calendar>>;
    async fn list_events(&self, calendar_id: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<CalendarEvent>>;
    async fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<CalendarEvent>;
    async fn create_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<CalendarEvent>;
    async fn update_event(&self, calendar_id: &str, event_id: &str, event: &CalendarEvent) -> Result<CalendarEvent>;
    async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()>;
}

#[async_trait]
pub trait FileStorageProvider: Send + Sync {
    async fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<File>>;
    async fn get_file(&self, file_id: &str) -> Result<File>;
    async fn download_file(&self, file_id: &str) -> Result<Vec<u8>>;
    async fn upload_file(&self, folder_id: Option<&str>, name: &str, content: &[u8], mime_type: &str) -> Result<File>;
    async fn delete_file(&self, file_id: &str) -> Result<()>;
    async fn create_folder(&self, parent_id: Option<&str>, name: &str) -> Result<Folder>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spreadsheet {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub sheets: Vec<Sheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub id: String,
    pub name: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Formula(String),
    Empty,
}

#[async_trait]
pub trait SpreadsheetProvider: Send + Sync {
    async fn get_spreadsheet(&self, id: &str) -> Result<Spreadsheet>;
    async fn create_spreadsheet(&self, title: &str) -> Result<Spreadsheet>;
    async fn get_values(&self, spreadsheet_id: &str, range: &str) -> Result<Vec<Vec<CellValue>>>;
    async fn update_values(&self, spreadsheet_id: &str, range: &str, values: Vec<Vec<CellValue>>) -> Result<()>;
    async fn append_values(&self, spreadsheet_id: &str, range: &str, values: Vec<Vec<CellValue>>) -> Result<()>;
    async fn clear_values(&self, spreadsheet_id: &str, range: &str) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub labels: Vec<String>,
    pub url: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    None,
    Low,
    Medium,
    High,
    Urgent,
}

#[async_trait]
pub trait TaskProvider: Send + Sync {
    async fn list_tasks(&self, project_id: Option<&str>) -> Result<Vec<Task>>;
    async fn get_task(&self, task_id: &str) -> Result<Task>;
    async fn create_task(&self, task: &Task) -> Result<Task>;
    async fn update_task(&self, task_id: &str, task: &Task) -> Result<Task>;
    async fn delete_task(&self, task_id: &str) -> Result<()>;
}
