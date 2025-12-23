mod error;

pub use error::{Error, Result};

#[cfg(feature = "tavily")]
pub mod tavily;

#[cfg(feature = "exa")]
pub mod exa;

#[cfg(feature = "serper")]
pub mod serper;

#[cfg(feature = "jina")]
pub mod jina;

#[cfg(feature = "firecrawl")]
pub mod firecrawl;

#[cfg(feature = "apify")]
pub mod apify;

#[cfg(feature = "perplexity")]
pub mod perplexity;

#[cfg(feature = "duckduckgo")]
pub mod duckduckgo;

#[cfg(feature = "ahrefs")]
pub mod ahrefs;

#[cfg(feature = "google")]
pub mod google;

#[cfg(feature = "linkup")]
pub mod linkup;

#[cfg(feature = "wikipedia")]
pub mod wikipedia;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub content: Option<String>,
    pub score: Option<f64>,
    pub published_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub answer: Option<String>,
    pub total_results: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub max_results: Option<u32>,
    pub search_depth: Option<SearchDepth>,
    pub include_answer: bool,
    pub include_domains: Vec<String>,
    pub exclude_domains: Vec<String>,
    pub time_range: Option<TimeRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDepth {
    Basic,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub markdown: Option<String>,
    pub links: Vec<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub base_url: String,
    pub pages: Vec<ExtractedContent>,
    pub total_pages: usize,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse>;
}

#[async_trait]
pub trait ContentExtractor: Send + Sync {
    async fn extract(&self, url: &str) -> Result<ExtractedContent>;
    async fn extract_many(&self, urls: &[&str]) -> Result<Vec<ExtractedContent>>;
}

#[async_trait]
pub trait WebCrawler: Send + Sync {
    async fn crawl(&self, url: &str, max_depth: u32, max_pages: u32) -> Result<CrawlResult>;
}
