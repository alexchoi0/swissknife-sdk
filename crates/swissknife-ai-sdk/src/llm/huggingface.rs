use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    ChatChoice, ChatMessage, ChatProvider, ChatRequest, ChatResponse, ChatStreamEvent,
    ChatStreamResponse, EmbeddingData, EmbeddingProvider, EmbeddingRequest, EmbeddingResponse,
    MessageContent, MessageRole, ProviderConfig, Usage,
};

const INFERENCE_API_BASE: &str = "https://api-inference.huggingface.co";

pub struct HuggingFaceClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl HuggingFaceClient {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            api_key: config.api_key,
            base_url: config.base_url.unwrap_or_else(|| INFERENCE_API_BASE.to_string()),
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
    }
}

#[derive(Serialize)]
struct HFChatRequest<'a> {
    model: &'a str,
    messages: &'a [HFMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    stream: bool,
}

#[derive(Serialize)]
struct HFMessage {
    role: String,
    content: String,
}

fn convert_messages(messages: &[ChatMessage]) -> Vec<HFMessage> {
    messages.iter().map(|m| {
        let content = match &m.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Parts(parts) => {
                parts.iter().filter_map(|p| match p {
                    super::ContentPart::Text { text } => Some(text.clone()),
                    _ => None,
                }).collect::<Vec<_>>().join("\n")
            }
        };
        HFMessage {
            role: match m.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
                MessageRole::Tool => "tool".to_string(),
            },
            content,
        }
    }).collect()
}

#[derive(Debug, Deserialize)]
struct HFChatResponse {
    id: Option<String>,
    choices: Vec<HFChoice>,
    usage: Option<HFUsage>,
}

#[derive(Debug, Deserialize)]
struct HFChoice {
    index: u32,
    message: HFResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HFResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct HFUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl ChatProvider for HuggingFaceClient {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let messages = convert_messages(&request.messages);

        let hf_request = HFChatRequest {
            model: &request.model,
            messages: &messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: false,
        };

        let response = self.request(reqwest::Method::POST, "/models")
            .json(&hf_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Api {
                message: error_text,
                code: None
            });
        }

        let hf_response: HFChatResponse = response.json().await?;

        Ok(ChatResponse {
            id: hf_response.id.unwrap_or_default(),
            model: request.model.clone(),
            choices: hf_response.choices.into_iter().map(|c| ChatChoice {
                index: c.index,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: MessageContent::Text(c.message.content),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
                finish_reason: c.finish_reason,
            }).collect(),
            usage: hf_response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    async fn chat_stream(&self, _request: &ChatRequest) -> Result<ChatStreamResponse> {
        Err(Error::Api {
            message: "Streaming not supported for HuggingFace Inference API".to_string(),
            code: None
        })
    }
}

#[derive(Serialize)]
struct HFEmbeddingRequest<'a> {
    inputs: &'a [String],
}

#[async_trait]
impl EmbeddingProvider for HuggingFaceClient {
    async fn embed(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse> {
        let hf_request = HFEmbeddingRequest {
            inputs: &request.input,
        };

        let response = self.request(reqwest::Method::POST, &format!("/models/{}", request.model))
            .json(&hf_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Api {
                message: error_text,
                code: None
            });
        }

        let embeddings: Vec<Vec<f32>> = response.json().await?;

        Ok(EmbeddingResponse {
            model: request.model.clone(),
            data: embeddings.into_iter().enumerate().map(|(i, embedding)| {
                EmbeddingData {
                    index: i as u32,
                    embedding,
                }
            }).collect(),
            usage: None,
        })
    }
}

pub mod models {
    pub const MISTRAL_7B_INSTRUCT: &str = "mistralai/Mistral-7B-Instruct-v0.3";
    pub const LLAMA_3_8B_INSTRUCT: &str = "meta-llama/Meta-Llama-3-8B-Instruct";
    pub const LLAMA_3_70B_INSTRUCT: &str = "meta-llama/Meta-Llama-3-70B-Instruct";
    pub const QWEN_2_72B_INSTRUCT: &str = "Qwen/Qwen2-72B-Instruct";
    pub const PHI_3_MINI: &str = "microsoft/Phi-3-mini-4k-instruct";
    pub const SENTENCE_TRANSFORMERS: &str = "sentence-transformers/all-MiniLM-L6-v2";
    pub const BGE_LARGE: &str = "BAAI/bge-large-en-v1.5";
}
