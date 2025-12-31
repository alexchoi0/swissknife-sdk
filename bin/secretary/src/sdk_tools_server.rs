use std::sync::Arc;
use rmcp::{
    ServerHandler,
    tool, tool_router, tool_handler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerInfo, ServerCapabilities, Implementation, ProtocolVersion},
    schemars::JsonSchema,
};
use serde::Deserialize;
use swissknife_search_sdk::{SearchOptions, SearchProvider};
use swissknife_search_sdk::tavily::TavilyClient;

#[derive(Clone)]
pub struct SdkToolServer {
    tavily: Option<Arc<TavilyClient>>,
    tool_router: ToolRouter<Self>,
}

impl SdkToolServer {
    pub fn new() -> Self {
        let tavily = std::env::var("TAVILY_API_KEY")
            .ok()
            .map(|key| Arc::new(TavilyClient::new(&key)));

        if tavily.is_some() {
            eprintln!("SDK Tools: Tavily search enabled");
        }

        Self {
            tavily,
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for SdkToolServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SdkToolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "swissknife-sdk-tools".into(),
                title: Some("Swissknife SDK Tools".into()),
                version: "0.1.0".into(),
                icons: None,
                website_url: None,
            },
            instructions: Some("Swissknife SDK tools including web search".into()),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TavilySearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WebFetchRequest {
    pub url: String,
}

#[tool_router]
impl SdkToolServer {
    #[tool(description = "Search the web using Tavily AI-powered search engine")]
    pub async fn tavily_search(
        &self,
        Parameters(req): Parameters<TavilySearchRequest>,
    ) -> Result<String, String> {
        let client = self.tavily.as_ref()
            .ok_or_else(|| "Tavily not configured. Set TAVILY_API_KEY environment variable.".to_string())?;

        let options = SearchOptions {
            max_results: req.max_results,
            include_answer: true,
            ..Default::default()
        };

        let response = client.search(&req.query, &options).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "query": response.query,
            "answer": response.answer,
            "results": response.results.iter().map(|r| serde_json::json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.snippet,
                "content": r.content
            })).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[tool(description = "Fetch content from a URL")]
    pub async fn web_fetch(
        &self,
        Parameters(req): Parameters<WebFetchRequest>,
    ) -> Result<String, String> {
        let client = reqwest::Client::new();
        let response = client.get(&req.url)
            .header("User-Agent", "Mozilla/5.0 (compatible; Secretary/1.0)")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let text = response.text().await.map_err(|e| e.to_string())?;

        if text.len() > 10000 {
            Ok(format!("Status: {}\nContent (truncated):\n{}", status, &text[..10000]))
        } else {
            Ok(format!("Status: {}\nContent:\n{}", status, text))
        }
    }
}
