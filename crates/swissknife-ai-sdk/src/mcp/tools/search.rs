use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "search")]
use swissknife_search_sdk as search;

#[derive(Clone)]
pub struct SearchTools {
    #[cfg(feature = "tavily")]
    pub tavily: Option<search::tavily::TavilyClient>,
    #[cfg(feature = "exa")]
    pub exa: Option<search::exa::ExaClient>,
    #[cfg(feature = "serper")]
    pub serper: Option<search::serper::SerperClient>,
    #[cfg(feature = "perplexity")]
    pub perplexity: Option<search::perplexity::PerplexityClient>,
    #[cfg(feature = "duckduckgo")]
    pub duckduckgo: Option<search::duckduckgo::DuckDuckGoClient>,
    #[cfg(feature = "wikipedia")]
    pub wikipedia: Option<search::wikipedia::WikipediaClient>,
}

impl SearchTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "tavily")]
            tavily: None,
            #[cfg(feature = "exa")]
            exa: None,
            #[cfg(feature = "serper")]
            serper: None,
            #[cfg(feature = "perplexity")]
            perplexity: None,
            #[cfg(feature = "duckduckgo")]
            duckduckgo: None,
            #[cfg(feature = "wikipedia")]
            wikipedia: None,
        }
    }

    #[cfg(feature = "tavily")]
    pub fn with_tavily(mut self, client: search::tavily::TavilyClient) -> Self {
        self.tavily = Some(client);
        self
    }

    #[cfg(feature = "exa")]
    pub fn with_exa(mut self, client: search::exa::ExaClient) -> Self {
        self.exa = Some(client);
        self
    }

    #[cfg(feature = "serper")]
    pub fn with_serper(mut self, client: search::serper::SerperClient) -> Self {
        self.serper = Some(client);
        self
    }

    #[cfg(feature = "perplexity")]
    pub fn with_perplexity(mut self, client: search::perplexity::PerplexityClient) -> Self {
        self.perplexity = Some(client);
        self
    }

    #[cfg(feature = "duckduckgo")]
    pub fn with_duckduckgo(mut self, client: search::duckduckgo::DuckDuckGoClient) -> Self {
        self.duckduckgo = Some(client);
        self
    }

    #[cfg(feature = "wikipedia")]
    pub fn with_wikipedia(mut self, client: search::wikipedia::WikipediaClient) -> Self {
        self.wikipedia = Some(client);
        self
    }
}

