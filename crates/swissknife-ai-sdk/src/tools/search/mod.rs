use crate::error::Result;
use crate::tool::{get_array_param, get_bool_param, get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_search_sdk::{SearchOptions, SearchProvider, ContentExtractor, WebCrawler};

pub struct WebSearchTool;

impl Default for WebSearchTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "web_search",
            "Web Search",
            "Search the web using various search providers (Tavily, Exa, Serper, etc.)",
            "search",
        )
        .with_param("api_key", ParameterSchema::string("Search provider API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Search provider: tavily, exa, serper, jina, perplexity, duckduckgo").with_default(serde_json::json!("tavily")))
        .with_param("query", ParameterSchema::string("Search query").required())
        .with_param("max_results", ParameterSchema::integer("Maximum number of results").with_default(serde_json::json!(10)))
        .with_param("include_answer", ParameterSchema::boolean("Include AI-generated answer (if supported)").with_default(serde_json::json!(false)))
        .with_param("include_domains", ParameterSchema::array("Domains to include in search", ParameterSchema::string("Domain")))
        .with_param("exclude_domains", ParameterSchema::array("Domains to exclude from search", ParameterSchema::string("Domain")))
        .with_output("results", OutputSchema::array("Search results with title, url, snippet", OutputSchema::json("Result")))
        .with_output("answer", OutputSchema::string("AI-generated answer if requested"))
        .with_output("total_results", OutputSchema::number("Total number of results found"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_string_param(&params, "provider").unwrap_or_else(|| "tavily".to_string());
        let query = get_required_string_param(&params, "query")?;
        let max_results = get_i64_param(&params, "max_results").unwrap_or(10) as u32;
        let include_answer = get_bool_param(&params, "include_answer").unwrap_or(false);
        let include_domains = get_array_param(&params, "include_domains")
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let exclude_domains = get_array_param(&params, "exclude_domains")
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let options = SearchOptions {
            max_results: Some(max_results),
            include_answer,
            include_domains,
            exclude_domains,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "tavily")]
            "tavily" => {
                use swissknife_search_sdk::tavily::TavilyClient;
                let client = TavilyClient::new(&api_key);
                client.search(&query, &options).await
            }
            #[cfg(feature = "exa")]
            "exa" => {
                use swissknife_search_sdk::exa::ExaClient;
                let client = ExaClient::new(&api_key);
                client.search(&query, &options).await
            }
            #[cfg(feature = "serper")]
            "serper" => {
                use swissknife_search_sdk::serper::SerperClient;
                let client = SerperClient::new(&api_key);
                client.search(&query, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported search provider: {}", provider)));
            }
        };

        match result {
            Ok(response) => Ok(ToolResponse::success(serde_json::json!({
                "results": response.results.iter().map(|r| serde_json::json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": r.snippet,
                    "content": r.content,
                    "score": r.score,
                })).collect::<Vec<_>>(),
                "answer": response.answer,
                "total_results": response.total_results,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Search failed: {}", e))),
        }
    }
}

pub struct ExtractContentTool;

impl Default for ExtractContentTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ExtractContentTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "extract_content",
            "Extract Web Content",
            "Extract content from a URL using web scraping providers (Firecrawl, Jina, etc.)",
            "search",
        )
        .with_param("api_key", ParameterSchema::string("Provider API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Content extraction provider: firecrawl, jina, apify").with_default(serde_json::json!("firecrawl")))
        .with_param("url", ParameterSchema::string("URL to extract content from").required())
        .with_output("title", OutputSchema::string("Page title"))
        .with_output("content", OutputSchema::string("Extracted text content"))
        .with_output("markdown", OutputSchema::string("Content as markdown"))
        .with_output("links", OutputSchema::array("Links found on the page", OutputSchema::string("Link")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_string_param(&params, "provider").unwrap_or_else(|| "firecrawl".to_string());
        let url = get_required_string_param(&params, "url")?;

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "firecrawl")]
            "firecrawl" => {
                use swissknife_search_sdk::firecrawl::FirecrawlClient;
                let client = FirecrawlClient::new(&api_key);
                client.extract(&url).await
            }
            #[cfg(feature = "jina")]
            "jina" => {
                use swissknife_search_sdk::jina::JinaClient;
                let client = JinaClient::new(&api_key);
                client.extract(&url).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported content provider: {}", provider)));
            }
        };

        match result {
            Ok(content) => Ok(ToolResponse::success(serde_json::json!({
                "title": content.title,
                "content": content.content,
                "markdown": content.markdown,
                "links": content.links,
                "images": content.images,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Content extraction failed: {}", e))),
        }
    }
}

pub struct CrawlWebsiteTool;

impl Default for CrawlWebsiteTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CrawlWebsiteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "crawl_website",
            "Crawl Website",
            "Crawl a website and extract content from multiple pages",
            "search",
        )
        .with_param("api_key", ParameterSchema::string("Provider API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Crawl provider: firecrawl, apify").with_default(serde_json::json!("firecrawl")))
        .with_param("url", ParameterSchema::string("Base URL to crawl").required())
        .with_param("max_depth", ParameterSchema::integer("Maximum crawl depth").with_default(serde_json::json!(2)))
        .with_param("max_pages", ParameterSchema::integer("Maximum pages to crawl").with_default(serde_json::json!(10)))
        .with_output("pages", OutputSchema::array("Crawled pages with content", OutputSchema::json("Page")))
        .with_output("total_pages", OutputSchema::number("Total pages crawled"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_string_param(&params, "provider").unwrap_or_else(|| "firecrawl".to_string());
        let url = get_required_string_param(&params, "url")?;
        let max_depth = get_i64_param(&params, "max_depth").unwrap_or(2) as u32;
        let max_pages = get_i64_param(&params, "max_pages").unwrap_or(10) as u32;

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "firecrawl")]
            "firecrawl" => {
                use swissknife_search_sdk::firecrawl::FirecrawlClient;
                let client = FirecrawlClient::new(&api_key);
                client.crawl(&url, max_depth, max_pages).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported crawl provider: {}", provider)));
            }
        };

        match result {
            Ok(crawl_result) => Ok(ToolResponse::success(serde_json::json!({
                "pages": crawl_result.pages.iter().map(|p| serde_json::json!({
                    "url": p.url,
                    "title": p.title,
                    "content": p.content,
                    "markdown": p.markdown,
                })).collect::<Vec<_>>(),
                "total_pages": crawl_result.total_pages,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Crawl failed: {}", e))),
        }
    }
}
