use crate::{Error, Result};
use crate::stagehand::StagehandClient;
use serde::{Deserialize, Serialize};

impl StagehandClient {
    pub async fn navigate(&self, session_id: &str, url: &str) -> Result<NavigateResponse> {
        let body = serde_json::json!({
            "url": url
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/navigate", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: NavigateResponse = response.json().await?;
        Ok(result)
    }

    pub async fn click(&self, session_id: &str, selector: &str) -> Result<ActionResponse> {
        let body = serde_json::json!({
            "selector": selector
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/click", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn type_text(&self, session_id: &str, selector: &str, text: &str) -> Result<ActionResponse> {
        let body = serde_json::json!({
            "selector": selector,
            "text": text
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/type", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn fill(&self, session_id: &str, selector: &str, value: &str) -> Result<ActionResponse> {
        let body = serde_json::json!({
            "selector": selector,
            "value": value
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/fill", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn select(&self, session_id: &str, selector: &str, value: &str) -> Result<ActionResponse> {
        let body = serde_json::json!({
            "selector": selector,
            "value": value
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/select", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn hover(&self, session_id: &str, selector: &str) -> Result<ActionResponse> {
        let body = serde_json::json!({
            "selector": selector
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/hover", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn scroll(&self, session_id: &str, options: ScrollOptions) -> Result<ActionResponse> {
        let response = self.client()
            .post(format!("{}/v1/sessions/{}/scroll", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
            .json(&options)
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn wait_for_selector(&self, session_id: &str, selector: &str, options: Option<WaitOptions>) -> Result<ActionResponse> {
        let mut body = serde_json::json!({
            "selector": selector
        });

        if let Some(opts) = options {
            if let Some(timeout) = opts.timeout {
                body["timeout"] = serde_json::Value::Number(timeout.into());
            }
            if let Some(state) = opts.state {
                body["state"] = serde_json::Value::String(state);
            }
        }

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/wait-for-selector", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn evaluate(&self, session_id: &str, expression: &str) -> Result<EvaluateResponse> {
        let body = serde_json::json!({
            "expression": expression
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/evaluate", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: EvaluateResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_page_content(&self, session_id: &str) -> Result<PageContent> {
        let response = self.client()
            .get(format!("{}/v1/sessions/{}/content", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: PageContent = response.json().await?;
        Ok(result)
    }

    pub async fn get_element_text(&self, session_id: &str, selector: &str) -> Result<TextResponse> {
        let response = self.client()
            .get(format!("{}/v1/sessions/{}/text", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
            .query(&[("selector", selector)])
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

        let result: TextResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_element_attribute(&self, session_id: &str, selector: &str, attribute: &str) -> Result<AttributeResponse> {
        let response = self.client()
            .get(format!("{}/v1/sessions/{}/attribute", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
            .query(&[("selector", selector), ("attribute", attribute)])
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

        let result: AttributeResponse = response.json().await?;
        Ok(result)
    }

    pub async fn press_key(&self, session_id: &str, key: &str) -> Result<ActionResponse> {
        let body = serde_json::json!({
            "key": key
        });

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/press", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn act(&self, session_id: &str, instruction: &str, options: Option<ActOptions>) -> Result<ActResponse> {
        let mut body = serde_json::json!({
            "instruction": instruction
        });

        if let Some(opts) = options {
            if let Some(model) = opts.model {
                body["model"] = serde_json::Value::String(model);
            }
            if let Some(variables) = opts.variables {
                body["variables"] = serde_json::to_value(variables).unwrap_or_default();
            }
        }

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/act", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ActResponse = response.json().await?;
        Ok(result)
    }

    pub async fn extract(&self, session_id: &str, instruction: &str, schema: Option<serde_json::Value>) -> Result<ExtractResponse> {
        let mut body = serde_json::json!({
            "instruction": instruction
        });

        if let Some(s) = schema {
            body["schema"] = s;
        }

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/extract", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ExtractResponse = response.json().await?;
        Ok(result)
    }

    pub async fn observe(&self, session_id: &str, instruction: Option<&str>) -> Result<ObserveResponse> {
        let body = match instruction {
            Some(inst) => serde_json::json!({ "instruction": inst }),
            None => serde_json::json!({}),
        };

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/observe", self.base_url(), session_id))
            .header("X-BB-API-Key", self.api_key())
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

        let result: ObserveResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NavigateResponse {
    pub success: bool,
    pub url: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrollOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WaitOptions {
    pub timeout: Option<u32>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateResponse {
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageContent {
    pub html: Option<String>,
    pub text: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextResponse {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttributeResponse {
    pub value: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ActOptions {
    pub model: Option<String>,
    pub variables: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActResponse {
    pub success: bool,
    pub action: Option<String>,
    pub thought: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractResponse {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObserveResponse {
    pub actions: Vec<ObservedAction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservedAction {
    pub selector: String,
    pub description: String,
    #[serde(rename = "actionType")]
    pub action_type: Option<String>,
    pub arguments: Option<Vec<String>>,
}
