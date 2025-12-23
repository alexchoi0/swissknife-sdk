use crate::{Error, Result};
use crate::firecrawl::FirecrawlClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl FirecrawlClient {
    pub async fn scrape(&self, url: &str, options: Option<ScrapeOptions>) -> Result<ScrapeResponse> {
        let mut body = serde_json::json!({
            "url": url
        });

        if let Some(opts) = options {
            if let Some(formats) = opts.formats {
                body["formats"] = serde_json::to_value(formats).unwrap_or_default();
            }
            if let Some(only_main) = opts.only_main_content {
                body["onlyMainContent"] = serde_json::Value::Bool(only_main);
            }
            if let Some(include_tags) = opts.include_tags {
                body["includeTags"] = serde_json::to_value(include_tags).unwrap_or_default();
            }
            if let Some(exclude_tags) = opts.exclude_tags {
                body["excludeTags"] = serde_json::to_value(exclude_tags).unwrap_or_default();
            }
            if let Some(headers) = opts.headers {
                body["headers"] = serde_json::to_value(headers).unwrap_or_default();
            }
            if let Some(wait_for) = opts.wait_for {
                body["waitFor"] = serde_json::Value::Number(wait_for.into());
            }
            if let Some(timeout) = opts.timeout {
                body["timeout"] = serde_json::Value::Number(timeout.into());
            }
            if let Some(actions) = opts.actions {
                body["actions"] = serde_json::to_value(actions).unwrap_or_default();
            }
            if let Some(mobile) = opts.mobile {
                body["mobile"] = serde_json::Value::Bool(mobile);
            }
            if let Some(skip_tls) = opts.skip_tls_verification {
                body["skipTlsVerification"] = serde_json::Value::Bool(skip_tls);
            }
            if let Some(remove_base64) = opts.remove_base64_images {
                body["removeBase64Images"] = serde_json::Value::Bool(remove_base64);
            }
            if let Some(location) = opts.location {
                body["location"] = serde_json::to_value(location).unwrap_or_default();
            }
        }

        let response = self.client()
            .post(format!("{}/v1/scrape", self.base_url()))
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

        let result: ScrapeResponse = response.json().await?;
        Ok(result)
    }

    pub async fn batch_scrape(&self, urls: &[&str], options: Option<ScrapeOptions>) -> Result<BatchScrapeResponse> {
        let mut body = serde_json::json!({
            "urls": urls
        });

        if let Some(opts) = options {
            if let Some(formats) = opts.formats {
                body["formats"] = serde_json::to_value(formats).unwrap_or_default();
            }
            if let Some(only_main) = opts.only_main_content {
                body["onlyMainContent"] = serde_json::Value::Bool(only_main);
            }
        }

        let response = self.client()
            .post(format!("{}/v1/batch/scrape", self.base_url()))
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

        let result: BatchScrapeResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_batch_scrape_status(&self, batch_id: &str) -> Result<BatchScrapeStatus> {
        let response = self.client()
            .get(format!("{}/v1/batch/scrape/{}", self.base_url(), batch_id))
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

        let result: BatchScrapeStatus = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScrapeOptions {
    pub formats: Option<Vec<String>>,
    pub only_main_content: Option<bool>,
    pub include_tags: Option<Vec<String>>,
    pub exclude_tags: Option<Vec<String>>,
    pub headers: Option<HashMap<String, String>>,
    pub wait_for: Option<u32>,
    pub timeout: Option<u32>,
    pub actions: Option<Vec<PageAction>>,
    pub mobile: Option<bool>,
    pub skip_tls_verification: Option<bool>,
    pub remove_base64_images: Option<bool>,
    pub location: Option<LocationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageAction {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "milliseconds", skip_serializing_if = "Option::is_none")]
    pub wait_milliseconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    pub country: Option<String>,
    pub languages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScrapeResponse {
    pub success: bool,
    pub data: Option<ScrapedData>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScrapedData {
    pub markdown: Option<String>,
    pub html: Option<String>,
    #[serde(rename = "rawHtml")]
    pub raw_html: Option<String>,
    pub links: Option<Vec<String>>,
    pub screenshot: Option<String>,
    pub metadata: Option<PageMetadata>,
    #[serde(rename = "llm_extraction")]
    pub llm_extraction: Option<serde_json::Value>,
    pub actions: Option<ActionsResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub keywords: Option<String>,
    pub robots: Option<String>,
    #[serde(rename = "ogTitle")]
    pub og_title: Option<String>,
    #[serde(rename = "ogDescription")]
    pub og_description: Option<String>,
    #[serde(rename = "ogUrl")]
    pub og_url: Option<String>,
    #[serde(rename = "ogImage")]
    pub og_image: Option<String>,
    #[serde(rename = "ogLocaleAlternate")]
    pub og_locale_alternate: Option<Vec<String>>,
    #[serde(rename = "ogSiteName")]
    pub og_site_name: Option<String>,
    #[serde(rename = "sourceURL")]
    pub source_url: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionsResult {
    pub screenshots: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchScrapeResponse {
    pub success: bool,
    pub id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchScrapeStatus {
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

impl From<ScrapedData> for crate::ScrapedPage {
    fn from(data: ScrapedData) -> Self {
        let metadata = data.metadata.as_ref();
        Self {
            url: metadata.and_then(|m| m.source_url.clone()).unwrap_or_default(),
            title: metadata.and_then(|m| m.title.clone()),
            html: data.html,
            text: None,
            markdown: data.markdown,
            metadata: std::collections::HashMap::new(),
            links: data.links.unwrap_or_default(),
            images: Vec::new(),
            scraped_at: None,
        }
    }
}
