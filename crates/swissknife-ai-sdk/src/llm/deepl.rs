use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{TranslationProvider, TranslationRequest, TranslationResponse, ProviderConfig};

const API_BASE: &str = "https://api-free.deepl.com/v2";
const API_BASE_PRO: &str = "https://api.deepl.com/v2";

pub struct DeepLClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl DeepLClient {
    pub fn new(config: ProviderConfig) -> Self {
        let is_free = config.api_key.ends_with(":fx");
        Self {
            api_key: config.api_key,
            base_url: config.base_url.unwrap_or_else(|| {
                if is_free { API_BASE.to_string() } else { API_BASE_PRO.to_string() }
            }),
            http: reqwest::Client::new(),
        }
    }

    pub fn from_api_key(api_key: impl Into<String>) -> Self {
        Self::new(ProviderConfig::new(api_key))
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.http.request(method, &url)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .header("Content-Type", "application/json")
    }

    pub async fn translate(&self, request: &DeepLTranslateRequest) -> Result<DeepLTranslateResponse> {
        let response = self.request(reqwest::Method::POST, "/translate")
            .json(&request)
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

        Ok(response.json().await?)
    }

    pub async fn get_usage(&self) -> Result<UsageResponse> {
        let response = self.request(reqwest::Method::GET, "/usage")
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

        Ok(response.json().await?)
    }

    pub async fn get_languages(&self, lang_type: Option<&str>) -> Result<Vec<Language>> {
        let mut url = "/languages".to_string();
        if let Some(t) = lang_type {
            url = format!("{}?type={}", url, t);
        }

        let response = self.request(reqwest::Method::GET, &url)
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

        Ok(response.json().await?)
    }

    pub async fn translate_document_upload(&self, file: &[u8], filename: &str, target_lang: &str, source_lang: Option<&str>) -> Result<DocumentUploadResponse> {
        use reqwest::multipart::{Form, Part};

        let part = Part::bytes(file.to_vec())
            .file_name(filename.to_string());

        let mut form = Form::new()
            .part("file", part)
            .text("target_lang", target_lang.to_string());

        if let Some(src) = source_lang {
            form = form.text("source_lang", src.to_string());
        }

        let response = self.http
            .post(format!("{}/document", self.base_url))
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .multipart(form)
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

        Ok(response.json().await?)
    }

    pub async fn translate_document_status(&self, document_id: &str, document_key: &str) -> Result<DocumentStatusResponse> {
        let body = serde_json::json!({
            "document_key": document_key
        });

        let response = self.request(reqwest::Method::POST, &format!("/document/{}", document_id))
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

        Ok(response.json().await?)
    }

    pub async fn translate_document_download(&self, document_id: &str, document_key: &str) -> Result<Vec<u8>> {
        let body = serde_json::json!({
            "document_key": document_key
        });

        let response = self.request(reqwest::Method::POST, &format!("/document/{}/result", document_id))
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

        Ok(response.bytes().await?.to_vec())
    }

    pub async fn get_glossaries(&self) -> Result<GlossaryListResponse> {
        let response = self.request(reqwest::Method::GET, "/glossaries")
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

        Ok(response.json().await?)
    }

    pub async fn create_glossary(&self, request: &CreateGlossaryRequest) -> Result<Glossary> {
        let response = self.request(reqwest::Method::POST, "/glossaries")
            .json(&request)
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

        Ok(response.json().await?)
    }

    pub async fn delete_glossary(&self, glossary_id: &str) -> Result<()> {
        let response = self.request(reqwest::Method::DELETE, &format!("/glossaries/{}", glossary_id))
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

        Ok(())
    }
}

#[async_trait]
impl TranslationProvider for DeepLClient {
    async fn translate(&self, request: &TranslationRequest) -> Result<TranslationResponse> {
        let deepl_request = DeepLTranslateRequest {
            text: request.text.clone(),
            target_lang: request.target_language.clone(),
            source_lang: request.source_language.clone(),
            formality: request.formality.clone(),
            preserve_formatting: request.preserve_formatting,
            tag_handling: None,
            glossary_id: None,
            split_sentences: None,
        };

        let response = self.translate(&deepl_request).await?;

        Ok(TranslationResponse {
            translations: response.translations.into_iter().map(|t| super::Translation {
                text: t.text,
                detected_source_language: Some(t.detected_source_language),
            }).collect(),
        })
    }

