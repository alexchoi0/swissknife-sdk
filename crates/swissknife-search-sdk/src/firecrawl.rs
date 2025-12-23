use crate::{Error, Result, ContentExtractor, ExtractedContent, WebCrawler, CrawlResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.firecrawl.dev/v1";

pub struct FirecrawlClient {
    api_key: String,
    client: Client,
}

impl FirecrawlClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn scrape_with_options(&self, url: &str, options: ScrapeOptions) -> Result<ScrapeResult> {
        let request = FirecrawlScrapeRequest {
            url: url.to_string(),
            formats: options.formats,
            only_main_content: options.only_main_content,
            include_tags: options.include_tags,
            exclude_tags: options.exclude_tags,
            wait_for: options.wait_for,
        };

        let response = self.client
            .post(format!("{}/scrape", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let result: FirecrawlScrapeResponse = response.json().await?;

        Ok(ScrapeResult {
            markdown: result.data.markdown,
            html: result.data.html,
            raw_html: result.data.raw_html,
            links: result.data.links.unwrap_or_default(),
            metadata: result.data.metadata,
        })
    }

    pub async fn map_site(&self, url: &str, options: MapOptions) -> Result<Vec<String>> {
        let request = FirecrawlMapRequest {
            url: url.to_string(),
            search: options.search,
            ignore_sitemap: options.ignore_sitemap,
            include_subdomains: options.include_subdomains,
            limit: options.limit,
        };

        let response = self.client
            .post(format!("{}/map", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let result: FirecrawlMapResponse = response.json().await?;
        Ok(result.links)
    }
}

#[derive(Default)]
pub struct ScrapeOptions {
    pub formats: Vec<String>,
    pub only_main_content: Option<bool>,
    pub include_tags: Option<Vec<String>>,
    pub exclude_tags: Option<Vec<String>>,
    pub wait_for: Option<u32>,
}

pub struct ScrapeResult {
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub raw_html: Option<String>,
    pub links: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Default)]
pub struct MapOptions {
    pub search: Option<String>,
    pub ignore_sitemap: Option<bool>,
    pub include_subdomains: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
struct FirecrawlScrapeRequest {
    url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    formats: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    only_main_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for: Option<u32>,
}

#[derive(Deserialize)]
struct FirecrawlScrapeResponse {
    success: bool,
    data: FirecrawlScrapeData,
}

#[derive(Deserialize)]
struct FirecrawlScrapeData {
    markdown: Option<String>,
    html: Option<String>,
    #[serde(rename = "rawHtml")]
    raw_html: Option<String>,
    links: Option<Vec<String>>,
    metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct FirecrawlMapRequest {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore_sitemap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_subdomains: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct FirecrawlMapResponse {
    success: bool,
    links: Vec<String>,
}

#[derive(Serialize)]
struct FirecrawlCrawlRequest {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_paths: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct FirecrawlCrawlResponse {
    success: bool,
    id: String,
}

#[derive(Deserialize)]
struct FirecrawlCrawlStatusResponse {
    status: String,
    total: Option<usize>,
    completed: Option<usize>,
    data: Option<Vec<FirecrawlCrawlPage>>,
}

#[derive(Deserialize)]
struct FirecrawlCrawlPage {
    url: String,
    markdown: Option<String>,
    html: Option<String>,
    metadata: Option<FirecrawlPageMetadata>,
}

#[derive(Deserialize)]
struct FirecrawlPageMetadata {
    title: Option<String>,
}

#[async_trait]
impl ContentExtractor for FirecrawlClient {
    async fn extract(&self, url: &str) -> Result<ExtractedContent> {
        let options = ScrapeOptions {
            formats: vec!["markdown".to_string()],
            only_main_content: Some(true),
            ..Default::default()
        };

        let result = self.scrape_with_options(url, options).await?;

        let title = result.metadata
            .as_ref()
            .and_then(|m| m.get("title"))
            .and_then(|t| t.as_str())
            .map(String::from);

        Ok(ExtractedContent {
            url: url.to_string(),
            title,
            content: result.markdown.clone().unwrap_or_default(),
            markdown: result.markdown,
            links: result.links,
            images: vec![],
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

impl FirecrawlClient {
    pub async fn start_crawl(&self, url: &str, max_depth: u32, max_pages: u32) -> Result<String> {
        let request = FirecrawlCrawlRequest {
            url: url.to_string(),
            max_depth: Some(max_depth),
            limit: Some(max_pages),
            exclude_paths: None,
            include_paths: None,
        };

        let response = self.client
            .post(format!("{}/crawl", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let crawl_response: FirecrawlCrawlResponse = response.json().await?;
        Ok(crawl_response.id)
    }

    pub async fn get_crawl_status(&self, crawl_id: &str) -> Result<CrawlStatus> {
        let response = self.client
            .get(format!("{}/crawl/{}", BASE_URL, crawl_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let status: FirecrawlCrawlStatusResponse = response.json().await?;

        Ok(CrawlStatus {
            status: status.status,
            total: status.total,
            completed: status.completed,
            pages: status.data.map(|data| data.into_iter().map(|p| {
                ExtractedContent {
                    url: p.url,
                    title: p.metadata.and_then(|m| m.title),
                    content: p.markdown.clone().unwrap_or_default(),
                    markdown: p.markdown,
                    links: vec![],
                    images: vec![],
                }
            }).collect()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CrawlStatus {
    pub status: String,
    pub total: Option<usize>,
    pub completed: Option<usize>,
    pub pages: Option<Vec<ExtractedContent>>,
}

#[async_trait]
impl WebCrawler for FirecrawlClient {
    async fn crawl(&self, url: &str, max_depth: u32, max_pages: u32) -> Result<CrawlResult> {
        let crawl_id = self.start_crawl(url, max_depth, max_pages).await?;
        let status = self.get_crawl_status(&crawl_id).await?;

        if status.status == "completed" {
            let pages = status.pages.unwrap_or_default();
            let total_pages = pages.len();
            return Ok(CrawlResult {
                base_url: url.to_string(),
                pages,
                total_pages,
            });
        }

        Err(Error::Api {
            message: format!("Crawl job started with ID: {}. Poll get_crawl_status() for results.", crawl_id),
            code: Some("PENDING".to_string()),
        })
    }
}
