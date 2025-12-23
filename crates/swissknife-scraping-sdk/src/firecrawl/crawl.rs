use crate::{Error, Result};
use crate::firecrawl::FirecrawlClient;
use crate::firecrawl::scrape::ScrapedData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl FirecrawlClient {
    pub async fn crawl(&self, url: &str, options: Option<CrawlOptions>) -> Result<CrawlResponse> {
        let mut body = serde_json::json!({
            "url": url
        });

        if let Some(opts) = options {
            if let Some(exclude) = opts.exclude_paths {
                body["excludePaths"] = serde_json::to_value(exclude).unwrap_or_default();
            }
            if let Some(include) = opts.include_paths {
                body["includePaths"] = serde_json::to_value(include).unwrap_or_default();
            }
            if let Some(max_depth) = opts.max_depth {
                body["maxDepth"] = serde_json::Value::Number(max_depth.into());
            }
            if let Some(ignore_sitemap) = opts.ignore_sitemap {
                body["ignoreSitemap"] = serde_json::Value::Bool(ignore_sitemap);
            }
            if let Some(limit) = opts.limit {
                body["limit"] = serde_json::Value::Number(limit.into());
            }
            if let Some(allow_backward) = opts.allow_backward_links {
                body["allowBackwardLinks"] = serde_json::Value::Bool(allow_backward);
            }
            if let Some(allow_external) = opts.allow_external_links {
                body["allowExternalLinks"] = serde_json::Value::Bool(allow_external);
            }
            if let Some(webhook) = opts.webhook {
                body["webhook"] = serde_json::Value::String(webhook);
            }
            if let Some(scrape_opts) = opts.scrape_options {
                body["scrapeOptions"] = serde_json::to_value(scrape_opts).unwrap_or_default();
            }
        }

        let response = self.client()
            .post(format!("{}/v1/crawl", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
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

        let result: CrawlResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_crawl_status(&self, crawl_id: &str) -> Result<CrawlStatus> {
        let response = self.client()
            .get(format!("{}/v1/crawl/{}", self.base_url(), crawl_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let result: CrawlStatus = response.json().await?;
        Ok(result)
    }

    pub async fn cancel_crawl(&self, crawl_id: &str) -> Result<CancelCrawlResponse> {
        let response = self.client()
            .delete(format!("{}/v1/crawl/{}", self.base_url(), crawl_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let result: CancelCrawlResponse = response.json().await?;
        Ok(result)
    }

    pub async fn map(&self, url: &str, options: Option<MapOptions>) -> Result<MapResponse> {
        let mut body = serde_json::json!({
            "url": url
        });

        if let Some(opts) = options {
            if let Some(search) = opts.search {
                body["search"] = serde_json::Value::String(search);
            }
            if let Some(ignore_sitemap) = opts.ignore_sitemap {
                body["ignoreSitemap"] = serde_json::Value::Bool(ignore_sitemap);
            }
            if let Some(include_subdomains) = opts.include_subdomains {
                body["includeSubdomains"] = serde_json::Value::Bool(include_subdomains);
            }
            if let Some(limit) = opts.limit {
                body["limit"] = serde_json::Value::Number(limit.into());
            }
        }

        let response = self.client()
            .post(format!("{}/v1/map", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
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

        let result: MapResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CrawlOptions {
    pub exclude_paths: Option<Vec<String>>,
    pub include_paths: Option<Vec<String>>,
    pub max_depth: Option<u32>,
    pub ignore_sitemap: Option<bool>,
    pub limit: Option<u32>,
    pub allow_backward_links: Option<bool>,
    pub allow_external_links: Option<bool>,
    pub webhook: Option<String>,
    pub scrape_options: Option<CrawlScrapeOptions>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrawlScrapeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formats: Option<Vec<String>>,
    #[serde(rename = "onlyMainContent", skip_serializing_if = "Option::is_none")]
    pub only_main_content: Option<bool>,
    #[serde(rename = "includeTags", skip_serializing_if = "Option::is_none")]
    pub include_tags: Option<Vec<String>>,
    #[serde(rename = "excludeTags", skip_serializing_if = "Option::is_none")]
    pub exclude_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct MapOptions {
    pub search: Option<String>,
    pub ignore_sitemap: Option<bool>,
    pub include_subdomains: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrawlResponse {
    pub success: bool,
    pub id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrawlStatus {
    pub success: bool,
    pub status: String,
    pub total: Option<u32>,
    pub completed: Option<u32>,
    #[serde(rename = "creditsUsed")]
    pub credits_used: Option<u32>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
    pub data: Option<Vec<ScrapedData>>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelCrawlResponse {
    pub success: bool,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapResponse {
    pub success: bool,
    pub links: Option<Vec<String>>,
}
