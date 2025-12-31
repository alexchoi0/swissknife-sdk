use crate::{Error, Result};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use super::{
    AudioFormat, ChatProvider, ChatRequest, ChatResponse, ChatStreamEvent, ChatStreamResponse,
    CompletionProvider, CompletionRequest, CompletionResponse, EmbeddingProvider,
    EmbeddingRequest, EmbeddingResponse, ImageProvider, ImageRequest, ImageResponse,
    ProviderConfig, SpeechProvider, StreamDelta, TextToSpeechRequest, TranscriptionResponse,
    VisionProvider, VisionRequest, VisionResponse, ChatMessage, MessageRole, MessageContent,
    ContentPart, ImageContent,
};

const API_BASE: &str = "https://api.openai.com/v1";

pub struct OpenAIClient {
    api_key: String,
    base_url: String,
    organization: Option<String>,
    http: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            api_key: config.api_key,
            base_url: config.base_url.unwrap_or_else(|| API_BASE.to_string()),
            organization: config.organization,
            http: reqwest::Client::new(),
        }
    }

    pub fn from_api_key(api_key: impl Into<String>) -> Self {
        Self::new(ProviderConfig::new(api_key))
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url)
            .header("Authorization", format!("Bearer {}", self.api_key));

        if let Some(org) = &self.organization {
            req = req.header("OpenAI-Organization", org);
        }

        req
    }
}

#[async_trait]
impl ChatProvider for OpenAIClient {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let response = self.request(reqwest::Method::POST, "/chat/completions")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: OpenAIError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: error.error.code
            });
        }

        Ok(response.json().await?)
    }

    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStreamResponse> {
        #[derive(Serialize)]
        struct StreamRequest<'a> {
            #[serde(flatten)]
            request: &'a ChatRequest,
            stream: bool,
        }

        let response = self.request(reqwest::Method::POST, "/chat/completions")
            .json(&StreamRequest { request, stream: true })
            .send()
            .await?;

        if !response.status().is_success() {
            let error: OpenAIError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: error.error.code
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
            match serde_json::from_str::<OpenAIStreamChunk>(data) {
                Ok(chunk) => {
                    let delta = chunk.choices.first().map(|c| StreamDelta {
                        role: c.delta.role,
                        content: c.delta.content.clone(),
                        tool_calls: c.delta.tool_calls.clone(),
                    });
                    return Some(Ok(ChatStreamEvent {
                        id: Some(chunk.id),
                        delta,
                        finish_reason: chunk.choices.first().and_then(|c| c.finish_reason.clone()),
                        usage: chunk.usage,
                    }));
                }
                Err(e) => return Some(Err(Error::Json(e))),
            }
        }
    }
    None
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    id: String,
    choices: Vec<OpenAIStreamChoice>,
    usage: Option<super::Usage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    role: Option<super::MessageRole>,
    content: Option<String>,
    tool_calls: Option<Vec<super::ToolCallDelta>>,
}

#[async_trait]
impl CompletionProvider for OpenAIClient {
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let response = self.request(reqwest::Method::POST, "/completions")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: OpenAIError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: error.error.code
            });
        }

        Ok(response.json().await?)
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIClient {
    async fn embed(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse> {
        let response = self.request(reqwest::Method::POST, "/embeddings")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: OpenAIError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: error.error.code
            });
        }

        Ok(response.json().await?)
    }
}

#[async_trait]
impl ImageProvider for OpenAIClient {
    async fn generate_image(&self, request: &ImageRequest) -> Result<ImageResponse> {
        let response = self.request(reqwest::Method::POST, "/images/generations")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: OpenAIError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: error.error.code
            });
        }

        Ok(response.json().await?)
    }
}

#[async_trait]
impl SpeechProvider for OpenAIClient {
    async fn text_to_speech(&self, request: &TextToSpeechRequest) -> Result<Vec<u8>> {
        let response = self.request(reqwest::Method::POST, "/audio/speech")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: OpenAIError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: error.error.code
            });
        }

        Ok(response.bytes().await?.to_vec())
    }

    async fn speech_to_text(&self, audio: &[u8], format: AudioFormat) -> Result<TranscriptionResponse> {
        use reqwest::multipart::{Form, Part};

        let part = Part::bytes(audio.to_vec())
            .file_name(format!("audio.{}", format.as_str()))
            .mime_str(format.mime_type())?;

        let form = Form::new()
            .text("model", "whisper-1")
            .part("file", part);

        let response = self.request(reqwest::Method::POST, "/audio/transcriptions")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: OpenAIError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: error.error.code
            });
        }

        Ok(response.json().await?)
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIError {
    error: OpenAIErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAIErrorDetail {
    message: String,
    code: Option<String>,
}

#[async_trait]
impl VisionProvider for OpenAIClient {
    async fn analyze_image(&self, request: &VisionRequest) -> Result<VisionResponse> {
        let mut parts: Vec<ContentPart> = vec![ContentPart::Text { text: request.prompt.clone() }];

        for image in &request.images {
            parts.push(ContentPart::Image { image: image.clone() });
        }

        let chat_request = ChatRequest {
            model: request.model.clone(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: MessageContent::Parts(parts),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            }],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: None,
            stop: None,
            tools: None,
            tool_choice: None,
            response_format: None,
        };

        let response = self.chat(&chat_request).await?;
        let content = response.content().unwrap_or_default().to_string();

        Ok(VisionResponse {
            id: response.id,
            model: response.model,
            content,
            usage: response.usage,
        })
    }
}

pub mod models {
    pub const GPT_4O: &str = "gpt-4o";
    pub const GPT_4O_MINI: &str = "gpt-4o-mini";
    pub const GPT_4_TURBO: &str = "gpt-4-turbo";
    pub const GPT_4: &str = "gpt-4";
    pub const GPT_3_5_TURBO: &str = "gpt-3.5-turbo";
    pub const TEXT_EMBEDDING_3_LARGE: &str = "text-embedding-3-large";
    pub const TEXT_EMBEDDING_3_SMALL: &str = "text-embedding-3-small";
    pub const TEXT_EMBEDDING_ADA_002: &str = "text-embedding-ada-002";
    pub const DALL_E_3: &str = "dall-e-3";
    pub const DALL_E_2: &str = "dall-e-2";
    pub const TTS_1: &str = "tts-1";
    pub const TTS_1_HD: &str = "tts-1-hd";
    pub const WHISPER_1: &str = "whisper-1";
}
