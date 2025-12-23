use crate::{Error, Result, WebCrawler, CrawlResult, ExtractedContent, ContentExtractor};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.apify.com/v2";

pub struct ApifyClient {
    api_key: String,
    client: Client,
}

impl ApifyClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn run_actor(&self, actor_id: &str, input: serde_json::Value) -> Result<ActorRun> {
        let response = self.client
            .post(format!("{}/acts/{}/runs", BASE_URL, actor_id))
            .query(&[("token", &self.api_key)])
            .json(&input)
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

        let run: ApifyRunResponse = response.json().await?;
        Ok(ActorRun {
            id: run.data.id,
            status: run.data.status,
            default_dataset_id: run.data.default_dataset_id,
        })
    }

    pub async fn get_run_status(&self, run_id: &str) -> Result<ActorRun> {
        let response = self.client
            .get(format!("{}/actor-runs/{}", BASE_URL, run_id))
            .query(&[("token", &self.api_key)])
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

        let run: ApifyRunResponse = response.json().await?;
        Ok(ActorRun {
            id: run.data.id,
            status: run.data.status,
            default_dataset_id: run.data.default_dataset_id,
        })
    }

    pub fn is_run_finished(status: &str) -> bool {
        matches!(status, "SUCCEEDED" | "FAILED" | "ABORTED" | "TIMED-OUT")
    }

    pub fn is_run_successful(status: &str) -> bool {
        status == "SUCCEEDED"
    }

    pub async fn get_dataset_items<T: for<'de> Deserialize<'de>>(&self, dataset_id: &str) -> Result<Vec<T>> {
        let response = self.client
            .get(format!("{}/datasets/{}/items", BASE_URL, dataset_id))
            .query(&[("token", &self.api_key)])
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

        let items: Vec<T> = response.json().await?;
        Ok(items)
    }

    pub async fn scrape_urls(&self, urls: Vec<String>) -> Result<ScrapeJob> {
        let input = serde_json::json!({
            "startUrls": urls.iter().map(|u| serde_json::json!({"url": u})).collect::<Vec<_>>(),
            "maxCrawlPages": urls.len(),
            "crawlerType": "playwright:firefox"
        });

        let run = self.run_actor("apify~web-scraper", input).await?;
        Ok(ScrapeJob {
            run_id: run.id,
            status: run.status,
            dataset_id: run.default_dataset_id,
        })
    }

    pub async fn get_scrape_results(&self, dataset_id: &str) -> Result<Vec<ScrapedPage>> {
        self.get_dataset_items(dataset_id).await
    }
}

#[derive(Debug, Clone)]
pub struct ScrapeJob {
    pub run_id: String,
    pub status: String,
    pub dataset_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActorRun {
    pub id: String,
    pub status: String,
    pub default_dataset_id: Option<String>,
}

#[derive(Deserialize)]
struct ApifyRunResponse {
    data: ApifyRunData,
}

#[derive(Deserialize)]
struct ApifyRunData {
    id: String,
    status: String,
    #[serde(rename = "defaultDatasetId")]
    default_dataset_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScrapedPage {
    pub url: String,
    #[serde(default)]
    pub page_title: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
}

#[async_trait]
impl ContentExtractor for ApifyClient {
    async fn extract(&self, url: &str) -> Result<ExtractedContent> {
        let job = self.scrape_urls(vec![url.to_string()]).await?;

        if !Self::is_run_successful(&job.status) {
            return Err(Error::Api {
                message: format!("Scrape job started with run_id: {}. Poll get_run_status() and then get_scrape_results().", job.run_id),
                code: Some("PENDING".to_string()),
            });
        }

        let dataset_id = job.dataset_id
            .ok_or_else(|| Error::Api { message: "No dataset ID".to_string(), code: None })?;

        let pages = self.get_scrape_results(&dataset_id).await?;
        pages.into_iter().next()
            .map(|p| ExtractedContent {
                url: p.url,
                title: p.page_title,
                content: p.text.clone().unwrap_or_default(),
                markdown: p.text,
                links: vec![],
                images: vec![],
            })
            .ok_or_else(|| Error::NotFound(url.to_string()))
    }

    async fn extract_many(&self, urls: &[&str]) -> Result<Vec<ExtractedContent>> {
        let job = self.scrape_urls(urls.iter().map(|s| s.to_string()).collect()).await?;

        if !Self::is_run_successful(&job.status) {
            return Err(Error::Api {
                message: format!("Scrape job started with run_id: {}. Poll get_run_status() and then get_scrape_results().", job.run_id),
                code: Some("PENDING".to_string()),
            });
        }

        let dataset_id = job.dataset_id
            .ok_or_else(|| Error::Api { message: "No dataset ID".to_string(), code: None })?;

        let pages = self.get_scrape_results(&dataset_id).await?;
        Ok(pages.into_iter().map(|p| ExtractedContent {
            url: p.url,
            title: p.page_title,
            content: p.text.clone().unwrap_or_default(),
            markdown: p.text,
            links: vec![],
            images: vec![],
        }).collect())
    }
}

#[async_trait]
impl WebCrawler for ApifyClient {
    async fn crawl(&self, url: &str, max_depth: u32, max_pages: u32) -> Result<CrawlResult> {
        let input = serde_json::json!({
            "startUrls": [{"url": url}],
            "maxCrawlDepth": max_depth,
            "maxCrawlPages": max_pages,
            "crawlerType": "playwright:firefox"
        });

        let run = self.run_actor("apify~web-scraper", input).await?;

        if !Self::is_run_successful(&run.status) {
            return Err(Error::Api {
                message: format!("Crawl job started with run_id: {}. Poll get_run_status() for completion.", run.id),
                code: Some("PENDING".to_string()),
            });
        }

        let dataset_id = run.default_dataset_id
            .ok_or_else(|| Error::Api { message: "No dataset ID".to_string(), code: None })?;

        let pages: Vec<ScrapedPage> = self.get_dataset_items(&dataset_id).await?;

        let extracted: Vec<ExtractedContent> = pages.into_iter().map(|p| ExtractedContent {
            url: p.url,
            title: p.page_title,
            content: p.text.clone().unwrap_or_default(),
            markdown: p.text,
            links: vec![],
            images: vec![],
        }).collect();

        let total_pages = extracted.len();
        Ok(CrawlResult {
            base_url: url.to_string(),
            pages: extracted,
            total_pages,
        })
    }
}
