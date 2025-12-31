use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "pm")]
use swissknife_pm_sdk as pm;

#[derive(Clone)]
pub struct PmTools {
    #[cfg(feature = "linear")]
    pub linear: Option<pm::linear::LinearClient>,
    #[cfg(feature = "jira")]
    pub jira: Option<pm::jira::JiraClient>,
    #[cfg(feature = "asana")]
    pub asana: Option<pm::asana::AsanaClient>,
    #[cfg(feature = "trello")]
    pub trello: Option<pm::trello::TrelloClient>,
    #[cfg(feature = "clickup")]
    pub clickup: Option<pm::clickup::ClickUpClient>,
}

impl PmTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "linear")]
            linear: None,
            #[cfg(feature = "jira")]
            jira: None,
            #[cfg(feature = "asana")]
            asana: None,
            #[cfg(feature = "trello")]
            trello: None,
            #[cfg(feature = "clickup")]
            clickup: None,
        }
    }

    #[cfg(feature = "linear")]
    pub fn with_linear(mut self, client: pm::linear::LinearClient) -> Self {
        self.linear = Some(client);
        self
    }

    #[cfg(feature = "jira")]
    pub fn with_jira(mut self, client: pm::jira::JiraClient) -> Self {
        self.jira = Some(client);
        self
    }

    #[cfg(feature = "asana")]
    pub fn with_asana(mut self, client: pm::asana::AsanaClient) -> Self {
        self.asana = Some(client);
        self
    }

    #[cfg(feature = "trello")]
    pub fn with_trello(mut self, client: pm::trello::TrelloClient) -> Self {
        self.trello = Some(client);
        self
    }

    #[cfg(feature = "clickup")]
    pub fn with_clickup(mut self, client: pm::clickup::ClickUpClient) -> Self {
        self.clickup = Some(client);
        self
    }
}

