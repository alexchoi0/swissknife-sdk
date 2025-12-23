use crate::error::Result;
use crate::tool::{get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_devtools_sdk::{GitProvider, ListOptions};

pub struct GitHubGetRepoTool;

impl Default for GitHubGetRepoTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitHubGetRepoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "github_get_repo",
            "GitHub Get Repository",
            "Get information about a GitHub repository",
            "devtools",
        )
        .with_param("token", ParameterSchema::string("GitHub personal access token").required().user_only())
        .with_param("owner", ParameterSchema::string("Repository owner").required())
        .with_param("repo", ParameterSchema::string("Repository name").required())
        .with_output("repository", OutputSchema::json("Repository information"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let token = get_required_string_param(&params, "token")?;
        let owner = get_required_string_param(&params, "owner")?;
        let repo = get_required_string_param(&params, "repo")?;

        #[cfg(feature = "github")]
        {
            use swissknife_devtools_sdk::github::GitHubClient;
            let client = GitHubClient::new(&token);
            match client.get_repository(&owner, &repo).await {
                Ok(repository) => Ok(ToolResponse::success(serde_json::json!({
                    "repository": {
                        "id": repository.id,
                        "name": repository.name,
                        "full_name": repository.full_name,
                        "description": repository.description,
                        "url": repository.url,
                        "clone_url": repository.clone_url,
                        "default_branch": repository.default_branch,
                        "is_private": repository.is_private,
                        "language": repository.language,
                        "stars": repository.stars,
                        "forks": repository.forks,
                    }
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to get repository: {}", e))),
            }
        }
        #[cfg(not(feature = "github"))]
        {
            let _ = (token, owner, repo);
            Ok(ToolResponse::error("GitHub feature not enabled"))
        }
    }
}

pub struct GitHubListIssuesTool;

impl Default for GitHubListIssuesTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitHubListIssuesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "github_list_issues",
            "GitHub List Issues",
            "List issues in a GitHub repository",
            "devtools",
        )
        .with_param("token", ParameterSchema::string("GitHub personal access token").required().user_only())
        .with_param("owner", ParameterSchema::string("Repository owner").required())
        .with_param("repo", ParameterSchema::string("Repository name").required())
        .with_param("state", ParameterSchema::string("Issue state: open, closed, all").with_default(serde_json::json!("open")))
        .with_param("per_page", ParameterSchema::integer("Results per page").with_default(serde_json::json!(30)))
        .with_output("issues", OutputSchema::array("List of issues", OutputSchema::json("Issue")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let token = get_required_string_param(&params, "token")?;
        let owner = get_required_string_param(&params, "owner")?;
        let repo = get_required_string_param(&params, "repo")?;
        let state = get_string_param(&params, "state");
        let per_page = get_i64_param(&params, "per_page").map(|v| v as u32);

        #[cfg(feature = "github")]
        {
            use swissknife_devtools_sdk::github::GitHubClient;
            let client = GitHubClient::new(&token);
            let options = ListOptions {
                state,
                per_page,
                ..Default::default()
            };
            match client.list_issues(&owner, &repo, &options).await {
                Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                    "issues": result.items.iter().map(|i| serde_json::json!({
                        "id": i.id,
                        "number": i.number,
                        "title": i.title,
                        "body": i.body,
                        "state": format!("{:?}", i.state),
                        "author": i.author,
                        "labels": i.labels,
                        "created_at": i.created_at.to_rfc3339(),
                    })).collect::<Vec<_>>(),
                    "total": result.total,
                    "has_more": result.has_more,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to list issues: {}", e))),
            }
        }
        #[cfg(not(feature = "github"))]
        {
            let _ = (token, owner, repo, state, per_page);
            Ok(ToolResponse::error("GitHub feature not enabled"))
        }
    }
}

pub struct GitHubCreateIssueTool;

impl Default for GitHubCreateIssueTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitHubCreateIssueTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "github_create_issue",
            "GitHub Create Issue",
            "Create a new issue in a GitHub repository",
            "devtools",
        )
        .with_param("token", ParameterSchema::string("GitHub personal access token").required().user_only())
        .with_param("owner", ParameterSchema::string("Repository owner").required())
        .with_param("repo", ParameterSchema::string("Repository name").required())
        .with_param("title", ParameterSchema::string("Issue title").required())
        .with_param("body", ParameterSchema::string("Issue body/description"))
        .with_output("issue", OutputSchema::json("Created issue"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let token = get_required_string_param(&params, "token")?;
        let owner = get_required_string_param(&params, "owner")?;
        let repo = get_required_string_param(&params, "repo")?;
        let title = get_required_string_param(&params, "title")?;
        let body = get_string_param(&params, "body");

        #[cfg(feature = "github")]
        {
            use swissknife_devtools_sdk::github::GitHubClient;
            let client = GitHubClient::new(&token);
            match client.create_issue(&owner, &repo, &title, body.as_deref()).await {
                Ok(issue) => Ok(ToolResponse::success(serde_json::json!({
                    "issue": {
                        "id": issue.id,
                        "number": issue.number,
                        "title": issue.title,
                        "state": format!("{:?}", issue.state),
                    }
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to create issue: {}", e))),
            }
        }
        #[cfg(not(feature = "github"))]
        {
            let _ = (token, owner, repo, title, body);
            Ok(ToolResponse::error("GitHub feature not enabled"))
        }
    }
}

pub struct GitHubListPRsTool;

impl Default for GitHubListPRsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitHubListPRsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "github_list_pull_requests",
            "GitHub List Pull Requests",
            "List pull requests in a GitHub repository",
            "devtools",
        )
        .with_param("token", ParameterSchema::string("GitHub personal access token").required().user_only())
        .with_param("owner", ParameterSchema::string("Repository owner").required())
        .with_param("repo", ParameterSchema::string("Repository name").required())
        .with_param("state", ParameterSchema::string("PR state: open, closed, all").with_default(serde_json::json!("open")))
        .with_output("pull_requests", OutputSchema::array("List of pull requests", OutputSchema::json("PullRequest")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let token = get_required_string_param(&params, "token")?;
        let owner = get_required_string_param(&params, "owner")?;
        let repo = get_required_string_param(&params, "repo")?;
        let state = get_string_param(&params, "state");

        #[cfg(feature = "github")]
        {
            use swissknife_devtools_sdk::github::GitHubClient;
            let client = GitHubClient::new(&token);
            let options = ListOptions {
                state,
                ..Default::default()
            };
            match client.list_pull_requests(&owner, &repo, &options).await {
                Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                    "pull_requests": result.items.iter().map(|pr| serde_json::json!({
                        "id": pr.id,
                        "number": pr.number,
                        "title": pr.title,
                        "state": format!("{:?}", pr.state),
                        "author": pr.author,
                        "head_branch": pr.head_branch,
                        "base_branch": pr.base_branch,
                        "is_draft": pr.is_draft,
                        "mergeable": pr.mergeable,
                        "additions": pr.additions,
                        "deletions": pr.deletions,
                    })).collect::<Vec<_>>(),
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to list PRs: {}", e))),
            }
        }
        #[cfg(not(feature = "github"))]
        {
            let _ = (token, owner, repo, state);
            Ok(ToolResponse::error("GitHub feature not enabled"))
        }
    }
}

pub struct GitHubCreatePRTool;

impl Default for GitHubCreatePRTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitHubCreatePRTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "github_create_pull_request",
            "GitHub Create Pull Request",
            "Create a new pull request in a GitHub repository",
            "devtools",
        )
        .with_param("token", ParameterSchema::string("GitHub personal access token").required().user_only())
        .with_param("owner", ParameterSchema::string("Repository owner").required())
        .with_param("repo", ParameterSchema::string("Repository name").required())
        .with_param("title", ParameterSchema::string("PR title").required())
        .with_param("head", ParameterSchema::string("Head branch").required())
        .with_param("base", ParameterSchema::string("Base branch").required())
        .with_param("body", ParameterSchema::string("PR description"))
        .with_output("pull_request", OutputSchema::json("Created pull request"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let token = get_required_string_param(&params, "token")?;
        let owner = get_required_string_param(&params, "owner")?;
        let repo = get_required_string_param(&params, "repo")?;
        let title = get_required_string_param(&params, "title")?;
        let head = get_required_string_param(&params, "head")?;
        let base = get_required_string_param(&params, "base")?;
        let body = get_string_param(&params, "body");

        #[cfg(feature = "github")]
        {
            use swissknife_devtools_sdk::github::GitHubClient;
            let client = GitHubClient::new(&token);
            match client.create_pull_request(&owner, &repo, &title, &head, &base, body.as_deref()).await {
                Ok(pr) => Ok(ToolResponse::success(serde_json::json!({
                    "pull_request": {
                        "id": pr.id,
                        "number": pr.number,
                        "title": pr.title,
                        "state": format!("{:?}", pr.state),
                    }
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to create PR: {}", e))),
            }
        }
        #[cfg(not(feature = "github"))]
        {
            let _ = (token, owner, repo, title, head, base, body);
            Ok(ToolResponse::error("GitHub feature not enabled"))
        }
    }
}

pub struct GitHubGetFileTool;

impl Default for GitHubGetFileTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitHubGetFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "github_get_file",
            "GitHub Get File",
            "Get file contents from a GitHub repository",
            "devtools",
        )
        .with_param("token", ParameterSchema::string("GitHub personal access token").required().user_only())
        .with_param("owner", ParameterSchema::string("Repository owner").required())
        .with_param("repo", ParameterSchema::string("Repository name").required())
        .with_param("path", ParameterSchema::string("File path").required())
        .with_param("ref", ParameterSchema::string("Branch, tag, or commit SHA"))
        .with_output("content", OutputSchema::string("File content"))
        .with_output("sha", OutputSchema::string("File SHA"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let token = get_required_string_param(&params, "token")?;
        let owner = get_required_string_param(&params, "owner")?;
        let repo = get_required_string_param(&params, "repo")?;
        let path = get_required_string_param(&params, "path")?;
        let ref_ = get_string_param(&params, "ref");

        #[cfg(feature = "github")]
        {
            use swissknife_devtools_sdk::github::GitHubClient;
            let client = GitHubClient::new(&token);
            match client.get_file(&owner, &repo, &path, ref_.as_deref()).await {
                Ok(file) => Ok(ToolResponse::success(serde_json::json!({
                    "content": file.content,
                    "sha": file.sha,
                    "path": file.path,
                    "size": file.size,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Failed to get file: {}", e))),
            }
        }
        #[cfg(not(feature = "github"))]
        {
            let _ = (token, owner, repo, path, ref_);
            Ok(ToolResponse::error("GitHub feature not enabled"))
        }
    }
}
