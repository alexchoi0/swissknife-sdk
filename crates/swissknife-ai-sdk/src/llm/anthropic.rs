use crate::{Error, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use super::{
    ChatMessage, ChatProvider, ChatRequest, ChatResponse, ChatChoice, ChatStreamEvent,
    ChatStreamResponse, MessageContent, MessageRole, ProviderConfig, StreamDelta, Usage,
    VisionProvider, VisionRequest, VisionResponse, ContentPart,
};

const API_BASE: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl AnthropicClient {
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
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
    }
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<&'a Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: AnthropicContent,
}

#[derive(Serialize)]
#[serde(untagged)]
enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: AnthropicImageSource },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: serde_json::Value },
    #[serde(rename = "tool_result")]
    ToolResult { tool_use_id: String, content: String },
}

#[derive(Serialize)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Serialize)]
struct AnthropicTool {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    input_schema: serde_json::Value,
}

fn convert_request(request: &ChatRequest) -> AnthropicRequest {
    let mut system = None;
    let mut messages = Vec::new();

    for msg in &request.messages {
        match msg.role {
            MessageRole::System => {
                if let MessageContent::Text(text) = &msg.content {
                    system = Some(text.clone());
                }
            }
            MessageRole::User => {
                let content = match &msg.content {
                    MessageContent::Text(text) => AnthropicContent::Text(text.clone()),
                    MessageContent::Parts(parts) => {
                        let blocks: Vec<_> = parts.iter().filter_map(|p| match p {
                            super::ContentPart::Text { text } => {
                                Some(AnthropicContentBlock::Text { text: text.clone() })
                            }
                            super::ContentPart::Image { image } => {
                                if let (Some(base64), Some(media_type)) = (&image.base64, &image.media_type) {
                                    Some(AnthropicContentBlock::Image {
                                        source: AnthropicImageSource {
                                            source_type: "base64".to_string(),
                                            media_type: media_type.clone(),
                                            data: base64.clone(),
                                        }
                                    })
                                } else {
                                    None
                                }
                            }
                        }).collect();
                        AnthropicContent::Blocks(blocks)
                    }
                };
                messages.push(AnthropicMessage { role: "user".to_string(), content });
            }
            MessageRole::Assistant => {
                let content = match &msg.content {
                    MessageContent::Text(text) => {
                        if let Some(tool_calls) = &msg.tool_calls {
                            let mut blocks = vec![AnthropicContentBlock::Text { text: text.clone() }];
                            for tc in tool_calls {
                                blocks.push(AnthropicContentBlock::ToolUse {
                                    id: tc.id.clone(),
                                    name: tc.function.name.clone(),
                                    input: serde_json::from_str(&tc.function.arguments).unwrap_or_default(),
                                });
                            }
                            AnthropicContent::Blocks(blocks)
                        } else {
                            AnthropicContent::Text(text.clone())
                        }
                    }
                    MessageContent::Parts(_) => AnthropicContent::Text(String::new()),
                };
                messages.push(AnthropicMessage { role: "assistant".to_string(), content });
            }
            MessageRole::Tool => {
                if let (Some(tool_call_id), MessageContent::Text(result)) = (&msg.tool_call_id, &msg.content) {
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Blocks(vec![
                            AnthropicContentBlock::ToolResult {
                                tool_use_id: tool_call_id.clone(),
                                content: result.clone(),
                            }
                        ]),
                    });
                }
            }
        }
    }

    let tools = request.tools.as_ref().map(|tools| {
        tools.iter().map(|t| AnthropicTool {
            name: t.function.name.clone(),
            description: t.function.description.clone(),
            input_schema: t.function.parameters.clone(),
        }).collect()
    });

    AnthropicRequest {
        model: &request.model,
        max_tokens: request.max_tokens.unwrap_or(4096),
        system,
        messages,
        temperature: request.temperature,
        top_p: request.top_p,
        stop_sequences: request.stop.as_ref(),
        tools,
        stream: false,
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<AnthropicResponseContent>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: serde_json::Value },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

