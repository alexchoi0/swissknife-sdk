use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "research")]
use swissknife_research_sdk as research;

#[derive(Clone)]
pub struct ResearchTools {
    #[cfg(feature = "arxiv")]
    pub arxiv: Option<research::arxiv::ArxivClient>,
}

impl ResearchTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "arxiv")]
            arxiv: None,
        }
    }

    #[cfg(feature = "arxiv")]
    pub fn with_arxiv(mut self, client: research::arxiv::ArxivClient) -> Self {
        self.arxiv = Some(client);
        self
    }
}

impl Default for ResearchTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ArxivSearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ArxivGetPaperRequest {
    pub paper_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ArxivSearchByAuthorRequest {
    pub author: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ArxivSearchByCategoryRequest {
    pub category: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[tool_router]
impl ResearchTools {
    #[cfg(feature = "arxiv")]
    #[rmcp::tool(description = "Search for papers on arXiv")]
    pub async fn arxiv_search(
        &self,
        #[rmcp::tool(aggr)] req: ArxivSearchRequest,
    ) -> Result<String, String> {
        let client = self.arxiv.as_ref()
            .ok_or_else(|| "arXiv client not configured".to_string())?;

        let papers = client.search(
            &req.query,
            req.max_results,
            req.sort_by.as_deref(),
            req.sort_order.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&papers).map_err(|e| e.to_string())
    }

    #[cfg(feature = "arxiv")]
    #[rmcp::tool(description = "Get a specific paper from arXiv by ID")]
    pub async fn arxiv_get_paper(
        &self,
        #[rmcp::tool(aggr)] req: ArxivGetPaperRequest,
    ) -> Result<String, String> {
        let client = self.arxiv.as_ref()
            .ok_or_else(|| "arXiv client not configured".to_string())?;

        let paper = client.get_paper(&req.paper_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&paper).map_err(|e| e.to_string())
    }

    #[cfg(feature = "arxiv")]
    #[rmcp::tool(description = "Search for papers by author on arXiv")]
    pub async fn arxiv_search_by_author(
        &self,
        #[rmcp::tool(aggr)] req: ArxivSearchByAuthorRequest,
    ) -> Result<String, String> {
        let client = self.arxiv.as_ref()
            .ok_or_else(|| "arXiv client not configured".to_string())?;

        let papers = client.search_by_author(&req.author, req.max_results).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&papers).map_err(|e| e.to_string())
    }

    #[cfg(feature = "arxiv")]
    #[rmcp::tool(description = "Search for papers by category on arXiv")]
    pub async fn arxiv_search_by_category(
        &self,
        #[rmcp::tool(aggr)] req: ArxivSearchByCategoryRequest,
    ) -> Result<String, String> {
        let client = self.arxiv.as_ref()
            .ok_or_else(|| "arXiv client not configured".to_string())?;

        let papers = client.search_by_category(&req.category, req.max_results).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&papers).map_err(|e| e.to_string())
    }
}
