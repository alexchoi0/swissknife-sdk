use crate::error::Result;
use crate::tool::{get_array_param, get_bool_param, get_f64_param, get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct ChatCompletionTool;

impl Default for ChatCompletionTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ChatCompletionTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "llm_chat",
            "LLM Chat Completion",
            "Send a chat completion request to an LLM provider (OpenAI, Anthropic, HuggingFace)",
            "llm",
        )
        .with_param("api_key", ParameterSchema::string("API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: openai, anthropic, huggingface").required())
        .with_param("model", ParameterSchema::string("Model name").required())
        .with_param("messages", ParameterSchema::array("Chat messages array", ParameterSchema::json("Message")).required())
        .with_param("max_tokens", ParameterSchema::integer("Maximum tokens to generate"))
        .with_param("temperature", ParameterSchema::number("Temperature (0-2)"))
        .with_param("system", ParameterSchema::string("System message"))
        .with_output("content", OutputSchema::string("Generated response"))
        .with_output("usage", OutputSchema::json("Token usage"))
        .with_output("finish_reason", OutputSchema::string("Completion finish reason"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let model = get_required_string_param(&params, "model")?;
        let messages_json = get_array_param(&params, "messages").ok_or_else(|| crate::Error::MissingParameter("messages".into()))?;
        let max_tokens = get_i64_param(&params, "max_tokens").map(|v| v as u32);
        let temperature = get_f64_param(&params, "temperature").map(|v| v as f32);
        let system = get_string_param(&params, "system");

        use crate::llm::{ChatMessage, ChatRequest, ChatProvider, ProviderConfig, MessageContent};

        let mut messages: Vec<ChatMessage> = Vec::new();

        if let Some(sys) = system {
            messages.push(ChatMessage::system(sys));
        }

        for msg in messages_json {
            if let Some(obj) = msg.as_object() {
                let role = obj.get("role").and_then(|v| v.as_str()).unwrap_or("user");
                let content = obj.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let message = match role {
                    "system" => ChatMessage::system(content),
                    "assistant" => ChatMessage::assistant(content),
                    _ => ChatMessage::user(content),
                };
                messages.push(message);
            }
        }

        let mut request = ChatRequest::new(model, messages);
        if let Some(max) = max_tokens {
            request = request.with_max_tokens(max);
        }
        if let Some(temp) = temperature {
            request = request.with_temperature(temp);
        }

        let config = ProviderConfig::new(api_key);

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "openai")]
            "openai" => {
                use crate::llm::openai::OpenAIClient;
                let client = OpenAIClient::new(config);
                client.chat(&request).await
            }
            #[cfg(feature = "anthropic")]
            "anthropic" => {
                use crate::llm::anthropic::AnthropicClient;
                let client = AnthropicClient::new(config);
                client.chat(&request).await
            }
            #[cfg(feature = "huggingface")]
            "huggingface" => {
                use crate::llm::huggingface::HuggingFaceClient;
                let client = HuggingFaceClient::new(config);
                client.chat(&request).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported LLM provider: {}", provider)));
            }
        };

        match result {
            Ok(response) => {
                let content = response.content().map(String::from);
                let finish_reason = response.choices.first().and_then(|c| c.finish_reason.clone());
                Ok(ToolResponse::success(serde_json::json!({
                    "content": content,
                    "finish_reason": finish_reason,
                    "usage": response.usage.map(|u| serde_json::json!({
                        "prompt_tokens": u.prompt_tokens,
                        "completion_tokens": u.completion_tokens,
                        "total_tokens": u.total_tokens,
                    })),
                })))
            }
            Err(e) => Ok(ToolResponse::error(format!("Chat completion failed: {}", e))),
        }
    }
}

pub struct EmbeddingTool;

