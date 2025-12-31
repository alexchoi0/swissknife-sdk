use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "llm")]
use crate::llm;

#[derive(Clone)]
pub struct LlmTools {
    #[cfg(feature = "openai")]
    pub openai: Option<llm::openai::OpenAIClient>,
    #[cfg(feature = "anthropic")]
    pub anthropic: Option<llm::anthropic::AnthropicClient>,
    #[cfg(feature = "mistral")]
    pub mistral: Option<llm::mistral::MistralClient>,
    #[cfg(feature = "deepl")]
    pub deepl: Option<llm::deepl::DeepLClient>,
    #[cfg(feature = "stability")]
    pub stability: Option<llm::stability::StabilityClient>,
}

impl LlmTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "openai")]
            openai: None,
            #[cfg(feature = "anthropic")]
            anthropic: None,
            #[cfg(feature = "mistral")]
            mistral: None,
            #[cfg(feature = "deepl")]
            deepl: None,
            #[cfg(feature = "stability")]
            stability: None,
        }
    }

    #[cfg(feature = "openai")]
    pub fn with_openai(mut self, client: llm::openai::OpenAIClient) -> Self {
        self.openai = Some(client);
        self
    }

    #[cfg(feature = "anthropic")]
    pub fn with_anthropic(mut self, client: llm::anthropic::AnthropicClient) -> Self {
        self.anthropic = Some(client);
        self
    }

    #[cfg(feature = "mistral")]
    pub fn with_mistral(mut self, client: llm::mistral::MistralClient) -> Self {
        self.mistral = Some(client);
        self
    }

    #[cfg(feature = "deepl")]
    pub fn with_deepl(mut self, client: llm::deepl::DeepLClient) -> Self {
        self.deepl = Some(client);
        self
    }

    #[cfg(feature = "stability")]
    pub fn with_stability(mut self, client: llm::stability::StabilityClient) -> Self {
        self.stability = Some(client);
        self
    }
}

