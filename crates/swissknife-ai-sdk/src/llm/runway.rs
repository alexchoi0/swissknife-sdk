use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{VideoProvider, VideoRequest, VideoResponse, VideoData, ProviderConfig};

const API_BASE: &str = "https://api.runwayml.com/v1";

pub struct RunwayClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl RunwayClient {
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("X-Runway-Version", "2024-11-06")
    }

    pub async fn generate_gen3(&self, request: &Gen3Request) -> Result<TaskResponse> {
        let response = self.request(reqwest::Method::POST, "/image_to_video")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: RunwayError = response.json().await?;
            return Err(Error::Api {
                message: error.error,
                code: None,
            });
        }

        Ok(response.json().await?)
    }

    pub async fn generate_text_to_video(&self, request: &TextToVideoRequest) -> Result<TaskResponse> {
        let body = serde_json::json!({
            "model": request.model.clone().unwrap_or_else(|| "gen3a_turbo".to_string()),
            "promptText": request.prompt,
            "duration": request.duration.unwrap_or(5),
            "ratio": request.ratio.clone().unwrap_or_else(|| "1280:768".to_string()),
        });

        let response = self.request(reqwest::Method::POST, "/text_to_video")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: RunwayError = response.json().await?;
            return Err(Error::Api {
                message: error.error,
                code: None,
            });
        }

        Ok(response.json().await?)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<TaskStatus> {
        let response = self.request(reqwest::Method::GET, &format!("/tasks/{}", task_id))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: RunwayError = response.json().await?;
            return Err(Error::Api {
                message: error.error,
                code: None,
            });
        }

        Ok(response.json().await?)
    }

    pub async fn wait_for_task(&self, task_id: &str, timeout: Duration) -> Result<TaskStatus> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_secs(5);

        loop {
            if start.elapsed() > timeout {
                return Err(Error::Api {
                    message: "Task timeout".to_string(),
                    code: Some("TIMEOUT".to_string()),
                });
            }

            let status = self.get_task(task_id).await?;

            match status.status.as_str() {
                "SUCCEEDED" => return Ok(status),
                "FAILED" => return Err(Error::Api {
                    message: status.failure.unwrap_or_else(|| "Task failed".to_string()),
                    code: Some(status.failure_code.unwrap_or_default()),
                }),
                "CANCELLED" => return Err(Error::Api {
                    message: "Task cancelled".to_string(),
                    code: Some("CANCELLED".to_string()),
                }),
                _ => {
                    tokio::time::sleep(poll_interval).await;
                }
            }
        }
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let response = self.request(reqwest::Method::DELETE, &format!("/tasks/{}", task_id))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: RunwayError = response.json().await?;
            return Err(Error::Api {
                message: error.error,
                code: None,
            });
        }

        Ok(())
    }
}

#[async_trait]
impl VideoProvider for RunwayClient {
    async fn generate_video(&self, request: &VideoRequest) -> Result<VideoResponse> {
        let task = if let Some(image) = &request.image {
            let gen3_request = Gen3Request {
                model: request.model.clone(),
                prompt_image: image.clone(),
                prompt_text: Some(request.prompt.clone()),
                duration: request.duration,
                ratio: request.aspect_ratio.clone(),
                seed: request.seed,
            };
            self.generate_gen3(&gen3_request).await?
        } else {
            let text_request = TextToVideoRequest {
                model: request.model.clone(),
                prompt: request.prompt.clone(),
                duration: request.duration,
                ratio: request.aspect_ratio.clone(),
            };
            self.generate_text_to_video(&text_request).await?
        };

        Ok(VideoResponse {
            id: task.id.clone(),
            status: "pending".to_string(),
            data: vec![VideoData {
                url: None,
                task_id: Some(task.id),
            }],
        })
    }

    async fn get_video_status(&self, task_id: &str) -> Result<VideoResponse> {
        let status = self.get_task(task_id).await?;

        Ok(VideoResponse {
            id: task_id.to_string(),
            status: status.status.clone(),
            data: vec![VideoData {
                url: status.output.and_then(|o| o.first().cloned()),
                task_id: Some(task_id.to_string()),
            }],
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Gen3Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(rename = "promptImage")]
    pub prompt_image: String,
    #[serde(rename = "promptText", skip_serializing_if = "Option::is_none")]
    pub prompt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

impl Gen3Request {
    pub fn new(prompt_image: impl Into<String>) -> Self {
        Self {
            model: None,
            prompt_image: prompt_image.into(),
            prompt_text: None,
            duration: None,
            ratio: None,
            seed: None,
        }
    }

    pub fn with_prompt(mut self, text: impl Into<String>) -> Self {
        self.prompt_text = Some(text.into());
        self
    }

    pub fn with_duration(mut self, seconds: u32) -> Self {
        self.duration = Some(seconds);
        self
    }

    pub fn with_ratio(mut self, ratio: impl Into<String>) -> Self {
        self.ratio = Some(ratio.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextToVideoRequest {
    pub model: Option<String>,
    pub prompt: String,
    pub duration: Option<u32>,
    pub ratio: Option<String>,
}

impl TextToVideoRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    pub fn with_duration(mut self, seconds: u32) -> Self {
        self.duration = Some(seconds);
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskResponse {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskStatus {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub progress: Option<f32>,
    #[serde(default)]
    pub output: Option<Vec<String>>,
    #[serde(default)]
    pub failure: Option<String>,
    #[serde(default, rename = "failureCode")]
    pub failure_code: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RunwayError {
    error: String,
}

pub mod models {
    pub const GEN3A_TURBO: &str = "gen3a_turbo";
}

pub mod ratios {
    pub const LANDSCAPE_1280_768: &str = "1280:768";
    pub const PORTRAIT_768_1280: &str = "768:1280";
}
