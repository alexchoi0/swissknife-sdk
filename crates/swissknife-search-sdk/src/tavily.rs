use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult, SearchDepth, TimeRange};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.tavily.com";

pub struct TavilyClient {
    api_key: String,
    client: Client,
}

impl TavilyClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct TavilySearchRequest {
    api_key: String,
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_answer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_domains: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct TavilySearchResponse {
    query: String,
    answer: Option<String>,
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: Option<String>,
    score: Option<f64>,
    published_date: Option<String>,
}

#[async_trait]
impl SearchProvider for TavilyClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let search_depth = options.search_depth.map(|d| match d {
            SearchDepth::Basic => "basic".to_string(),
            SearchDepth::Advanced => "advanced".to_string(),
        });

        let request = TavilySearchRequest {
            api_key: self.api_key.clone(),
            query: query.to_string(),
            search_depth,
            max_results: options.max_results,
            include_answer: Some(options.include_answer),
            include_domains: if options.include_domains.is_empty() { None } else { Some(options.include_domains.clone()) },
            exclude_domains: if options.exclude_domains.is_empty() { None } else { Some(options.exclude_domains.clone()) },
        };

        let response = self.client
            .post(format!("{}/search", BASE_URL))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let tavily_response: TavilySearchResponse = response.json().await?;

        let results = tavily_response.results.into_iter().map(|r| SearchResult {
            title: r.title,
            url: r.url,
            snippet: r.content.clone(),
            content: r.content,
            score: r.score,
            published_date: r.published_date,
        }).collect();

        Ok(SearchResponse {
            query: tavily_response.query,
            results,
            answer: tavily_response.answer,
            total_results: None,
        })
    }
}
