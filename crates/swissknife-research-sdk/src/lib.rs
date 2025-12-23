mod error;

pub use error::{Error, Result};

#[cfg(feature = "arxiv")]
pub mod arxiv;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub summary: Option<String>,
    pub published: Option<String>,
    pub updated: Option<String>,
    pub categories: Vec<String>,
    pub primary_category: Option<String>,
    pub pdf_url: Option<String>,
    pub html_url: Option<String>,
    pub doi: Option<String>,
    pub journal_ref: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub affiliation: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub query: String,
    pub max_results: Option<u32>,
    pub start: Option<u32>,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Relevance,
    LastUpdated,
    Submitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub papers: Vec<Paper>,
    pub total_results: Option<u32>,
    pub start_index: u32,
    pub items_per_page: u32,
}

#[async_trait]
pub trait PaperSearchProvider: Send + Sync {
    async fn search(&self, params: &SearchParams) -> Result<SearchResult>;
    async fn get_paper(&self, paper_id: &str) -> Result<Paper>;
    async fn get_pdf_url(&self, paper_id: &str) -> Result<String>;
}
