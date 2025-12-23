use crate::{Error, Result, ContentExtractor, ExtractedContent, SearchProvider, SearchOptions, SearchResponse, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const READER_URL: &str = "https://r.jina.ai";
const SEARCH_URL: &str = "https://s.jina.ai";

pub struct JinaClient {
    api_key: Option<String>,
    client: Client,
}

impl JinaClient {
    pub fn new(api_key: Option<&str>) -> Self {
        Self {
            api_key: api_key.map(String::from),
            client: Client::new(),
        }
    }

    pub fn with_api_key(api_key: &str) -> Self {
        Self::new(Some(api_key))
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url);
        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        request
    }
}

#[derive(Deserialize)]
struct JinaReaderResponse {
    data: JinaReaderData,
}

#[derive(Deserialize)]
struct JinaReaderData {
    url: String,
    title: String,
    content: String,
    #[serde(default)]
    links: Option<serde_json::Value>,
    #[serde(default)]
    images: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct JinaSearchResponse {
    data: Vec<JinaSearchResult>,
}

#[derive(Deserialize)]
struct JinaSearchResult {
    title: String,
    url: String,
    content: Option<String>,
    description: Option<String>,
}

#[async_trait]
impl ContentExtractor for JinaClient {
    async fn extract(&self, url: &str) -> Result<ExtractedContent> {
        let reader_url = format!("{}/{}", READER_URL, url);

        let response = self.build_request(&reader_url)
            .header("Accept", "application/json")
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

        let jina_response: JinaReaderResponse = response.json().await?;

        let links = jina_response.data.links
            .and_then(|v| v.as_array().map(|arr| arr.iter().filter_map(|l| l.as_str().map(String::from)).collect()))
            .unwrap_or_default();

        let images = jina_response.data.images
            .and_then(|v| v.as_array().map(|arr| arr.iter().filter_map(|i| i.as_str().map(String::from)).collect()))
            .unwrap_or_default();

        Ok(ExtractedContent {
            url: jina_response.data.url,
            title: Some(jina_response.data.title),
            content: jina_response.data.content.clone(),
            markdown: Some(jina_response.data.content),
            links,
            images,
        })
    }

    async fn extract_many(&self, urls: &[&str]) -> Result<Vec<ExtractedContent>> {
        let mut results = Vec::new();
        for url in urls {
            match self.extract(url).await {
                Ok(content) => results.push(content),
                Err(_) => continue,
            }
        }
        Ok(results)
    }
}

#[async_trait]
impl SearchProvider for JinaClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let search_url = format!("{}/{}", SEARCH_URL, urlencoding::encode(query));

        let response = self.build_request(&search_url)
            .header("Accept", "application/json")
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

        let jina_response: JinaSearchResponse = response.json().await?;

        let max_results = options.max_results.unwrap_or(10) as usize;
        let results: Vec<SearchResult> = jina_response.data.into_iter()
            .take(max_results)
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                snippet: r.description.clone(),
                content: r.content,
                score: None,
                published_date: None,
            })
            .collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
            answer: None,
            total_results: None,
        })
    }
}