impl Default for EmbeddingTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for EmbeddingTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "llm_embed",
            "Generate Embeddings",
            "Generate embeddings for text using an LLM provider",
            "llm",
        )
        .with_param("api_key", ParameterSchema::string("API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: openai, huggingface").required())
        .with_param("model", ParameterSchema::string("Model name").required())
        .with_param("input", ParameterSchema::array("Text(s) to embed", ParameterSchema::string("Text")).required())
        .with_output("embeddings", OutputSchema::array("Generated embeddings", OutputSchema::json("Embedding")))
        .with_output("usage", OutputSchema::json("Token usage"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let model = get_required_string_param(&params, "model")?;
        let input_json = get_array_param(&params, "input").ok_or_else(|| crate::Error::MissingParameter("input".into()))?;

        let input: Vec<String> = input_json.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        use crate::llm::{EmbeddingRequest, EmbeddingProvider, ProviderConfig};

        let request = EmbeddingRequest::new(model, input);
        let config = ProviderConfig::new(api_key);

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "openai")]
            "openai" => {
                use crate::llm::openai::OpenAIClient;
                let client = OpenAIClient::new(config);
                client.embed(&request).await
            }
            #[cfg(feature = "huggingface")]
            "huggingface" => {
                use crate::llm::huggingface::HuggingFaceClient;
                let client = HuggingFaceClient::new(config);
                client.embed(&request).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported embedding provider: {}", provider)));
            }
        };

        match result {
            Ok(response) => {
                Ok(ToolResponse::success(serde_json::json!({
                    "embeddings": response.data.iter().map(|d| serde_json::json!({
                        "index": d.index,
                        "embedding": d.embedding,
                    })).collect::<Vec<_>>(),
                    "usage": response.usage.map(|u| serde_json::json!({
                        "prompt_tokens": u.prompt_tokens,
                        "total_tokens": u.total_tokens,
                    })),
                })))
            }
            Err(e) => Ok(ToolResponse::error(format!("Embedding failed: {}", e))),
        }
    }
}

pub struct ImageGenerationTool;

impl Default for ImageGenerationTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ImageGenerationTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "llm_generate_image",
            "Generate Image",
            "Generate an image using DALL-E or similar models",
            "llm",
        )
        .with_param("api_key", ParameterSchema::string("API key").required().user_only())
        .with_param("prompt", ParameterSchema::string("Image prompt").required())
        .with_param("model", ParameterSchema::string("Model: dall-e-3, dall-e-2").with_default(serde_json::json!("dall-e-3")))
        .with_param("size", ParameterSchema::string("Image size: 1024x1024, 1792x1024, 1024x1792"))
        .with_param("quality", ParameterSchema::string("Quality: standard, hd"))
        .with_param("n", ParameterSchema::integer("Number of images"))
        .with_output("images", OutputSchema::array("Generated image URLs or base64", OutputSchema::json("Image")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let prompt = get_required_string_param(&params, "prompt")?;
        let model = get_string_param(&params, "model");
        let size = get_string_param(&params, "size");
        let quality = get_string_param(&params, "quality");
        let n = get_i64_param(&params, "n").map(|v| v as u32);

        use crate::llm::{ImageRequest, ImageProvider, ProviderConfig};

        let mut request = ImageRequest::new(prompt);
        if let Some(m) = model {
            request = request.with_model(m);
        }
        if let Some(s) = size {
            request = request.with_size(s);
        }
        request.quality = quality;
        request.n = n;

        let config = ProviderConfig::new(api_key);

        #[cfg(feature = "openai")]
        {
            use crate::llm::openai::OpenAIClient;
            let client = OpenAIClient::new(config);
            match client.generate_image(&request).await {
                Ok(response) => {
                    Ok(ToolResponse::success(serde_json::json!({
                        "images": response.data.iter().map(|d| serde_json::json!({
                            "url": d.url,
                            "b64_json": d.b64_json,
                            "revised_prompt": d.revised_prompt,
                        })).collect::<Vec<_>>(),
                    })))
                }
                Err(e) => Ok(ToolResponse::error(format!("Image generation failed: {}", e))),
            }
        }
        #[cfg(not(feature = "openai"))]
        {
            let _ = (config, request);
            Ok(ToolResponse::error("OpenAI feature not enabled for image generation"))
        }
    }
}

pub struct TextToSpeechTool;

