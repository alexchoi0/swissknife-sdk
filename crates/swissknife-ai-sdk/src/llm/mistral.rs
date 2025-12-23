use crate::{Error, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use super::{
    ChatProvider, ChatRequest, ChatResponse, ChatStreamEvent, ChatStreamResponse,
    EmbeddingProvider, EmbeddingRequest, EmbeddingResponse, ProviderConfig, StreamDelta,
};

const API_BASE: &str = "https://api.mistral.ai/v1";

pub struct MistralClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl MistralClient {
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
        let url = format!("{}{}", self.base_url, path);
        self.http.request(method, &url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
    }
}

#[async_trait]
impl ChatProvider for MistralClient {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let mistral_request = MistralChatRequest::from(request);

        let response = self.request(reqwest::Method::POST, "/chat/completions")
            .json(&mistral_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: MistralError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.error_type,
            });
        }

        Ok(response.json().await?)
    }

    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStreamResponse> {
        let mut mistral_request = MistralChatRequest::from(request);
        mistral_request.stream = Some(true);

        let response = self.request(reqwest::Method::POST, "/chat/completions")
            .json(&mistral_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: MistralError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.error_type,
            });
        }

        let stream = response.bytes_stream()
            .filter_map(|result| async move {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        parse_sse_events(&text)
                    }
                    Err(e) => Some(Err(Error::Http(e))),
                }
            });

        Ok(Box::pin(stream))
    }
}

fn parse_sse_events(text: &str) -> Option<Result<ChatStreamEvent>> {
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if data.trim() == "[DONE]" {
                return None;
            }
            match serde_json::from_str::<MistralStreamChunk>(data) {
                Ok(chunk) => {
                    let delta = chunk.choices.first().map(|c| StreamDelta {
                        role: c.delta.role,
                        content: c.delta.content.clone(),
                        tool_calls: None,
                    });
                    return Some(Ok(ChatStreamEvent {
                        id: Some(chunk.id),
                        delta,
                        finish_reason: chunk.choices.first().and_then(|c| c.finish_reason.clone()),
                        usage: chunk.usage.map(|u| super::Usage {
                            prompt_tokens: u.prompt_tokens,
                            completion_tokens: u.completion_tokens,
                            total_tokens: u.total_tokens,
                        }),
                    }));
                }
                Err(e) => return Some(Err(Error::Json(e))),
            }
        }
    }
    None
}

#[async_trait]
impl EmbeddingProvider for MistralClient {
    async fn embed(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse> {
        let mistral_request = MistralEmbeddingRequest {
            model: request.model.clone(),
            input: request.input.clone(),
            encoding_format: request.encoding_format.clone(),
        };

        let response = self.request(reqwest::Method::POST, "/embeddings")
            .json(&mistral_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: MistralError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.error_type,
            });
        }

        Ok(response.json().await?)
    }
}

#[derive(Debug, Serialize)]
struct MistralChatRequest {
    model: String,
    messages: Vec<MistralMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safe_prompt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    random_seed: Option<u64>,
}

impl From<&ChatRequest> for MistralChatRequest {
    fn from(request: &ChatRequest) -> Self {
        Self {
            model: request.model.clone(),
            messages: request.messages.iter().map(|m| MistralMessage {
                role: match m.role {
                    super::MessageRole::System => "system".to_string(),
                    super::MessageRole::User => "user".to_string(),
                    super::MessageRole::Assistant => "assistant".to_string(),
                    super::MessageRole::Tool => "tool".to_string(),
                },
                content: match &m.content {
                    super::MessageContent::Text(s) => s.clone(),
                    super::MessageContent::Parts(parts) => {
                        parts.iter()
                            .filter_map(|p| match p {
                                super::ContentPart::Text { text } => Some(text.clone()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                },
            }).collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: None,
            safe_prompt: None,
            random_seed: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MistralMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct MistralEmbeddingRequest {
    model: String,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MistralStreamChunk {
    id: String,
    choices: Vec<MistralStreamChoice>,
    usage: Option<MistralUsage>,
}

#[derive(Debug, Deserialize)]
struct MistralStreamChoice {
    delta: MistralStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MistralStreamDelta {
    role: Option<super::MessageRole>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MistralUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct MistralError {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

pub mod models {
    pub const MISTRAL_LARGE: &str = "mistral-large-latest";
    pub const MISTRAL_MEDIUM: &str = "mistral-medium-latest";
    pub const MISTRAL_SMALL: &str = "mistral-small-latest";
    pub const MISTRAL_TINY: &str = "open-mistral-7b";
    pub const MISTRAL_NEMO: &str = "open-mistral-nemo";
    pub const CODESTRAL: &str = "codestral-latest";
    pub const MISTRAL_EMBED: &str = "mistral-embed";
}
