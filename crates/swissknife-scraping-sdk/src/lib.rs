mod error;

pub use error::{Error, Result};

#[cfg(feature = "apify")]
pub mod apify;

#[cfg(feature = "browseruse")]
pub mod browseruse;

#[cfg(feature = "firecrawl")]
pub mod firecrawl;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedPage {
    pub url: String,
    pub title: Option<String>,
    pub html: Option<String>,
    pub text: Option<String>,
    pub markdown: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub links: Vec<String>,
    pub images: Vec<String>,
    pub scraped_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub pages: Vec<ScrapedPage>,
    pub total_pages: u32,
    pub duration_ms: Option<u64>,
    pub errors: Vec<CrawlError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlError {
    pub url: String,
    pub error: String,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct ScrapeOptions {
    pub include_html: bool,
    pub include_markdown: bool,
    pub include_links: bool,
    pub include_images: bool,
    pub wait_for_selector: Option<String>,
    pub timeout_ms: Option<u32>,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<Cookie>,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CrawlOptions {
    pub max_pages: Option<u32>,
    pub max_depth: Option<u32>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub scrape_options: ScrapeOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedData {
    pub data: serde_json::Value,
    pub source_url: String,
}

#[async_trait]
pub trait WebScraper: Send + Sync {
    async fn scrape(&self, url: &str, options: &ScrapeOptions) -> Result<ScrapedPage>;
    async fn scrape_batch(&self, urls: &[&str], options: &ScrapeOptions) -> Result<Vec<ScrapedPage>>;
}

#[async_trait]
pub trait WebCrawler: Send + Sync {
    async fn crawl(&self, start_url: &str, options: &CrawlOptions) -> Result<CrawlResult>;
    async fn crawl_sitemap(&self, sitemap_url: &str, options: &CrawlOptions) -> Result<CrawlResult>;
}

#[async_trait]
pub trait DataExtractor: Send + Sync {
    async fn extract(&self, url: &str, schema: &serde_json::Value) -> Result<ExtractedData>;
    async fn extract_with_prompt(&self, url: &str, prompt: &str) -> Result<ExtractedData>;
}
