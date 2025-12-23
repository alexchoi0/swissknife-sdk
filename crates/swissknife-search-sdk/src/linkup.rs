use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.linkup.so/v1";

pub struct LinkupClient {
    api_key: String,
    client: Client,
}

impl LinkupClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn search_with_options(&self, query: &str, options: LinkupSearchOptions) -> Result<LinkupSearchResponse> {
        let request = LinkupSearchRequest {
            q: query.to_string(),
            depth: options.depth.unwrap_or_else(|| "standard".to_string()),
            output_type: options.output_type.unwrap_or_else(|| "searchResults".to_string()),
            include_images: options.include_images,
        };

        let response = self.client
            .post(format!("{}/search", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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

        let linkup_response: LinkupApiResponse = response.json().await?;

        Ok(LinkupSearchResponse {
            results: linkup_response.results.into_iter().map(|r| LinkupResult {
                name: r.name,
                url: r.url,
                content: r.content,
            }).collect(),
            answer: linkup_response.answer,
            images: linkup_response.images.unwrap_or_default(),
        })
    }
}

#[derive(Default)]
pub struct LinkupSearchOptions {
    pub depth: Option<String>,
    pub output_type: Option<String>,
    pub include_images: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LinkupSearchResponse {
    pub results: Vec<LinkupResult>,
    pub answer: Option<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LinkupResult {
    pub name: String,
    pub url: String,
    pub content: Option<String>,
}

#[derive(Serialize)]
struct LinkupSearchRequest {
    q: String,
    depth: String,
    #[serde(rename = "outputType")]
    output_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_images: Option<bool>,
}

#[derive(Deserialize)]
struct LinkupApiResponse {
    results: Vec<LinkupApiResult>,
    answer: Option<String>,
    images: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct LinkupApiResult {
    name: String,
    url: String,
    content: Option<String>,
}

#[async_trait]
impl SearchProvider for LinkupClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let depth = match options.search_depth {
            Some(crate::SearchDepth::Basic) => "standard".to_string(),
            Some(crate::SearchDepth::Advanced) => "deep".to_string(),
            None => "standard".to_string(),
        };

        let linkup_options = LinkupSearchOptions {
            depth: Some(depth),
            output_type: if options.include_answer {
                Some("sourcedAnswer".to_string())
            } else {
                Some("searchResults".to_string())
            },
            include_images: None,
        };

        let linkup_response = self.search_with_options(query, linkup_options).await?;

        let max_results = options.max_results.unwrap_or(10) as usize;
        let results: Vec<SearchResult> = linkup_response.results.into_iter()
            .take(max_results)
            .enumerate()
            .map(|(i, r)| SearchResult {
                title: r.name,
                url: r.url,
                snippet: r.content.clone(),
                content: r.content,
                score: Some(1.0 / (i + 1) as f64),
                published_date: None,
            })
            .collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
            answer: linkup_response.answer,
            total_results: None,
        })
    }
}
