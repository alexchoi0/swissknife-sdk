use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "devtools")]
use swissknife_devtools_sdk as devtools;

#[derive(Clone)]
pub struct DevtoolsTools {
    #[cfg(feature = "github")]
    pub github: Option<devtools::github::GitHubClient>,
    #[cfg(feature = "gitlab")]
    pub gitlab: Option<devtools::gitlab::GitLabClient>,
}

impl DevtoolsTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "github")]
            github: None,
            #[cfg(feature = "gitlab")]
            gitlab: None,
        }
    }

    #[cfg(feature = "github")]
    pub fn with_github(mut self, client: devtools::github::GitHubClient) -> Self {
        self.github = Some(client);
        self
    }

    #[cfg(feature = "gitlab")]
    pub fn with_gitlab(mut self, client: devtools::gitlab::GitLabClient) -> Self {
        self.gitlab = Some(client);
        self
    }
}

impl Default for DevtoolsTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubGetRepoRequest {
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubListIssuesRequest {
    pub owner: String,
    pub repo: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubCreateIssueRequest {
    pub owner: String,
    pub repo: String,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubListPullRequestsRequest {
    pub owner: String,
    pub repo: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubCreatePullRequestRequest {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub head: String,
    pub base: String,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubGetFileRequest {
    pub owner: String,
    pub repo: String,
    pub path: String,
    #[serde(default)]
    pub ref_name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubSearchCodeRequest {
    pub query: String,
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitHubSearchReposRequest {
    pub query: String,
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitLabGetProjectRequest {
    pub project_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitLabListIssuesRequest {
    pub project_id: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitLabCreateIssueRequest {
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub labels: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitLabListMergeRequestsRequest {
    pub project_id: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[tool_box]
impl DevtoolsTools {
    #[cfg(feature = "github")]
    #[rmcp::tool(description = "Get information about a GitHub repository")]
    pub async fn github_get_repo(
        &self,
        #[rmcp::tool(aggr)] req: GitHubGetRepoRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let repo = client.get_repository(&req.owner, &req.repo).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&repo).map_err(|e| e.to_string())
    }

    #[cfg(feature = "github")]
    #[rmcp::tool(description = "List issues in a GitHub repository")]
    pub async fn github_list_issues(
        &self,
        #[rmcp::tool(aggr)] req: GitHubListIssuesRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let issues = client.list_issues(
            &req.owner,
            &req.repo,
            req.state.as_deref(),
            req.per_page,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issues).map_err(|e| e.to_string())
    }

    #[cfg(feature = "github")]
    #[rmcp::tool(description = "Create a new issue in a GitHub repository")]
    pub async fn github_create_issue(
        &self,
        #[rmcp::tool(aggr)] req: GitHubCreateIssueRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let issue = client.create_issue(
            &req.owner,
            &req.repo,
            &req.title,
            req.body.as_deref(),
            req.labels.as_deref(),
            req.assignees.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issue).map_err(|e| e.to_string())
    }

    #[cfg(feature = "github")]
    #[rmcp::tool(description = "List pull requests in a GitHub repository")]
    pub async fn github_list_pull_requests(
        &self,
        #[rmcp::tool(aggr)] req: GitHubListPullRequestsRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let prs = client.list_pull_requests(
            &req.owner,
            &req.repo,
            req.state.as_deref(),
            req.per_page,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&prs).map_err(|e| e.to_string())
    }

    #[cfg(feature = "github")]
    #[rmcp::tool(description = "Create a new pull request in a GitHub repository")]
    pub async fn github_create_pull_request(
        &self,
        #[rmcp::tool(aggr)] req: GitHubCreatePullRequestRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let pr = client.create_pull_request(
            &req.owner,
            &req.repo,
            &req.title,
            &req.head,
            &req.base,
            req.body.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&pr).map_err(|e| e.to_string())
    }

    #[cfg(feature = "github")]
    #[rmcp::tool(description = "Get a file from a GitHub repository")]
    pub async fn github_get_file(
        &self,
        #[rmcp::tool(aggr)] req: GitHubGetFileRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let content = client.get_file_content(
            &req.owner,
            &req.repo,
            &req.path,
            req.ref_name.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&content).map_err(|e| e.to_string())
    }

    #[cfg(feature = "github")]
    #[rmcp::tool(description = "Search for code on GitHub")]
    pub async fn github_search_code(
        &self,
        #[rmcp::tool(aggr)] req: GitHubSearchCodeRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let results = client.search_code(&req.query, req.per_page).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
    }

    #[cfg(feature = "github")]
    #[rmcp::tool(description = "Search for repositories on GitHub")]
    pub async fn github_search_repos(
        &self,
        #[rmcp::tool(aggr)] req: GitHubSearchReposRequest,
    ) -> Result<String, String> {
        let client = self.github.as_ref()
            .ok_or_else(|| "GitHub client not configured".to_string())?;

        let results = client.search_repositories(&req.query, req.per_page).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gitlab")]
    #[rmcp::tool(description = "Get information about a GitLab project")]
    pub async fn gitlab_get_project(
        &self,
        #[rmcp::tool(aggr)] req: GitLabGetProjectRequest,
    ) -> Result<String, String> {
        let client = self.gitlab.as_ref()
            .ok_or_else(|| "GitLab client not configured".to_string())?;

        let project = client.get_project(&req.project_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&project).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gitlab")]
    #[rmcp::tool(description = "List issues in a GitLab project")]
    pub async fn gitlab_list_issues(
        &self,
        #[rmcp::tool(aggr)] req: GitLabListIssuesRequest,
    ) -> Result<String, String> {
        let client = self.gitlab.as_ref()
            .ok_or_else(|| "GitLab client not configured".to_string())?;

        let issues = client.list_issues(
            &req.project_id,
            req.state.as_deref(),
            req.per_page,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issues).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gitlab")]
    #[rmcp::tool(description = "Create a new issue in a GitLab project")]
    pub async fn gitlab_create_issue(
        &self,
        #[rmcp::tool(aggr)] req: GitLabCreateIssueRequest,
    ) -> Result<String, String> {
        let client = self.gitlab.as_ref()
            .ok_or_else(|| "GitLab client not configured".to_string())?;

        let issue = client.create_issue(
            &req.project_id,
            &req.title,
            req.description.as_deref(),
            req.labels.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issue).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gitlab")]
    #[rmcp::tool(description = "List merge requests in a GitLab project")]
    pub async fn gitlab_list_merge_requests(
        &self,
        #[rmcp::tool(aggr)] req: GitLabListMergeRequestsRequest,
    ) -> Result<String, String> {
        let client = self.gitlab.as_ref()
            .ok_or_else(|| "GitLab client not configured".to_string())?;

        let mrs = client.list_merge_requests(
            &req.project_id,
            req.state.as_deref(),
            req.per_page,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&mrs).map_err(|e| e.to_string())
    }
}