impl Default for SearchTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TavilySearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
    #[serde(default)]
    pub include_answer: Option<bool>,
    #[serde(default)]
    pub search_depth: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExaSearchRequest {
    pub query: String,
    #[serde(default)]
    pub num_results: Option<u32>,
    #[serde(default)]
    pub search_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SerperSearchRequest {
    pub query: String,
    #[serde(default)]
    pub num: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PerplexityRequest {
    pub question: String,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DuckDuckGoRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WikipediaSearchRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WikipediaPageRequest {
    pub title: String,
}

#[tool_router]
impl SearchTools {
    #[cfg(feature = "tavily")]
    #[rmcp::tool(description = "Search the web using Tavily AI-powered search")]
    pub async fn tavily_search(
        &self,
        #[rmcp::tool(aggr)] req: TavilySearchRequest,
    ) -> Result<String, String> {
        let client = self.tavily.as_ref()
            .ok_or_else(|| "Tavily client not configured".to_string())?;

        let mut request = search::tavily::SearchRequest::new(req.query);

        if let Some(max) = req.max_results {
            request = request.with_max_results(max);
        }
        if let Some(include) = req.include_answer {
            request = request.with_include_answer(include);
        }
        if let Some(depth) = req.search_depth {
            let d = match depth.as_str() {
                "advanced" => search::tavily::SearchDepth::Advanced,
                _ => search::tavily::SearchDepth::Basic,
            };
            request = request.with_search_depth(d);
        }

        let response = client.search(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "answer": response.answer,
            "results": response.results.iter().map(|r| {
                serde_json::json!({
                    "title": r.title,
                    "url": r.url,
                    "content": r.content,
                    "score": r.score
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "exa")]
    #[rmcp::tool(description = "Search using Exa neural search engine")]
    pub async fn exa_search(
        &self,
        #[rmcp::tool(aggr)] req: ExaSearchRequest,
    ) -> Result<String, String> {
        let client = self.exa.as_ref()
            .ok_or_else(|| "Exa client not configured".to_string())?;

        let mut request = search::exa::SearchRequest::new(req.query);

        if let Some(num) = req.num_results {
            request = request.with_num_results(num);
        }
        if let Some(t) = req.search_type {
            let search_type = match t.as_str() {
                "neural" => search::exa::SearchType::Neural,
                "keyword" => search::exa::SearchType::Keyword,
                _ => search::exa::SearchType::Auto,
            };
            request = request.with_search_type(search_type);
        }

        let response = client.search(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": response.results.iter().map(|r| {
                serde_json::json!({
                    "title": r.title,
                    "url": r.url,
                    "score": r.score,
                    "text": r.text
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "serper")]
    #[rmcp::tool(description = "Search Google via Serper API")]
    pub async fn serper_search(
        &self,
        #[rmcp::tool(aggr)] req: SerperSearchRequest,
    ) -> Result<String, String> {
        let client = self.serper.as_ref()
            .ok_or_else(|| "Serper client not configured".to_string())?;

        let mut request = search::serper::SearchRequest::new(req.query);

        if let Some(num) = req.num {
            request = request.with_num(num);
        }

        let response = client.search(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "organic": response.organic.iter().map(|r| {
                serde_json::json!({
                    "title": r.title,
                    "link": r.link,
                    "snippet": r.snippet
                })
            }).collect::<Vec<_>>(),
            "answer_box": response.answer_box
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "perplexity")]
    #[rmcp::tool(description = "Ask a question using Perplexity AI with web search")]
    pub async fn perplexity_ask(
        &self,
        #[rmcp::tool(aggr)] req: PerplexityRequest,
    ) -> Result<String, String> {
        let client = self.perplexity.as_ref()
            .ok_or_else(|| "Perplexity client not configured".to_string())?;

        let model = req.model
            .unwrap_or_else(|| "llama-3.1-sonar-small-128k-online".to_string());

        let messages = vec![search::perplexity::Message {
            role: search::perplexity::Role::User,
            content: req.question,
        }];

        let request = search::perplexity::ChatRequest::new(model, messages);
        let response = client.chat(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "answer": response.choices.first().map(|c| &c.message.content),
            "citations": response.citations
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "duckduckgo")]
    #[rmcp::tool(description = "Search using DuckDuckGo")]
    pub async fn duckduckgo_search(
        &self,
        #[rmcp::tool(aggr)] req: DuckDuckGoRequest,
    ) -> Result<String, String> {
        let client = self.duckduckgo.as_ref()
            .ok_or_else(|| "DuckDuckGo client not configured".to_string())?;

        let mut params = search::duckduckgo::SearchParams::new(req.query);

        if let Some(max) = req.max_results {
            params = params.with_max_results(max);
        }

        let results = client.search(&params).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "title": r.title,
                    "url": r.url,
                    "body": r.body
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "wikipedia")]
    #[rmcp::tool(description = "Search Wikipedia articles")]
    pub async fn wikipedia_search(
        &self,
        #[rmcp::tool(aggr)] req: WikipediaSearchRequest,
    ) -> Result<String, String> {
        let client = self.wikipedia.as_ref()
            .ok_or_else(|| "Wikipedia client not configured".to_string())?;

        let mut params = search::wikipedia::SearchParams::new(req.query);

        if let Some(limit) = req.limit {
            params = params.with_limit(limit);
        }

        let results = client.search(&params).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "title": r.title,
                    "pageid": r.pageid,
                    "snippet": r.snippet
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "wikipedia")]
    #[rmcp::tool(description = "Get a Wikipedia article by title")]
    pub async fn wikipedia_page(
        &self,
        #[rmcp::tool(aggr)] req: WikipediaPageRequest,
    ) -> Result<String, String> {
        let client = self.wikipedia.as_ref()
            .ok_or_else(|| "Wikipedia client not configured".to_string())?;

        let page = client.get_page(&req.title).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "title": page.title,
            "pageid": page.pageid,
            "extract": page.extract,
            "url": page.fullurl
        })).map_err(|e| e.to_string())
    }
}
