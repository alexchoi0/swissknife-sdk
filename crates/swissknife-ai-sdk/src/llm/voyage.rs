use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{EmbeddingData, EmbeddingProvider, EmbeddingRequest, EmbeddingResponse, ProviderConfig};
use crate::Result;

const API_BASE: &str = "https://api.voyageai.com/v1";

pub struct VoyageClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl VoyageClient {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            api_key: config.api_key,
            base_url: config.base_url.unwrap_or_else(|| API_BASE.to_string()),
            http: reqwest::Client::new(),
        }
    }

    pub fn from_api_key(api_key: impl Into<String>) -> Self {
        Self::new(ProviderConfig::new(api_key))
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
    }
}

#[derive(Debug, Serialize)]
struct VoyageEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    input_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncation: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct VoyageEmbeddingResponse {
    data: Vec<VoyageEmbeddingData>,
    model: String,
    usage: VoyageUsage,
}

#[derive(Debug, Deserialize)]
struct VoyageEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct VoyageUsage {
    total_tokens: u32,
}

#[async_trait]
impl EmbeddingProvider for VoyageClient {
    async fn embed(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse> {
        let voyage_request = VoyageEmbeddingRequest {
            model: &request.model,
            input: &request.input,
            input_type: Some("document"),
            truncation: Some(true),
        };

        let response = self
            .request(reqwest::Method::POST, "/embeddings")
            .json(&voyage_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(crate::Error::Api {
                message: format!("Voyage API error {}: {}", status, text),
                code: Some(status.to_string()),
            });
        }

        let voyage_response: VoyageEmbeddingResponse = response.json().await?;

        Ok(EmbeddingResponse {
            model: voyage_response.model,
            data: voyage_response
                .data
                .into_iter()
                .map(|d| EmbeddingData {
                    index: d.index as u32,
                    embedding: d.embedding,
                })
                .collect(),
            usage: Some(super::Usage {
                prompt_tokens: voyage_response.usage.total_tokens,
                completion_tokens: 0,
                total_tokens: voyage_response.usage.total_tokens,
            }),
        })
    }
}

pub mod models {
    pub const VOYAGE_3: &str = "voyage-3";
    pub const VOYAGE_3_LITE: &str = "voyage-3-lite";
    pub const VOYAGE_CODE_3: &str = "voyage-code-3";
    pub const VOYAGE_FINANCE_2: &str = "voyage-finance-2";
    pub const VOYAGE_LAW_2: &str = "voyage-law-2";
    pub const VOYAGE_LARGE_2_INSTRUCT: &str = "voyage-large-2-instruct";
}
