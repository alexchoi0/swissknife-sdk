mod types;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "anthropic")]
pub mod anthropic;

#[cfg(feature = "huggingface")]
pub mod huggingface;

#[cfg(feature = "elevenlabs")]
pub mod elevenlabs;

#[cfg(feature = "mistral")]
pub mod mistral;

#[cfg(feature = "stability")]
pub mod stability;

#[cfg(feature = "runway")]
pub mod runway;

#[cfg(feature = "deepl")]
pub mod deepl;

#[cfg(feature = "voyage")]
pub mod voyage;

pub use types::*;

use async_trait::async_trait;
use crate::Result;

#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStreamResponse>;
}

#[async_trait]
pub trait CompletionProvider: Send + Sync {
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse>;
}

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn generate_image(&self, request: &ImageRequest) -> Result<ImageResponse>;
}

#[async_trait]
pub trait SpeechProvider: Send + Sync {
    async fn text_to_speech(&self, request: &TextToSpeechRequest) -> Result<Vec<u8>>;
    async fn speech_to_text(&self, audio: &[u8], format: AudioFormat) -> Result<TranscriptionResponse>;
}

#[async_trait]
pub trait VideoProvider: Send + Sync {
    async fn generate_video(&self, request: &VideoRequest) -> Result<VideoResponse>;
    async fn get_video_status(&self, task_id: &str) -> Result<VideoResponse>;
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(&self, request: &TranslationRequest) -> Result<TranslationResponse>;
    async fn detect_language(&self, text: &str) -> Result<String>;
    async fn get_supported_languages(&self) -> Result<Vec<String>>;
}

#[async_trait]
pub trait VisionProvider: Send + Sync {
    async fn analyze_image(&self, request: &VisionRequest) -> Result<VisionResponse>;
}
