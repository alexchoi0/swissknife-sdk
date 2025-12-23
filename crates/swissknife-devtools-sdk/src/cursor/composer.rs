use crate::{Error, Result};
use crate::cursor::CursorClient;
use serde::{Deserialize, Serialize};

impl CursorClient {
    pub async fn send_composer_message(&self, request: ComposerRequest) -> Result<ComposerResponse> {
        let response = self.client()
            .post(format!("{}/composer/message", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let result: ComposerResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_composer_history(&self, conversation_id: Option<&str>) -> Result<Vec<ComposerMessage>> {
        let mut request = self.client()
            .get(format!("{}/composer/history", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        if let Some(id) = conversation_id {
            request = request.query(&[("conversation_id", id)]);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ComposerHistoryResponse = response.json().await?;
        Ok(result.messages)
    }

    pub async fn clear_composer_context(&self) -> Result<()> {
        let response = self.client()
            .post(format!("{}/composer/clear", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

    pub async fn accept_suggestion(&self, suggestion_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "suggestion_id": suggestion_id
        });

        let response = self.client()
            .post(format!("{}/composer/accept", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        Ok(())
    }

    pub async fn reject_suggestion(&self, suggestion_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "suggestion_id": suggestion_id
        });

        let response = self.client()
            .post(format!("{}/composer/reject", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        Ok(())
    }

    pub async fn request_edit(&self, request: EditRequest) -> Result<EditResponse> {
        let response = self.client()
            .post(format!("{}/composer/edit", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
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

        let result: EditResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposerRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ComposerContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposerContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<SelectionContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileContext {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectionContext {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposerResponse {
    pub id: String,
    pub conversation_id: String,
    pub message: ComposerMessage,
    pub suggestions: Option<Vec<CodeSuggestion>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposerMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeSuggestion {
    pub id: String,
    pub file_path: String,
    pub language: Option<String>,
    pub original_code: Option<String>,
    pub suggested_code: String,
    pub description: Option<String>,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposerHistoryResponse {
    pub messages: Vec<ComposerMessage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EditRequest {
    pub file_path: String,
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<SelectionContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EditResponse {
    pub id: String,
    pub file_path: String,
    pub original_content: String,
    pub edited_content: String,
    pub diff: Option<String>,
    pub suggestions: Vec<CodeSuggestion>,
}