impl Default for LlmTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChatRequest {
    pub model: String,
    pub message: String,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmbedRequest {
    pub text: String,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImageGenRequest {
    pub prompt: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TranslateRequest {
    pub text: String,
    pub target_lang: String,
    #[serde(default)]
    pub source_lang: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DetectLanguageRequest {
    pub text: String,
}

#[tool_router]
impl LlmTools {
    #[cfg(feature = "openai")]
    #[rmcp::tool(description = "Send a chat message to OpenAI GPT models")]
    pub async fn openai_chat(
        &self,
        #[rmcp::tool(aggr)] req: ChatRequest,
    ) -> Result<String, String> {
        use llm::{ChatProvider, ChatRequest as LlmChatRequest, ChatMessage};

        let client = self.openai.as_ref()
            .ok_or_else(|| "OpenAI client not configured".to_string())?;

        let mut messages = Vec::new();
        if let Some(system) = req.system {
            messages.push(ChatMessage::system(system));
        }
        messages.push(ChatMessage::user(req.message));

        let mut request = LlmChatRequest::new(req.model, messages);
        if let Some(max_tokens) = req.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }
        if let Some(temperature) = req.temperature {
            request = request.with_temperature(temperature);
        }

        let response = client.chat(&request).await
            .map_err(|e| e.to_string())?;

        Ok(response.content().unwrap_or_default())
    }

    #[cfg(feature = "openai")]
    #[rmcp::tool(description = "Generate embeddings using OpenAI models")]
    pub async fn openai_embed(
        &self,
        #[rmcp::tool(aggr)] req: EmbedRequest,
    ) -> Result<String, String> {
        use llm::{EmbeddingProvider, EmbeddingRequest};

        let client = self.openai.as_ref()
            .ok_or_else(|| "OpenAI client not configured".to_string())?;

        let model = req.model.unwrap_or_else(|| "text-embedding-3-small".to_string());
        let request = EmbeddingRequest::single(model, req.text);

        let response = client.embed(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "embedding": response.first(),
            "dimensions": response.first().map(|e| e.len())
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "openai")]
    #[rmcp::tool(description = "Generate images using DALL-E")]
    pub async fn openai_image(
        &self,
        #[rmcp::tool(aggr)] req: ImageGenRequest,
    ) -> Result<String, String> {
        use llm::{ImageProvider, ImageRequest};

        let client = self.openai.as_ref()
            .ok_or_else(|| "OpenAI client not configured".to_string())?;

        let mut request = ImageRequest::new(req.prompt);
        if let Some(model) = req.model {
            request = request.with_model(model);
        }
        if let Some(size) = req.size {
            request = request.with_size(size);
        }

        let response = client.generate_image(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "images": response.data.iter().map(|d| {
                serde_json::json!({
                    "url": d.url,
                    "revised_prompt": d.revised_prompt
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "anthropic")]
    #[rmcp::tool(description = "Send a chat message to Anthropic Claude models")]
    pub async fn anthropic_chat(
        &self,
        #[rmcp::tool(aggr)] req: ChatRequest,
    ) -> Result<String, String> {
        use llm::{ChatProvider, ChatRequest as LlmChatRequest, ChatMessage};

        let client = self.anthropic.as_ref()
            .ok_or_else(|| "Anthropic client not configured".to_string())?;

        let mut messages = Vec::new();
        if let Some(system) = req.system {
            messages.push(ChatMessage::system(system));
        }
        messages.push(ChatMessage::user(req.message));

        let mut request = LlmChatRequest::new(req.model, messages);
        if let Some(max_tokens) = req.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }
        if let Some(temperature) = req.temperature {
            request = request.with_temperature(temperature);
        }

        let response = client.chat(&request).await
            .map_err(|e| e.to_string())?;

        Ok(response.content().unwrap_or_default())
    }

    #[cfg(feature = "mistral")]
    #[rmcp::tool(description = "Send a chat message to Mistral AI models")]
    pub async fn mistral_chat(
        &self,
        #[rmcp::tool(aggr)] req: ChatRequest,
    ) -> Result<String, String> {
        use llm::{ChatProvider, ChatRequest as LlmChatRequest, ChatMessage};

        let client = self.mistral.as_ref()
            .ok_or_else(|| "Mistral client not configured".to_string())?;

        let messages = vec![ChatMessage::user(req.message)];
        let mut request = LlmChatRequest::new(req.model, messages);

        if let Some(max_tokens) = req.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }
        if let Some(temperature) = req.temperature {
            request = request.with_temperature(temperature);
        }

        let response = client.chat(&request).await
            .map_err(|e| e.to_string())?;

        Ok(response.content().unwrap_or_default())
    }

    #[cfg(feature = "mistral")]
    #[rmcp::tool(description = "Generate embeddings using Mistral")]
    pub async fn mistral_embed(
        &self,
        #[rmcp::tool(aggr)] req: EmbedRequest,
    ) -> Result<String, String> {
        use llm::{EmbeddingProvider, EmbeddingRequest};
        use llm::mistral::models;

        let client = self.mistral.as_ref()
            .ok_or_else(|| "Mistral client not configured".to_string())?;

        let request = EmbeddingRequest::single(models::MISTRAL_EMBED, req.text);
        let response = client.embed(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "embedding": response.first(),
            "dimensions": response.first().map(|e| e.len())
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "deepl")]
    #[rmcp::tool(description = "Translate text using DeepL")]
    pub async fn deepl_translate(
        &self,
        #[rmcp::tool(aggr)] req: TranslateRequest,
    ) -> Result<String, String> {
        use llm::{TranslationProvider, TranslationRequest};

        let client = self.deepl.as_ref()
            .ok_or_else(|| "DeepL client not configured".to_string())?;

        let mut request = TranslationRequest::single(req.text, req.target_lang);
        if let Some(source) = req.source_lang {
            request = request.with_source_language(source);
        }

        let response = client.translate(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "translated_text": response.first(),
            "detected_language": response.translations.first()
                .and_then(|t| t.detected_source_language.clone())
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "deepl")]
    #[rmcp::tool(description = "Detect the language of text")]
    pub async fn deepl_detect(
        &self,
        #[rmcp::tool(aggr)] req: DetectLanguageRequest,
    ) -> Result<String, String> {
        use llm::TranslationProvider;

        let client = self.deepl.as_ref()
            .ok_or_else(|| "DeepL client not configured".to_string())?;

        let language = client.detect_language(&req.text).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "detected_language": language
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "stability")]
    #[rmcp::tool(description = "Generate an image using Stability AI")]
    pub async fn stability_image(
        &self,
        #[rmcp::tool(aggr)] req: ImageGenRequest,
    ) -> Result<String, String> {
        use llm::{ImageProvider, ImageRequest};

        let client = self.stability.as_ref()
            .ok_or_else(|| "Stability client not configured".to_string())?;

        let mut request = ImageRequest::new(req.prompt);
        if let Some(model) = req.model {
            request = request.with_model(model);
        }

        let response = client.generate_image(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "images": response.data.iter().map(|d| {
                serde_json::json!({
                    "base64": d.b64_json,
                    "url": d.url
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }
}
