use std::sync::Arc;
use async_trait::async_trait;
use crate::Result;
use crate::mcp::types::{ToolDefinition, ToolResult};
use crate::mcp::provider::ToolProvider;
use swissknife_search_sdk::{SearchOptions, SearchProvider as SearchTrait};

#[cfg(feature = "tavily")]
use swissknife_search_sdk::tavily::TavilyClient;

pub struct WebSearchProvider {
    #[cfg(feature = "tavily")]
    tavily: Option<Arc<TavilyClient>>,
}

impl WebSearchProvider {
    pub fn new() -> Self {
        #[cfg(feature = "tavily")]
        let tavily = std::env::var("TAVILY_API_KEY")
            .ok()
            .map(|key| Arc::new(TavilyClient::new(&key)));

        Self {
            #[cfg(feature = "tavily")]
            tavily,
        }
    }

    #[cfg(feature = "tavily")]
    pub fn with_tavily(mut self, client: TavilyClient) -> Self {
        self.tavily = Some(Arc::new(client));
        self
    }
}

impl Default for WebSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolProvider for WebSearchProvider {
    fn tools(&self) -> Vec<ToolDefinition> {
        let mut tools = Vec::new();

        #[cfg(feature = "tavily")]
        if self.tavily.is_some() {
            tools.push(ToolDefinition::new("tavily_search")
                .with_description("Search the web using Tavily AI-powered search engine")
                .with_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return"
                        }
                    },
                    "required": ["query"]
                })));
        }

        tools.push(ToolDefinition::new("web_fetch")
            .with_description("Fetch content from a URL")
            .with_schema(serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch"
                    }
                },
                "required": ["url"]
            })));

        tools
    }

    async fn call(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        match name {
            #[cfg(feature = "tavily")]
            "tavily_search" => {
                let client = self.tavily.as_ref()
                    .ok_or_else(|| crate::Error::Provider("Tavily not configured".into()))?;

                let query = arguments.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::Error::MissingParameter("query".into()))?;

                let max_results = arguments.get("max_results")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);

                let options = SearchOptions {
                    max_results,
                    include_answer: true,
                    ..Default::default()
                };

                let response = client.search(query, &options).await
                    .map_err(|e| crate::Error::Provider(e.to_string()))?;

                let result = serde_json::json!({
                    "query": response.query,
                    "answer": response.answer,
                    "results": response.results.iter().map(|r| serde_json::json!({
                        "title": r.title,
                        "url": r.url,
                        "snippet": r.snippet,
                        "content": r.content
                    })).collect::<Vec<_>>()
                });

                Ok(ToolResult::json(result))
            }
            "web_fetch" => {
                let url = arguments.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::Error::MissingParameter("url".into()))?;

                let client = reqwest::Client::new();
                let response = client.get(url)
                    .header("User-Agent", "Mozilla/5.0 (compatible; SwissknifeMCP/1.0)")
                    .send()
                    .await
                    .map_err(|e| crate::Error::Provider(e.to_string()))?;

                let status = response.status();
                let text = response.text().await
                    .map_err(|e| crate::Error::Provider(e.to_string()))?;

                let content = if text.len() > 10000 {
                    format!("Status: {}\nContent (truncated):\n{}", status, &text[..10000])
                } else {
                    format!("Status: {}\nContent:\n{}", status, text)
                };

                Ok(ToolResult::text(content))
            }
            _ => Err(crate::Error::ToolNotFound(name.to_string())),
        }
    }
}
