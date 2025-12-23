mod error;

pub use error::{Error, Result};

#[cfg(feature = "linear")]
pub mod linear;

#[cfg(feature = "jira")]
pub mod jira;

#[cfg(feature = "asana")]
pub mod asana;

#[cfg(feature = "trello")]
pub mod trello;

#[cfg(feature = "clickup")]
pub mod clickup;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub key: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub lead_id: Option<String>,
    pub status: Option<ProjectStatus>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Active,
    Completed,
    Archived,
    OnHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub key: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub issue_type: IssueType,
    pub status: String,
    pub priority: Option<Priority>,
    pub assignee_id: Option<String>,
    pub reporter_id: Option<String>,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub labels: Vec<String>,
    pub estimate: Option<f64>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Task,
    Bug,
    Story,
    Epic,
    Feature,
    Improvement,
    Subtask,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub id: String,
    pub name: String,
    pub goal: Option<String>,
    pub status: SprintStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub project_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SprintStatus {
    Planned,
    Active,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    pub state_type: StateType,
    pub color: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateType {
    Backlog,
    Unstarted,
    Started,
    Completed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub author_id: String,
    pub issue_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct IssueFilter {
    pub project_id: Option<String>,
    pub assignee_id: Option<String>,
    pub status: Option<String>,
    pub labels: Vec<String>,
    pub issue_type: Option<IssueType>,
    pub priority: Option<Priority>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub total: Option<u64>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateIssue {
    pub title: String,
    pub description: Option<String>,
    pub issue_type: Option<IssueType>,
    pub priority: Option<Priority>,
    pub assignee_id: Option<String>,
    pub labels: Vec<String>,
    pub estimate: Option<f64>,
    pub due_date: Option<NaiveDate>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateIssue {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<Priority>,
    pub assignee_id: Option<String>,
    pub labels: Option<Vec<String>>,
    pub estimate: Option<f64>,
    pub due_date: Option<NaiveDate>,
}

#[async_trait]
pub trait ProjectManagementProvider: Send + Sync {
    async fn list_projects(&self, options: &ListOptions) -> Result<ListResult<Project>>;
    async fn get_project(&self, id: &str) -> Result<Project>;
    async fn create_project(&self, name: &str, description: Option<&str>) -> Result<Project>;
    async fn update_project(&self, id: &str, name: Option<&str>, description: Option<&str>) -> Result<Project>;
    async fn archive_project(&self, id: &str) -> Result<()>;

    async fn list_issues(&self, project_id: &str, filter: &IssueFilter, options: &ListOptions) -> Result<ListResult<Issue>>;
    async fn get_issue(&self, id: &str) -> Result<Issue>;
    async fn create_issue(&self, project_id: &str, issue: &CreateIssue) -> Result<Issue>;
    async fn update_issue(&self, id: &str, update: &UpdateIssue) -> Result<Issue>;
    async fn delete_issue(&self, id: &str) -> Result<()>;
    async fn search_issues(&self, query: &str, options: &ListOptions) -> Result<ListResult<Issue>>;

    async fn list_labels(&self, project_id: &str) -> Result<Vec<Label>>;
    async fn create_label(&self, project_id: &str, name: &str, color: Option<&str>) -> Result<Label>;

    async fn list_workflow_states(&self, project_id: &str) -> Result<Vec<WorkflowState>>;

    async fn list_comments(&self, issue_id: &str) -> Result<Vec<Comment>>;
    async fn create_comment(&self, issue_id: &str, body: &str) -> Result<Comment>;
    async fn update_comment(&self, comment_id: &str, body: &str) -> Result<Comment>;
    async fn delete_comment(&self, comment_id: &str) -> Result<()>;

    async fn list_users(&self) -> Result<Vec<User>>;
    async fn get_current_user(&self) -> Result<User>;
}

#[async_trait]
pub trait SprintProvider: Send + Sync {
    async fn list_sprints(&self, project_id: &str) -> Result<Vec<Sprint>>;
    async fn get_active_sprint(&self, project_id: &str) -> Result<Option<Sprint>>;
    async fn create_sprint(&self, project_id: &str, name: &str, goal: Option<&str>, start: Option<NaiveDate>, end: Option<NaiveDate>) -> Result<Sprint>;
    async fn start_sprint(&self, sprint_id: &str) -> Result<Sprint>;
    async fn complete_sprint(&self, sprint_id: &str) -> Result<Sprint>;
}
