use crate::{Error, Result};
use crate::browseruse::BrowserUseClient;
use serde::{Deserialize, Serialize};

impl BrowserUseClient {
    pub async fn create_browser(&self, options: Option<BrowserOptions>) -> Result<Browser> {
        let body = options.unwrap_or_default();

        let response = self.client()
            .post(format!("{}/v1/browsers", self.base_url()))
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

        let result: Browser = response.json().await?;
        Ok(result)
    }

    pub async fn get_browser(&self, browser_id: &str) -> Result<Browser> {
        let response = self.client()
            .get(format!("{}/v1/browsers/{}", self.base_url(), browser_id))
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

        let result: Browser = response.json().await?;
        Ok(result)
    }

    pub async fn list_browsers(&self) -> Result<Vec<Browser>> {
        let response = self.client()
            .get(format!("{}/v1/browsers", self.base_url()))
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

        let result: BrowsersResponse = response.json().await?;
        Ok(result.browsers)
    }

    pub async fn close_browser(&self, browser_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/v1/browsers/{}", self.base_url(), browser_id))
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

    pub async fn navigate(&self, browser_id: &str, url: &str) -> Result<PageInfo> {
        let body = serde_json::json!({
            "url": url
        });

        let response = self.client()
            .post(format!("{}/v1/browsers/{}/navigate", self.base_url(), browser_id))
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

        let result: PageInfo = response.json().await?;
        Ok(result)
    }

    pub async fn get_page_content(&self, browser_id: &str) -> Result<PageContent> {
        let response = self.client()
            .get(format!("{}/v1/browsers/{}/content", self.base_url(), browser_id))
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

        let result: PageContent = response.json().await?;
        Ok(result)
    }

    pub async fn take_screenshot(&self, browser_id: &str, options: Option<ScreenshotOptions>) -> Result<Screenshot> {
        let body = options.unwrap_or_default();

        let response = self.client()
            .post(format!("{}/v1/browsers/{}/screenshot", self.base_url(), browser_id))
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

        let result: Screenshot = response.json().await?;
        Ok(result)
    }

    pub async fn click(&self, browser_id: &str, selector: &str) -> Result<ActionResult> {
        let body = serde_json::json!({
            "selector": selector
        });

        let response = self.client()
            .post(format!("{}/v1/browsers/{}/click", self.base_url(), browser_id))
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

        let result: ActionResult = response.json().await?;
        Ok(result)
    }

    pub async fn type_text(&self, browser_id: &str, selector: &str, text: &str) -> Result<ActionResult> {
        let body = serde_json::json!({
            "selector": selector,
            "text": text
        });

        let response = self.client()
            .post(format!("{}/v1/browsers/{}/type", self.base_url(), browser_id))
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

        let result: ActionResult = response.json().await?;
        Ok(result)
    }

    pub async fn scroll(&self, browser_id: &str, direction: &str, amount: Option<i32>) -> Result<ActionResult> {
        let mut body = serde_json::json!({
            "direction": direction
        });
        if let Some(amt) = amount {
            body["amount"] = serde_json::Value::Number(amt.into());
        }

        let response = self.client()
            .post(format!("{}/v1/browsers/{}/scroll", self.base_url(), browser_id))
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

        let result: ActionResult = response.json().await?;
        Ok(result)
    }

    pub async fn wait_for_selector(&self, browser_id: &str, selector: &str, timeout_ms: Option<u32>) -> Result<ActionResult> {
        let mut body = serde_json::json!({
            "selector": selector
        });
        if let Some(timeout) = timeout_ms {
            body["timeout"] = serde_json::Value::Number(timeout.into());
        }

        let response = self.client()
            .post(format!("{}/v1/browsers/{}/wait", self.base_url(), browser_id))
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

        let result: ActionResult = response.json().await?;
        Ok(result)
    }

    pub async fn execute_script(&self, browser_id: &str, script: &str) -> Result<ScriptResult> {
        let body = serde_json::json!({
            "script": script
        });

        let response = self.client()
            .post(format!("{}/v1/browsers/{}/execute", self.base_url(), browser_id))
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

        let result: ScriptResult = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BrowserOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headless: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProxyConfig {
    pub server: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Browser {
    pub id: String,
    pub status: String,
    pub created_at: Option<String>,
    pub viewport: Option<Viewport>,
    pub current_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrowsersResponse {
    pub browsers: Vec<Browser>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageInfo {
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageContent {
    pub html: Option<String>,
    pub text: Option<String>,
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ScreenshotOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_page: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Screenshot {
    pub data: String,
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptResult {
    pub result: serde_json::Value,
}
