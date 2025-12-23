use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult, TimeRange};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://google.serper.dev";

pub struct SerperClient {
    api_key: String,
    client: Client,
}

impl SerperClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn search_images(&self, query: &str, num: Option<u32>) -> Result<Vec<ImageResult>> {
        let request = SerperSearchRequest {
            q: query.to_string(),
            num: num.or(Some(10)),
            tbs: None,
        };

        let response = self.client
            .post(format!("{}/images", BASE_URL))
            .header("X-API-KEY", &self.api_key)
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

        let serper_response: SerperImageResponse = response.json().await?;
        Ok(serper_response.images.into_iter().map(|i| ImageResult {
            title: i.title,
            image_url: i.image_url,
            link: i.link,
            source: i.source,
        }).collect())
    }

    pub async fn search_news(&self, query: &str, num: Option<u32>) -> Result<Vec<NewsResult>> {
        let request = SerperSearchRequest {
            q: query.to_string(),
            num: num.or(Some(10)),
            tbs: None,
        };

        let response = self.client
            .post(format!("{}/news", BASE_URL))
            .header("X-API-KEY", &self.api_key)
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

        let serper_response: SerperNewsResponse = response.json().await?;
        Ok(serper_response.news.into_iter().map(|n| NewsResult {
            title: n.title,
            link: n.link,
            snippet: n.snippet,
            date: n.date,
            source: n.source,
        }).collect())
    }
}

#[derive(Serialize)]
struct SerperSearchRequest {
    q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    num: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tbs: Option<String>,
}

#[derive(Deserialize)]
struct SerperSearchResponse {
    #[serde(default)]
    organic: Vec<SerperResult>,
    #[serde(default)]
    answer_box: Option<SerperAnswerBox>,
    search_parameters: Option<SerperSearchParams>,
}

#[derive(Deserialize)]
struct SerperSearchParams {
    q: String,
}

#[derive(Deserialize)]
struct SerperResult {
    title: String,
    link: String,
    snippet: Option<String>,
    position: Option<u32>,
    date: Option<String>,
}

#[derive(Deserialize)]
struct SerperAnswerBox {
    answer: Option<String>,
    snippet: Option<String>,
}

#[derive(Deserialize)]
struct SerperImageResponse {
    images: Vec<SerperImage>,
}

#[derive(Deserialize)]
struct SerperImage {
    title: String,
    #[serde(rename = "imageUrl")]
    image_url: String,
    link: String,
    source: Option<String>,
}

#[derive(Deserialize)]
struct SerperNewsResponse {
    news: Vec<SerperNews>,
}

#[derive(Deserialize)]
struct SerperNews {
    title: String,
    link: String,
    snippet: Option<String>,
    date: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImageResult {
    pub title: String,
    pub image_url: String,
    pub link: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewsResult {
    pub title: String,
    pub link: String,
    pub snippet: Option<String>,
    pub date: Option<String>,
    pub source: Option<String>,
}

#[async_trait]
impl SearchProvider for SerperClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let tbs = options.time_range.map(|t| match t {
            TimeRange::Day => "qdr:d".to_string(),
            TimeRange::Week => "qdr:w".to_string(),
            TimeRange::Month => "qdr:m".to_string(),
            TimeRange::Year => "qdr:y".to_string(),
        });

        let request = SerperSearchRequest {
            q: query.to_string(),
            num: options.max_results,
            tbs,
        };

        let response = self.client
            .post(format!("{}/search", BASE_URL))
            .header("X-API-KEY", &self.api_key)
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

        let serper_response: SerperSearchResponse = response.json().await?;

        let results = serper_response.organic.into_iter().map(|r| SearchResult {
            title: r.title,
            url: r.link,
            snippet: r.snippet.clone(),
            content: r.snippet,
            score: r.position.map(|p| 1.0 / p as f64),
            published_date: r.date,
        }).collect();

        let answer = serper_response.answer_box.and_then(|ab| ab.answer.or(ab.snippet));

        Ok(SearchResponse {
            query: serper_response.search_parameters.map(|p| p.q).unwrap_or_else(|| query.to_string()),
            results,
            answer,
            total_results: None,
        })
    }
}
