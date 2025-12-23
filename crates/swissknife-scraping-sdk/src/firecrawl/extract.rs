use crate::{Error, Result};
use crate::firecrawl::FirecrawlClient;
use serde::Deserialize;

impl FirecrawlClient {
    pub async fn extract(&self, urls: &[&str], options: ExtractOptions) -> Result<ExtractResponse> {
        let mut body = serde_json::json!({
            "urls": urls
        });

        if let Some(prompt) = options.prompt {
            body["prompt"] = serde_json::Value::String(prompt);
        }
        if let Some(schema) = options.schema {
            body["schema"] = schema;
        }
        if let Some(enable_web_search) = options.enable_web_search {
            body["enableWebSearch"] = serde_json::Value::Bool(enable_web_search);
        }
        if let Some(ignore_sitemap) = options.ignore_sitemap {
            body["ignoreSitemap"] = serde_json::Value::Bool(ignore_sitemap);
        }
        if let Some(include_subdomains) = options.include_subdomains {
            body["includeSubdomains"] = serde_json::Value::Bool(include_subdomains);
        }
        if let Some(allow_external) = options.allow_external_links {
            body["allowExternalLinks"] = serde_json::Value::Bool(allow_external);
        }

        let response = self.client()
            .post(format!("{}/v1/extract", self.base_url()))
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

        let result: ExtractResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_extract_status(&self, extract_id: &str) -> Result<ExtractStatus> {
        let response = self.client()
            .get(format!("{}/v1/extract/{}", self.base_url(), extract_id))
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

        let result: ExtractStatus = response.json().await?;
        Ok(result)
    }

    pub async fn extract_sync(&self, urls: &[&str], options: ExtractOptions) -> Result<ExtractSyncResponse> {
        let mut body = serde_json::json!({
            "urls": urls
        });

        if let Some(prompt) = options.prompt {
            body["prompt"] = serde_json::Value::String(prompt);
        }
        if let Some(schema) = options.schema {
            body["schema"] = schema;
        }

        let response = self.client()
            .post(format!("{}/v1/extract/sync", self.base_url()))
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

        let result: ExtractSyncResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    pub prompt: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub enable_web_search: Option<bool>,
    pub ignore_sitemap: Option<bool>,
    pub include_subdomains: Option<bool>,
    pub allow_external_links: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractResponse {
    pub success: bool,
    pub id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractStatus {
    pub success: bool,
    pub status: String,
    pub data: Option<serde_json::Value>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractSyncResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub warning: Option<String>,
}
