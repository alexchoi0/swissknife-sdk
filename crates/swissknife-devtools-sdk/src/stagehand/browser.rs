use crate::{Error, Result};
use crate::stagehand::StagehandClient;
use serde::{Deserialize, Serialize};

impl StagehandClient {
    pub async fn create_session(&self, options: Option<SessionOptions>) -> Result<Session> {
        let body = options.unwrap_or_default();

        let response = self.client()
            .post(format!("{}/v1/sessions", self.base_url()))
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

        let result: Session = response.json().await?;
        Ok(result)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let response = self.client()
            .get(format!("{}/v1/sessions/{}", self.base_url(), session_id))
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

        let result: Session = response.json().await?;
        Ok(result)
    }

    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        let response = self.client()
            .get(format!("{}/v1/sessions", self.base_url()))
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

        let result: SessionsResponse = response.json().await?;
        Ok(result.sessions)
    }

    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/v1/sessions/{}", self.base_url(), session_id))
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

        Ok(())
    }

    pub async fn get_session_debug_url(&self, session_id: &str) -> Result<DebugInfo> {
        let response = self.client()
            .get(format!("{}/v1/sessions/{}/debug", self.base_url(), session_id))
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

        let result: DebugInfo = response.json().await?;
        Ok(result)
    }

    pub async fn get_session_recording(&self, session_id: &str) -> Result<Recording> {
        let response = self.client()
            .get(format!("{}/v1/sessions/{}/recording", self.base_url(), session_id))
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

        let result: Recording = response.json().await?;
        Ok(result)
    }

    pub async fn get_session_logs(&self, session_id: &str) -> Result<Vec<LogEntry>> {
        let response = self.client()
            .get(format!("{}/v1/sessions/{}/logs", self.base_url(), session_id))
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

        let result: LogsResponse = response.json().await?;
        Ok(result.logs)
    }

    pub async fn take_screenshot(&self, session_id: &str, options: Option<ScreenshotOptions>) -> Result<Screenshot> {
        let body = options.unwrap_or_default();

        let response = self.client()
            .post(format!("{}/v1/sessions/{}/screenshot", self.base_url(), session_id))
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

        let result: Screenshot = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SessionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(rename = "browserSettings", skip_serializing_if = "Option::is_none")]
    pub browser_settings: Option<BrowserSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(rename = "keepAlive", skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowserSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<Fingerprint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextSettings>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Fingerprint {
    #[serde(rename = "browserListQuery", skip_serializing_if = "Option::is_none")]
    pub browser_list_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locales: Option<Vec<String>>,
    #[serde(rename = "operatingSystems", skip_serializing_if = "Option::is_none")]
    pub operating_systems: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen: Option<ScreenConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScreenConfig {
    #[serde(rename = "maxHeight", skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,
    #[serde(rename = "maxWidth", skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,
    #[serde(rename = "minHeight", skip_serializing_if = "Option::is_none")]
    pub min_height: Option<u32>,
    #[serde(rename = "minWidth", skip_serializing_if = "Option::is_none")]
    pub min_width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persist: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: Option<String>,
    #[serde(rename = "endedAt")]
    pub ended_at: Option<String>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
    pub status: SessionStatus,
    #[serde(rename = "proxyBytes")]
    pub proxy_bytes: Option<u64>,
    #[serde(rename = "avgCpuUsage")]
    pub avg_cpu_usage: Option<f64>,
    #[serde(rename = "memoryUsage")]
    pub memory_usage: Option<u64>,
    #[serde(rename = "connectUrl")]
    pub connect_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionStatus {
    Running,
    RequestRelease,
    Releasing,
    Released,
    Error,
    New,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionsResponse {
    pub sessions: Vec<Session>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebugInfo {
    #[serde(rename = "debuggerFullscreenUrl")]
    pub debugger_fullscreen_url: Option<String>,
    #[serde(rename = "debuggerUrl")]
    pub debugger_url: Option<String>,
    #[serde(rename = "wsUrl")]
    pub ws_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Recording {
    pub id: Option<String>,
    pub url: Option<String>,
    pub duration_ms: Option<u64>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ScreenshotOptions {
    #[serde(rename = "fullPage", skip_serializing_if = "Option::is_none")]
    pub full_page: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Screenshot {
    pub data: String,
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
