use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult, ContentExtractor, ExtractedContent};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.exa.ai";

pub struct ExaClient {
    api_key: String,
    client: Client,
}

impl ExaClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ExaSearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_published_date: Option<String>,
    use_autoprompt: bool,
    r#type: String,
}

#[derive(Serialize)]
struct ExaContentsRequest {
    ids: Vec<String>,
    text: bool,
}

#[derive(Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
    autoprompt_string: Option<String>,
}

#[derive(Deserialize)]
struct ExaResult {
    id: String,
    title: Option<String>,
    url: String,
    score: Option<f64>,
    published_date: Option<String>,
    text: Option<String>,
}

#[derive(Deserialize)]
struct ExaContentsResponse {
    results: Vec<ExaContentResult>,
}

#[derive(Deserialize)]
struct ExaContentResult {
    id: String,
    url: String,
    title: Option<String>,
    text: Option<String>,
}

#[async_trait]
impl SearchProvider for ExaClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let request = ExaSearchRequest {
            query: query.to_string(),
            num_results: options.max_results,
            include_domains: if options.include_domains.is_empty() { None } else { Some(options.include_domains.clone()) },
            exclude_domains: if options.exclude_domains.is_empty() { None } else { Some(options.exclude_domains.clone()) },
            start_published_date: None,
            end_published_date: None,
            use_autoprompt: true,
            r#type: "neural".to_string(),
        };

        let response = self.client
            .post(format!("{}/search", BASE_URL))
            .header("x-api-key", &self.api_key)
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

        let exa_response: ExaSearchResponse = response.json().await?;

        let results = exa_response.results.into_iter().map(|r| SearchResult {
            title: r.title.unwrap_or_default(),
            url: r.url,
            snippet: r.text.clone(),
            content: r.text,
            score: r.score,
            published_date: r.published_date,
        }).collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
            answer: None,
            total_results: None,
        })
    }
}

#[async_trait]
impl ContentExtractor for ExaClient {
    async fn extract(&self, url: &str) -> Result<ExtractedContent> {
        let results = self.extract_many(&[url]).await?;
        results.into_iter().next().ok_or_else(|| Error::NotFound(url.to_string()))
    }

    async fn extract_many(&self, urls: &[&str]) -> Result<Vec<ExtractedContent>> {
        let search_request = ExaSearchRequest {
            query: urls.join(" OR "),
            num_results: Some(urls.len() as u32),
            include_domains: None,
            exclude_domains: None,
            start_published_date: None,
            end_published_date: None,
            use_autoprompt: false,
            r#type: "keyword".to_string(),
        };

        let search_response = self.client
            .post(format!("{}/search", BASE_URL))
            .header("x-api-key", &self.api_key)
            .json(&search_request)
            .send()
            .await?;

        if !search_response.status().is_success() {
            let status = search_response.status();
            let text = search_response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let exa_search: ExaSearchResponse = search_response.json().await?;
        let ids: Vec<String> = exa_search.results.iter().map(|r| r.id.clone()).collect();

        let contents_request = ExaContentsRequest {
            ids,
            text: true,
        };

        let contents_response = self.client
            .post(format!("{}/contents", BASE_URL))
            .header("x-api-key", &self.api_key)
            .json(&contents_request)
            .send()
            .await?;

        if !contents_response.status().is_success() {
            let status = contents_response.status();
            let text = contents_response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let exa_contents: ExaContentsResponse = contents_response.json().await?;

        Ok(exa_contents.results.into_iter().map(|r| ExtractedContent {
            url: r.url,
            title: r.title,
            content: r.text.clone().unwrap_or_default(),
            markdown: r.text,
            links: vec![],
            images: vec![],
        }).collect())
    }
}