impl Default for TextToSpeechTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TextToSpeechTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "llm_text_to_speech",
            "Text to Speech",
            "Convert text to speech using OpenAI or ElevenLabs",
            "llm",
        )
        .with_param("api_key", ParameterSchema::string("API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: openai, elevenlabs").required())
        .with_param("text", ParameterSchema::string("Text to convert").required())
        .with_param("voice", ParameterSchema::string("Voice ID or name").required())
        .with_param("model", ParameterSchema::string("Model name"))
        .with_output("audio", OutputSchema::string("Base64 encoded audio"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let text = get_required_string_param(&params, "text")?;
        let voice = get_required_string_param(&params, "voice")?;
        let model = get_string_param(&params, "model").unwrap_or_else(|| {
            if provider == "elevenlabs" {
                "eleven_multilingual_v2".to_string()
            } else {
                "tts-1".to_string()
            }
        });

        use crate::llm::{TextToSpeechRequest, SpeechProvider, ProviderConfig};

        let request = TextToSpeechRequest::new(model, text, voice);
        let config = ProviderConfig::new(api_key);

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "openai")]
            "openai" => {
                use crate::llm::openai::OpenAIClient;
                let client = OpenAIClient::new(config);
                client.text_to_speech(&request).await
            }
            #[cfg(feature = "elevenlabs")]
            "elevenlabs" => {
                use crate::llm::elevenlabs::ElevenLabsClient;
                let client = ElevenLabsClient::new(config);
                client.text_to_speech(&request).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported TTS provider: {}", provider)));
            }
        };

        match result {
            Ok(audio) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&audio);
                Ok(ToolResponse::success(serde_json::json!({
                    "audio": encoded,
                    "size": audio.len(),
                })))
            }
            Err(e) => Ok(ToolResponse::error(format!("Text to speech failed: {}", e))),
        }
    }
}

pub struct SpeechToTextTool;

impl Default for SpeechToTextTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SpeechToTextTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "llm_speech_to_text",
            "Speech to Text",
            "Transcribe audio to text using OpenAI Whisper",
            "llm",
        )
        .with_param("api_key", ParameterSchema::string("OpenAI API key").required().user_only())
        .with_param("audio", ParameterSchema::string("Base64 encoded audio").required())
        .with_param("format", ParameterSchema::string("Audio format: mp3, mp4, wav, webm").with_default(serde_json::json!("mp3")))
        .with_output("text", OutputSchema::string("Transcribed text"))
        .with_output("language", OutputSchema::string("Detected language"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let audio_b64 = get_required_string_param(&params, "audio")?;
        let format_str = get_string_param(&params, "format").unwrap_or_else(|| "mp3".to_string());

        use base64::Engine;
        let audio = base64::engine::general_purpose::STANDARD
            .decode(&audio_b64)
            .map_err(|_| crate::Error::InvalidParameter("audio: invalid base64".into()))?;

        use crate::llm::{AudioFormat, SpeechProvider, ProviderConfig};

        let format = match format_str.to_lowercase().as_str() {
            "mp4" => AudioFormat::Mp4,
            "wav" => AudioFormat::Wav,
            "webm" => AudioFormat::Webm,
            "ogg" => AudioFormat::Ogg,
            "flac" => AudioFormat::Flac,
            _ => AudioFormat::Mp3,
        };

        let config = ProviderConfig::new(api_key);

        #[cfg(feature = "openai")]
        {
            use crate::llm::openai::OpenAIClient;
            let client = OpenAIClient::new(config);
            match client.speech_to_text(&audio, format).await {
                Ok(response) => {
                    Ok(ToolResponse::success(serde_json::json!({
                        "text": response.text,
                        "language": response.language,
                        "duration": response.duration,
                    })))
                }
                Err(e) => Ok(ToolResponse::error(format!("Speech to text failed: {}", e))),
            }
        }
        #[cfg(not(feature = "openai"))]
        {
            let _ = (config, audio, format);
            Ok(ToolResponse::error("OpenAI feature not enabled for speech to text"))
        }
    }
}
