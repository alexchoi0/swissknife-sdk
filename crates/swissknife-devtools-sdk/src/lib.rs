mod error;

pub use error::{Error, Result};

#[cfg(feature = "github")]
pub mod github;

#[cfg(feature = "gitlab")]
pub mod gitlab;

#[cfg(feature = "cursor")]
pub mod cursor;

#[cfg(feature = "stagehand")]
pub mod stagehand;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub url: String,
    pub clone_url: String,
    pub default_branch: String,
    pub is_private: bool,
    pub owner: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub stars: Option<u32>,
    pub forks: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub author: String,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: PullRequestState,
    pub author: String,
    pub head_branch: String,
    pub base_branch: String,
    pub is_draft: bool,
    pub mergeable: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
    pub changed_files: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub author_email: Option<String>,
    pub date: DateTime<Utc>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub sha: String,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub encoding: String,
    pub sha: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub path: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub branch: String,
    pub sha: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub total: Option<u64>,
    pub has_more: bool,
}

#[async_trait]
pub trait GitProvider: Send + Sync {
    async fn get_repository(&self, owner: &str, repo: &str) -> Result<Repository>;
    async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>>;
    async fn get_file(&self, owner: &str, repo: &str, path: &str, ref_: Option<&str>) -> Result<FileContent>;
    async fn list_commits(&self, owner: &str, repo: &str, options: &ListOptions) -> Result<ListResult<Commit>>;

    async fn list_issues(&self, owner: &str, repo: &str, options: &ListOptions) -> Result<ListResult<Issue>>;
    async fn get_issue(&self, owner: &str, repo: &str, number: u64) -> Result<Issue>;
    async fn create_issue(&self, owner: &str, repo: &str, title: &str, body: Option<&str>) -> Result<Issue>;
    async fn update_issue(&self, owner: &str, repo: &str, number: u64, title: Option<&str>, body: Option<&str>, state: Option<IssueState>) -> Result<Issue>;

    async fn list_pull_requests(&self, owner: &str, repo: &str, options: &ListOptions) -> Result<ListResult<PullRequest>>;
    async fn get_pull_request(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest>;
    async fn create_pull_request(&self, owner: &str, repo: &str, title: &str, head: &str, base: &str, body: Option<&str>) -> Result<PullRequest>;
    async fn merge_pull_request(&self, owner: &str, repo: &str, number: u64) -> Result<()>;

    async fn list_comments(&self, owner: &str, repo: &str, issue_number: u64) -> Result<Vec<Comment>>;
    async fn create_comment(&self, owner: &str, repo: &str, issue_number: u64, body: &str) -> Result<Comment>;
}

#[async_trait]
pub trait CiCdProvider: Send + Sync {
    async fn list_workflows(&self, owner: &str, repo: &str) -> Result<Vec<Workflow>>;
    async fn list_workflow_runs(&self, owner: &str, repo: &str, workflow_id: &str) -> Result<Vec<WorkflowRun>>;
    async fn trigger_workflow(&self, owner: &str, repo: &str, workflow_id: &str, ref_: &str, inputs: Option<HashMap<String, String>>) -> Result<()>;
    async fn cancel_workflow_run(&self, owner: &str, repo: &str, run_id: &str) -> Result<()>;
    async fn rerun_workflow(&self, owner: &str, repo: &str, run_id: &str) -> Result<()>;
}