impl Default for PmTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinearCreateIssueRequest {
    pub title: String,
    pub team_id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub assignee_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinearGetIssueRequest {
    pub issue_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinearSearchIssuesRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JiraCreateIssueRequest {
    pub project_key: String,
    pub summary: String,
    pub issue_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JiraGetIssueRequest {
    pub issue_key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JiraSearchRequest {
    pub jql: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JiraTransitionIssueRequest {
    pub issue_key: String,
    pub transition_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AsanaCreateTaskRequest {
    pub project_id: String,
    pub name: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub due_on: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AsanaGetTaskRequest {
    pub task_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AsanaListTasksRequest {
    pub project_id: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrelloCreateCardRequest {
    pub list_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub due: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrelloGetCardRequest {
    pub card_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrelloListCardsRequest {
    pub list_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickUpCreateTaskRequest {
    pub list_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub due_date: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickUpGetTaskRequest {
    pub task_id: String,
}

#[tool_router]
impl PmTools {
    #[cfg(feature = "linear")]
    #[rmcp::tool(description = "Create a new issue in Linear")]
    pub async fn linear_create_issue(
        &self,
        #[rmcp::tool(aggr)] req: LinearCreateIssueRequest,
    ) -> Result<String, String> {
        let client = self.linear.as_ref()
            .ok_or_else(|| "Linear client not configured".to_string())?;

        let issue = client.create_issue(
            &req.title,
            &req.team_id,
            req.description.as_deref(),
            req.priority,
            req.assignee_id.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issue).map_err(|e| e.to_string())
    }

    #[cfg(feature = "linear")]
    #[rmcp::tool(description = "Get a Linear issue by ID")]
    pub async fn linear_get_issue(
        &self,
        #[rmcp::tool(aggr)] req: LinearGetIssueRequest,
    ) -> Result<String, String> {
        let client = self.linear.as_ref()
            .ok_or_else(|| "Linear client not configured".to_string())?;

        let issue = client.get_issue(&req.issue_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issue).map_err(|e| e.to_string())
    }

    #[cfg(feature = "linear")]
    #[rmcp::tool(description = "Search for issues in Linear")]
    pub async fn linear_search_issues(
        &self,
        #[rmcp::tool(aggr)] req: LinearSearchIssuesRequest,
    ) -> Result<String, String> {
        let client = self.linear.as_ref()
            .ok_or_else(|| "Linear client not configured".to_string())?;

        let issues = client.search_issues(&req.query, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issues).map_err(|e| e.to_string())
    }

    #[cfg(feature = "jira")]
    #[rmcp::tool(description = "Create a new issue in Jira")]
    pub async fn jira_create_issue(
        &self,
        #[rmcp::tool(aggr)] req: JiraCreateIssueRequest,
    ) -> Result<String, String> {
        let client = self.jira.as_ref()
            .ok_or_else(|| "Jira client not configured".to_string())?;

        let issue = client.create_issue(
            &req.project_key,
            &req.summary,
            &req.issue_type,
            req.description.as_deref(),
            req.assignee.as_deref(),
            req.priority.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issue).map_err(|e| e.to_string())
    }

    #[cfg(feature = "jira")]
    #[rmcp::tool(description = "Get a Jira issue by key")]
    pub async fn jira_get_issue(
        &self,
        #[rmcp::tool(aggr)] req: JiraGetIssueRequest,
    ) -> Result<String, String> {
        let client = self.jira.as_ref()
            .ok_or_else(|| "Jira client not configured".to_string())?;

        let issue = client.get_issue(&req.issue_key).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issue).map_err(|e| e.to_string())
    }

    #[cfg(feature = "jira")]
    #[rmcp::tool(description = "Search for issues in Jira using JQL")]
    pub async fn jira_search(
        &self,
        #[rmcp::tool(aggr)] req: JiraSearchRequest,
    ) -> Result<String, String> {
        let client = self.jira.as_ref()
            .ok_or_else(|| "Jira client not configured".to_string())?;

        let results = client.search(&req.jql, req.max_results).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
    }

    #[cfg(feature = "jira")]
    #[rmcp::tool(description = "Transition a Jira issue to a new status")]
    pub async fn jira_transition_issue(
        &self,
        #[rmcp::tool(aggr)] req: JiraTransitionIssueRequest,
    ) -> Result<String, String> {
        let client = self.jira.as_ref()
            .ok_or_else(|| "Jira client not configured".to_string())?;

        client.transition_issue(&req.issue_key, &req.transition_id).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Issue {} transitioned successfully", req.issue_key))
    }

    #[cfg(feature = "asana")]
    #[rmcp::tool(description = "Create a new task in Asana")]
    pub async fn asana_create_task(
        &self,
        #[rmcp::tool(aggr)] req: AsanaCreateTaskRequest,
    ) -> Result<String, String> {
        let client = self.asana.as_ref()
            .ok_or_else(|| "Asana client not configured".to_string())?;

        let task = client.create_task(
            &req.project_id,
            &req.name,
            req.notes.as_deref(),
            req.assignee.as_deref(),
            req.due_on.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&task).map_err(|e| e.to_string())
    }

    #[cfg(feature = "asana")]
    #[rmcp::tool(description = "Get an Asana task by ID")]
    pub async fn asana_get_task(
        &self,
        #[rmcp::tool(aggr)] req: AsanaGetTaskRequest,
    ) -> Result<String, String> {
        let client = self.asana.as_ref()
            .ok_or_else(|| "Asana client not configured".to_string())?;

        let task = client.get_task(&req.task_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&task).map_err(|e| e.to_string())
    }

    #[cfg(feature = "asana")]
    #[rmcp::tool(description = "List tasks in an Asana project")]
    pub async fn asana_list_tasks(
        &self,
        #[rmcp::tool(aggr)] req: AsanaListTasksRequest,
    ) -> Result<String, String> {
        let client = self.asana.as_ref()
            .ok_or_else(|| "Asana client not configured".to_string())?;

        let tasks = client.list_tasks(&req.project_id, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&tasks).map_err(|e| e.to_string())
    }

    #[cfg(feature = "trello")]
    #[rmcp::tool(description = "Create a new card in Trello")]
    pub async fn trello_create_card(
        &self,
        #[rmcp::tool(aggr)] req: TrelloCreateCardRequest,
    ) -> Result<String, String> {
        let client = self.trello.as_ref()
            .ok_or_else(|| "Trello client not configured".to_string())?;

        let card = client.create_card(
            &req.list_id,
            &req.name,
            req.description.as_deref(),
            req.due.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&card).map_err(|e| e.to_string())
    }

    #[cfg(feature = "trello")]
    #[rmcp::tool(description = "Get a Trello card by ID")]
    pub async fn trello_get_card(
        &self,
        #[rmcp::tool(aggr)] req: TrelloGetCardRequest,
    ) -> Result<String, String> {
        let client = self.trello.as_ref()
            .ok_or_else(|| "Trello client not configured".to_string())?;

        let card = client.get_card(&req.card_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&card).map_err(|e| e.to_string())
    }

    #[cfg(feature = "trello")]
    #[rmcp::tool(description = "List cards in a Trello list")]
    pub async fn trello_list_cards(
        &self,
        #[rmcp::tool(aggr)] req: TrelloListCardsRequest,
    ) -> Result<String, String> {
        let client = self.trello.as_ref()
            .ok_or_else(|| "Trello client not configured".to_string())?;

        let cards = client.list_cards(&req.list_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&cards).map_err(|e| e.to_string())
    }

    #[cfg(feature = "clickup")]
    #[rmcp::tool(description = "Create a new task in ClickUp")]
    pub async fn clickup_create_task(
        &self,
        #[rmcp::tool(aggr)] req: ClickUpCreateTaskRequest,
    ) -> Result<String, String> {
        let client = self.clickup.as_ref()
            .ok_or_else(|| "ClickUp client not configured".to_string())?;

        let task = client.create_task(
            &req.list_id,
            &req.name,
            req.description.as_deref(),
            req.priority,
            req.due_date,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&task).map_err(|e| e.to_string())
    }

    #[cfg(feature = "clickup")]
    #[rmcp::tool(description = "Get a ClickUp task by ID")]
    pub async fn clickup_get_task(
        &self,
        #[rmcp::tool(aggr)] req: ClickUpGetTaskRequest,
    ) -> Result<String, String> {
        let client = self.clickup.as_ref()
            .ok_or_else(|| "ClickUp client not configured".to_string())?;

        let task = client.get_task(&req.task_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&task).map_err(|e| e.to_string())
    }
}
