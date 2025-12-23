use crate::error::Result;
use crate::tool::{get_i64_param, get_required_string_param, get_string_param, get_array_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_pm_sdk::{ProjectManagementProvider, IssueFilter, ListOptions, CreateIssue, UpdateIssue, Priority, IssueType};

pub struct ListProjectsTool;

impl Default for ListProjectsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListProjectsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "pm_list_projects",
            "List Projects",
            "List projects from a project management tool (Linear, Jira, Asana, Trello, ClickUp)",
            "pm",
        )
        .with_param("api_key", ParameterSchema::string("API key or token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: linear, jira, asana, trello, clickup").required())
        .with_param("domain", ParameterSchema::string("Domain/workspace (for Jira, Asana)"))
        .with_output("projects", OutputSchema::array("List of projects", OutputSchema::json("Project")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let _domain = get_string_param(&params, "domain");

        let options = ListOptions::default();

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "linear")]
            "linear" => {
                use swissknife_pm_sdk::linear::LinearClient;
                let client = LinearClient::new(&api_key);
                client.list_projects(&options).await
            }
            #[cfg(feature = "jira")]
            "jira" => {
                use swissknife_pm_sdk::jira::JiraClient;
                let domain = _domain.ok_or_else(|| crate::Error::MissingParameter("domain".into()))?;
                let client = JiraClient::new(&domain, &api_key);
                client.list_projects(&options).await
            }
            #[cfg(feature = "asana")]
            "asana" => {
                use swissknife_pm_sdk::asana::AsanaClient;
                let client = AsanaClient::new(&api_key);
                client.list_projects(&options).await
            }
            #[cfg(feature = "trello")]
            "trello" => {
                use swissknife_pm_sdk::trello::TrelloClient;
                let client = TrelloClient::new(&api_key);
                client.list_projects(&options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported PM provider: {}", provider)));
            }
        };

        match result {
            Ok(projects) => Ok(ToolResponse::success(serde_json::json!({
                "projects": projects.items.iter().map(|p| serde_json::json!({
                    "id": p.id,
                    "name": p.name,
                    "key": p.key,
                    "description": p.description,
                    "status": p.status.map(|s| format!("{:?}", s)),
                })).collect::<Vec<_>>(),
                "has_more": projects.has_more,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list projects: {}", e))),
        }
    }
}

pub struct ListIssuesTool;

impl Default for ListIssuesTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListIssuesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "pm_list_issues",
            "List Issues",
            "List issues/tasks from a project management tool",
            "pm",
        )
        .with_param("api_key", ParameterSchema::string("API key or token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: linear, jira, asana, trello, clickup").required())
        .with_param("project_id", ParameterSchema::string("Project ID").required())
        .with_param("domain", ParameterSchema::string("Domain/workspace (for Jira)"))
        .with_param("status", ParameterSchema::string("Filter by status"))
        .with_param("assignee_id", ParameterSchema::string("Filter by assignee"))
        .with_output("issues", OutputSchema::array("List of issues/tasks", OutputSchema::json("Issue")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let project_id = get_required_string_param(&params, "project_id")?;
        let _domain = get_string_param(&params, "domain");
        let status = get_string_param(&params, "status");
        let assignee_id = get_string_param(&params, "assignee_id");

        let filter = IssueFilter {
            project_id: Some(project_id.clone()),
            status,
            assignee_id,
            ..Default::default()
        };
        let options = ListOptions::default();

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "linear")]
            "linear" => {
                use swissknife_pm_sdk::linear::LinearClient;
                let client = LinearClient::new(&api_key);
                client.list_issues(&project_id, &filter, &options).await
            }
            #[cfg(feature = "jira")]
            "jira" => {
                use swissknife_pm_sdk::jira::JiraClient;
                let domain = _domain.ok_or_else(|| crate::Error::MissingParameter("domain".into()))?;
                let client = JiraClient::new(&domain, &api_key);
                client.list_issues(&project_id, &filter, &options).await
            }
            #[cfg(feature = "asana")]
            "asana" => {
                use swissknife_pm_sdk::asana::AsanaClient;
                let client = AsanaClient::new(&api_key);
                client.list_issues(&project_id, &filter, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported PM provider: {}", provider)));
            }
        };

        match result {
            Ok(issues) => Ok(ToolResponse::success(serde_json::json!({
                "issues": issues.items.iter().map(|i| serde_json::json!({
                    "id": i.id,
                    "key": i.key,
                    "title": i.title,
                    "description": i.description,
                    "status": i.status,
                    "priority": i.priority.map(|p| format!("{:?}", p)),
                    "issue_type": format!("{:?}", i.issue_type),
                    "assignee_id": i.assignee_id,
                    "labels": i.labels,
                    "due_date": i.due_date.map(|d| d.to_string()),
                })).collect::<Vec<_>>(),
                "has_more": issues.has_more,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list issues: {}", e))),
        }
    }
}

pub struct CreateIssueTool;

impl Default for CreateIssueTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CreateIssueTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "pm_create_issue",
            "Create Issue",
            "Create a new issue/task in a project management tool",
            "pm",
        )
        .with_param("api_key", ParameterSchema::string("API key or token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: linear, jira, asana, trello, clickup").required())
        .with_param("project_id", ParameterSchema::string("Project ID").required())
        .with_param("domain", ParameterSchema::string("Domain/workspace (for Jira)"))
        .with_param("title", ParameterSchema::string("Issue title").required())
        .with_param("description", ParameterSchema::string("Issue description"))
        .with_param("priority", ParameterSchema::string("Priority: urgent, high, medium, low, none"))
        .with_param("issue_type", ParameterSchema::string("Type: task, bug, story, epic, feature"))
        .with_param("assignee_id", ParameterSchema::string("Assignee ID"))
        .with_param("labels", ParameterSchema::array("Labels/tags", ParameterSchema::string("Label")))
        .with_output("issue", OutputSchema::json("Created issue"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let project_id = get_required_string_param(&params, "project_id")?;
        let _domain = get_string_param(&params, "domain");
        let title = get_required_string_param(&params, "title")?;
        let description = get_string_param(&params, "description");
        let priority_str = get_string_param(&params, "priority");
        let issue_type_str = get_string_param(&params, "issue_type");
        let assignee_id = get_string_param(&params, "assignee_id");
        let labels = get_array_param(&params, "labels")
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let priority = priority_str.map(|p| match p.to_lowercase().as_str() {
            "urgent" => Priority::Urgent,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::None,
        });

        let issue_type = issue_type_str.map(|t| match t.to_lowercase().as_str() {
            "bug" => IssueType::Bug,
            "story" => IssueType::Story,
            "epic" => IssueType::Epic,
            "feature" => IssueType::Feature,
            "improvement" => IssueType::Improvement,
            "subtask" => IssueType::Subtask,
            _ => IssueType::Task,
        });

        let create_issue = CreateIssue {
            title,
            description,
            priority,
            issue_type,
            assignee_id,
            labels,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "linear")]
            "linear" => {
                use swissknife_pm_sdk::linear::LinearClient;
                let client = LinearClient::new(&api_key);
                client.create_issue(&project_id, &create_issue).await
            }
            #[cfg(feature = "jira")]
            "jira" => {
                use swissknife_pm_sdk::jira::JiraClient;
                let domain = _domain.ok_or_else(|| crate::Error::MissingParameter("domain".into()))?;
                let client = JiraClient::new(&domain, &api_key);
                client.create_issue(&project_id, &create_issue).await
            }
            #[cfg(feature = "asana")]
            "asana" => {
                use swissknife_pm_sdk::asana::AsanaClient;
                let client = AsanaClient::new(&api_key);
                client.create_issue(&project_id, &create_issue).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported PM provider: {}", provider)));
            }
        };

        match result {
            Ok(issue) => Ok(ToolResponse::success(serde_json::json!({
                "issue": {
                    "id": issue.id,
                    "key": issue.key,
                    "title": issue.title,
                    "status": issue.status,
                }
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to create issue: {}", e))),
        }
    }
}

pub struct UpdateIssueTool;

impl Default for UpdateIssueTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for UpdateIssueTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "pm_update_issue",
            "Update Issue",
            "Update an existing issue/task",
            "pm",
        )
        .with_param("api_key", ParameterSchema::string("API key or token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: linear, jira, asana, trello, clickup").required())
        .with_param("issue_id", ParameterSchema::string("Issue ID").required())
        .with_param("domain", ParameterSchema::string("Domain/workspace (for Jira)"))
        .with_param("title", ParameterSchema::string("New title"))
        .with_param("description", ParameterSchema::string("New description"))
        .with_param("status", ParameterSchema::string("New status"))
        .with_param("priority", ParameterSchema::string("New priority"))
        .with_param("assignee_id", ParameterSchema::string("New assignee ID"))
        .with_output("issue", OutputSchema::json("Updated issue"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let issue_id = get_required_string_param(&params, "issue_id")?;
        let _domain = get_string_param(&params, "domain");
        let title = get_string_param(&params, "title");
        let description = get_string_param(&params, "description");
        let status = get_string_param(&params, "status");
        let priority_str = get_string_param(&params, "priority");
        let assignee_id = get_string_param(&params, "assignee_id");

        let priority = priority_str.map(|p| match p.to_lowercase().as_str() {
            "urgent" => Priority::Urgent,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::None,
        });

        let update = UpdateIssue {
            title,
            description,
            status,
            priority,
            assignee_id,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "linear")]
            "linear" => {
                use swissknife_pm_sdk::linear::LinearClient;
                let client = LinearClient::new(&api_key);
                client.update_issue(&issue_id, &update).await
            }
            #[cfg(feature = "jira")]
            "jira" => {
                use swissknife_pm_sdk::jira::JiraClient;
                let domain = _domain.ok_or_else(|| crate::Error::MissingParameter("domain".into()))?;
                let client = JiraClient::new(&domain, &api_key);
                client.update_issue(&issue_id, &update).await
            }
            #[cfg(feature = "asana")]
            "asana" => {
                use swissknife_pm_sdk::asana::AsanaClient;
                let client = AsanaClient::new(&api_key);
                client.update_issue(&issue_id, &update).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported PM provider: {}", provider)));
            }
        };

        match result {
            Ok(issue) => Ok(ToolResponse::success(serde_json::json!({
                "issue": {
                    "id": issue.id,
                    "key": issue.key,
                    "title": issue.title,
                    "status": issue.status,
                }
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to update issue: {}", e))),
        }
    }
}

pub struct SearchIssuesTool;

impl Default for SearchIssuesTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SearchIssuesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "pm_search_issues",
            "Search Issues",
            "Search for issues across projects",
            "pm",
        )
        .with_param("api_key", ParameterSchema::string("API key or token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: linear, jira, asana").required())
        .with_param("query", ParameterSchema::string("Search query").required())
        .with_param("domain", ParameterSchema::string("Domain/workspace (for Jira)"))
        .with_output("issues", OutputSchema::array("Matching issues", OutputSchema::json("Issue")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let query = get_required_string_param(&params, "query")?;
        let _domain = get_string_param(&params, "domain");

        let options = ListOptions::default();

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "linear")]
            "linear" => {
                use swissknife_pm_sdk::linear::LinearClient;
                let client = LinearClient::new(&api_key);
                client.search_issues(&query, &options).await
            }
            #[cfg(feature = "jira")]
            "jira" => {
                use swissknife_pm_sdk::jira::JiraClient;
                let domain = _domain.ok_or_else(|| crate::Error::MissingParameter("domain".into()))?;
                let client = JiraClient::new(&domain, &api_key);
                client.search_issues(&query, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported PM provider: {}", provider)));
            }
        };

        match result {
            Ok(issues) => Ok(ToolResponse::success(serde_json::json!({
                "issues": issues.items.iter().map(|i| serde_json::json!({
                    "id": i.id,
                    "key": i.key,
                    "title": i.title,
                    "status": i.status,
                    "project_id": i.project_id,
                })).collect::<Vec<_>>(),
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Search failed: {}", e))),
        }
    }
}