    async fn detect_language(&self, text: &str) -> Result<String> {
        let request = DeepLTranslateRequest {
            text: vec![text.to_string()],
            target_lang: "EN".to_string(),
            source_lang: None,
            formality: None,
            preserve_formatting: None,
            tag_handling: None,
            glossary_id: None,
            split_sentences: None,
        };

        let response = self.translate(&request).await?;

        response.translations.first()
            .map(|t| t.detected_source_language.clone())
            .ok_or_else(|| Error::Api {
                message: "No translation returned".to_string(),
                code: None,
            })
    }

    async fn get_supported_languages(&self) -> Result<Vec<String>> {
        let languages = self.get_languages(Some("target")).await?;
        Ok(languages.into_iter().map(|l| l.language).collect())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeepLTranslateRequest {
    pub text: Vec<String>,
    pub target_lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_formatting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_handling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossary_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_sentences: Option<String>,
}

impl DeepLTranslateRequest {
    pub fn new(text: Vec<String>, target_lang: impl Into<String>) -> Self {
        Self {
            text,
            target_lang: target_lang.into(),
            source_lang: None,
            formality: None,
            preserve_formatting: None,
            tag_handling: None,
            glossary_id: None,
            split_sentences: None,
        }
    }

    pub fn single(text: impl Into<String>, target_lang: impl Into<String>) -> Self {
        Self::new(vec![text.into()], target_lang)
    }

    pub fn with_source_lang(mut self, lang: impl Into<String>) -> Self {
        self.source_lang = Some(lang.into());
        self
    }

    pub fn with_formality(mut self, formality: impl Into<String>) -> Self {
        self.formality = Some(formality.into());
        self
    }

    pub fn with_glossary(mut self, glossary_id: impl Into<String>) -> Self {
        self.glossary_id = Some(glossary_id.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeepLTranslateResponse {
    pub translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeepLTranslation {
    pub text: String,
    pub detected_source_language: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageResponse {
    pub character_count: u64,
    pub character_limit: u64,
    #[serde(default)]
    pub document_count: Option<u64>,
    #[serde(default)]
    pub document_limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Language {
    pub language: String,
    pub name: String,
    #[serde(default)]
    pub supports_formality: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentUploadResponse {
    pub document_id: String,
    pub document_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentStatusResponse {
    pub document_id: String,
    pub status: String,
    #[serde(default)]
    pub seconds_remaining: Option<u64>,
    #[serde(default)]
    pub billed_characters: Option<u64>,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateGlossaryRequest {
    pub name: String,
    pub source_lang: String,
    pub target_lang: String,
    pub entries: String,
    pub entries_format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlossaryListResponse {
    pub glossaries: Vec<Glossary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Glossary {
    pub glossary_id: String,
    pub name: String,
    pub ready: bool,
    pub source_lang: String,
    pub target_lang: String,
    pub creation_time: String,
    pub entry_count: u64,
}

pub mod languages {
    pub const BULGARIAN: &str = "BG";
    pub const CZECH: &str = "CS";
    pub const DANISH: &str = "DA";
    pub const GERMAN: &str = "DE";
    pub const GREEK: &str = "EL";
    pub const ENGLISH: &str = "EN";
    pub const ENGLISH_GB: &str = "EN-GB";
    pub const ENGLISH_US: &str = "EN-US";
    pub const SPANISH: &str = "ES";
    pub const ESTONIAN: &str = "ET";
    pub const FINNISH: &str = "FI";
    pub const FRENCH: &str = "FR";
    pub const HUNGARIAN: &str = "HU";
    pub const INDONESIAN: &str = "ID";
    pub const ITALIAN: &str = "IT";
    pub const JAPANESE: &str = "JA";
    pub const KOREAN: &str = "KO";
    pub const LITHUANIAN: &str = "LT";
    pub const LATVIAN: &str = "LV";
    pub const NORWEGIAN: &str = "NB";
    pub const DUTCH: &str = "NL";
    pub const POLISH: &str = "PL";
    pub const PORTUGUESE: &str = "PT";
    pub const PORTUGUESE_BR: &str = "PT-BR";
    pub const PORTUGUESE_PT: &str = "PT-PT";
    pub const ROMANIAN: &str = "RO";
    pub const RUSSIAN: &str = "RU";
    pub const SLOVAK: &str = "SK";
    pub const SLOVENIAN: &str = "SL";
    pub const SWEDISH: &str = "SV";
    pub const TURKISH: &str = "TR";
    pub const UKRAINIAN: &str = "UK";
    pub const CHINESE: &str = "ZH";
}

pub mod formality {
    pub const DEFAULT: &str = "default";
    pub const MORE: &str = "more";
    pub const LESS: &str = "less";
    pub const PREFER_MORE: &str = "prefer_more";
    pub const PREFER_LESS: &str = "prefer_less";
}