fn convert_response(resp: AnthropicResponse) -> ChatResponse {
    let mut text_content = String::new();
    let mut tool_calls = Vec::new();

    for content in resp.content {
        match content {
            AnthropicResponseContent::Text { text } => {
                text_content.push_str(&text);
            }
            AnthropicResponseContent::ToolUse { id, name, input } => {
                tool_calls.push(super::ToolCall {
                    id,
                    call_type: "function".to_string(),
                    function: super::FunctionCall {
                        name,
                        arguments: serde_json::to_string(&input).unwrap_or_default(),
                    },
                });
            }
        }
    }

    ChatResponse {
        id: resp.id,
        model: resp.model,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: MessageContent::Text(text_content),
                name: None,
                tool_call_id: None,
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
            },
            finish_reason: resp.stop_reason,
        }],
        usage: Some(Usage {
            prompt_tokens: resp.usage.input_tokens,
            completion_tokens: resp.usage.output_tokens,
            total_tokens: resp.usage.input_tokens + resp.usage.output_tokens,
        }),
    }
}

#[async_trait]
impl ChatProvider for AnthropicClient {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let anthropic_request = convert_request(request);

        let response = self.request(reqwest::Method::POST, "/messages")
            .json(&anthropic_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: AnthropicError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: Some(error.error.error_type)
            });
        }

        let resp: AnthropicResponse = response.json().await?;
        Ok(convert_response(resp))
    }

    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStreamResponse> {
        let mut anthropic_request = convert_request(request);
        anthropic_request.stream = true;

        let response = self.request(reqwest::Method::POST, "/messages")
            .json(&anthropic_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: AnthropicError = response.json().await?;
            return Err(Error::Api {
                message: error.error.message,
                code: Some(error.error.error_type)
            });
        }

        let stream = response.bytes_stream()
            .filter_map(|result| async move {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        parse_anthropic_sse(&text)
                    }
                    Err(e) => Some(Err(Error::Http(e))),
                }
            });

        Ok(Box::pin(stream))
    }
}

fn parse_anthropic_sse(text: &str) -> Option<Result<ChatStreamEvent>> {
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data) {
                match event.event_type.as_str() {
                    "content_block_delta" => {
                        if let Some(delta) = event.delta {
                            return Some(Ok(ChatStreamEvent {
                                id: None,
                                delta: Some(StreamDelta {
                                    role: None,
                                    content: delta.text,
                                    tool_calls: None,
                                }),
                                finish_reason: None,
                                usage: None,
                            }));
                        }
                    }
                    "message_stop" => {
                        return Some(Ok(ChatStreamEvent {
                            id: None,
                            delta: None,
                            finish_reason: Some("stop".to_string()),
                            usage: None,
                        }));
                    }
                    "message_delta" => {
                        return Some(Ok(ChatStreamEvent {
                            id: None,
                            delta: None,
                            finish_reason: event.delta.and_then(|d| d.stop_reason),
                            usage: event.usage.map(|u| Usage {
                                prompt_tokens: 0,
                                completion_tokens: u.output_tokens,
                                total_tokens: u.output_tokens,
                            }),
                        }));
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicStreamDelta>,
    usage: Option<AnthropicStreamUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    text: Option<String>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamUsage {
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

#[async_trait]
impl VisionProvider for AnthropicClient {
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
    pub const CLAUDE_OPUS_4: &str = "claude-opus-4-20250514";
    pub const CLAUDE_SONNET_4: &str = "claude-sonnet-4-20250514";
    pub const CLAUDE_3_5_SONNET: &str = "claude-3-5-sonnet-20241022";
    pub const CLAUDE_3_5_HAIKU: &str = "claude-3-5-haiku-20241022";
    pub const CLAUDE_3_OPUS: &str = "claude-3-opus-20240229";
    pub const CLAUDE_3_SONNET: &str = "claude-3-sonnet-20240229";
    pub const CLAUDE_3_HAIKU: &str = "claude-3-haiku-20240307";
}
