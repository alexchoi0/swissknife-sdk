use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{AudioFormat, ProviderConfig, SpeechProvider, TextToSpeechRequest, TranscriptionResponse};

const API_BASE: &str = "https://api.elevenlabs.io/v1";

pub struct ElevenLabsClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl ElevenLabsClient {
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
            .header("xi-api-key", &self.api_key)
    }

    pub async fn list_voices(&self) -> Result<Vec<Voice>> {
        let response = self.request(reqwest::Method::GET, "/voices")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Api {
                message: error_text,
                code: None
            });
        }

        let voices: VoicesResponse = response.json().await?;
        Ok(voices.voices)
    }

    pub async fn text_to_speech_with_options(
        &self,
        voice_id: &str,
        text: &str,
        options: &TtsOptions,
    ) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        struct Request<'a> {
            text: &'a str,
            model_id: &'a str,
            voice_settings: Option<VoiceSettings>,
        }

        let request = Request {
            text,
            model_id: &options.model_id,
            voice_settings: options.voice_settings.clone(),
        };

        let response = self.request(reqwest::Method::POST, &format!("/text-to-speech/{}", voice_id))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Api {
                message: error_text,
                code: None
            });
        }

        Ok(response.bytes().await?.to_vec())
    }
}

#[async_trait]
impl SpeechProvider for ElevenLabsClient {
    async fn text_to_speech(&self, request: &TextToSpeechRequest) -> Result<Vec<u8>> {
        let options = TtsOptions {
            model_id: request.model.clone(),
            voice_settings: None,
        };
        self.text_to_speech_with_options(&request.voice, &request.input, &options).await
    }

    async fn speech_to_text(&self, _audio: &[u8], _format: AudioFormat) -> Result<TranscriptionResponse> {
        Err(Error::Api {
            message: "ElevenLabs does not support speech-to-text".to_string(),
            code: None
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    pub voice_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub labels: Option<std::collections::HashMap<String, String>>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VoicesResponse {
    voices: Vec<Voice>,
}

#[derive(Debug, Clone, Default)]
pub struct TtsOptions {
    pub model_id: String,
    pub voice_settings: Option<VoiceSettings>,
}

impl TtsOptions {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            voice_settings: None,
        }
    }

    pub fn with_voice_settings(mut self, settings: VoiceSettings) -> Self {
        self.voice_settings = Some(settings);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSettings {
    pub stability: f32,
    pub similarity_boost: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_speaker_boost: Option<bool>,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            stability: 0.5,
            similarity_boost: 0.75,
            style: None,
            use_speaker_boost: None,
        }
    }
}

pub mod models {
    pub const ELEVEN_MULTILINGUAL_V2: &str = "eleven_multilingual_v2";
    pub const ELEVEN_TURBO_V2_5: &str = "eleven_turbo_v2_5";
    pub const ELEVEN_TURBO_V2: &str = "eleven_turbo_v2";
    pub const ELEVEN_MONOLINGUAL_V1: &str = "eleven_monolingual_v1";
    pub const ELEVEN_ENGLISH_STS_V2: &str = "eleven_english_sts_v2";
}

pub mod voices {
    pub const RACHEL: &str = "21m00Tcm4TlvDq8ikWAM";
    pub const DREW: &str = "29vD33N1CtxCmqQRPOHJ";
    pub const CLYDE: &str = "2EiwWnXFnvU5JabPnv8n";
    pub const PAUL: &str = "5Q0t7uMcjvnagumLfvZi";
    pub const DOMI: &str = "AZnzlk1XvdvUeBnXmlld";
    pub const DAVE: &str = "CYw3kZ02Hs0563khs1Fj";
    pub const FIN: &str = "D38z5RcWu1voky8WS1ja";
    pub const SARAH: &str = "EXAVITQu4vr4xnSDxMaL";
    pub const ANTONI: &str = "ErXwobaYiN019PkySvjV";
    pub const THOMAS: &str = "GBv7mTt0atIp3Br8iCZE";
    pub const CHARLIE: &str = "IKne3meq5aSn9XLyUdCD";
    pub const GEORGE: &str = "JBFqnCBsd6RMkjVDRZzb";
    pub const EMILY: &str = "LcfcDJNUP1GQjkzn1xUU";
    pub const ELLI: &str = "MF3mGyEYCl7XYWbV9V6O";
    pub const CALLUM: &str = "N2lVS1w4EtoT3dr4eOWO";
    pub const PATRICK: &str = "ODq5zmih8GrVes37Dizd";
    pub const HARRY: &str = "SOYHLrjzK2X1ezoPC6cr";
    pub const LIAM: &str = "TX3LPaxmHKxFdv7VOQHJ";
    pub const DOROTHY: &str = "ThT5KcBeYPX3keUQqHPh";
    pub const JOSH: &str = "TxGEqnHWrfWFTfGW9XjX";
    pub const ARNOLD: &str = "VR6AewLTigWG4xSOukaG";
    pub const CHARLOTTE: &str = "XB0fDUnXU5powFXDhCwa";
    pub const MATILDA: &str = "XrExE9yKIg1WjnnlVkGX";
    pub const MATTHEW: &str = "Yko7PKs6WkbO6YZgBKFN";
    pub const JAMES: &str = "ZQe5CZNOzWyzPSCn5a3c";
    pub const JOSEPH: &str = "Zlb1dXrM653N07WRdFW3";
    pub const JEREMY: &str = "bVMeCyTHy58xNoL34h3p";
    pub const MICHAEL: &str = "flq6f7yk4E4fJM5XTYuZ";
    pub const ETHAN: &str = "g5CIjZEefAph4nQFvHAz";
    pub const GIGI: &str = "jBpfuIE2acCO8z3wKNLl";
    pub const FREYA: &str = "jsCqWAovK2LkecY7zXl4";
    pub const GRACE: &str = "oWAxZDx7w5VEj9dCyTzz";
    pub const DANIEL: &str = "onwK4e9ZLuTAKqWW03F9";
    pub const SERENA: &str = "pMsXgVXv3BLzUgSXRplE";
    pub const ADAM: &str = "pNInz6obpgDQGcFmaJgB";
    pub const NICOLE: &str = "piTKgcLEGmPE4e6mEKli";
    pub const JESSIE: &str = "t0jbNlBVZ17f02VDIeMI";
    pub const RYAN: &str = "wViXBPUzp2ZZixB1xQuM";
    pub const SAM: &str = "yoZ06aMxZJJ28mfd3POQ";
    pub const GLINDA: &str = "z9fAnlkpzviPz146aGWa";
    pub const GIOVANNI: &str = "zcAOhNBS3c14rBihAFp1";
    pub const MIMI: &str = "zrHiDhphv9ZnVXBqCLjz";
}
