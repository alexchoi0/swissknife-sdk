use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{ImageProvider, ImageRequest, ImageResponse, ImageData, ProviderConfig};

const API_BASE: &str = "https://api.stability.ai/v2beta";

pub struct StabilityClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl StabilityClient {
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
            .header("Accept", "application/json")
    }

    pub async fn generate_sd3(&self, request: &SD3Request) -> Result<ImageResponse> {
        use reqwest::multipart::{Form, Part};

        let mut form = Form::new()
            .text("prompt", request.prompt.clone())
            .text("output_format", request.output_format.clone().unwrap_or_else(|| "png".to_string()));

        if let Some(model) = &request.model {
            form = form.text("model", model.clone());
        }
        if let Some(negative) = &request.negative_prompt {
            form = form.text("negative_prompt", negative.clone());
        }
        if let Some(ar) = &request.aspect_ratio {
            form = form.text("aspect_ratio", ar.clone());
        }
        if let Some(seed) = request.seed {
            form = form.text("seed", seed.to_string());
        }

        let response = self.request(reqwest::Method::POST, "/stable-image/generate/sd3")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: StabilityError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.name,
            });
        }

        let result: StabilityImageResponse = response.json().await?;
        Ok(ImageResponse {
            created: 0,
            data: vec![ImageData {
                url: None,
                b64_json: Some(result.image),
                revised_prompt: None,
            }],
        })
    }

    pub async fn generate_ultra(&self, request: &UltraRequest) -> Result<ImageResponse> {
        use reqwest::multipart::{Form, Part};

        let mut form = Form::new()
            .text("prompt", request.prompt.clone())
            .text("output_format", request.output_format.clone().unwrap_or_else(|| "png".to_string()));

        if let Some(negative) = &request.negative_prompt {
            form = form.text("negative_prompt", negative.clone());
        }
        if let Some(ar) = &request.aspect_ratio {
            form = form.text("aspect_ratio", ar.clone());
        }
        if let Some(seed) = request.seed {
            form = form.text("seed", seed.to_string());
        }

        let response = self.request(reqwest::Method::POST, "/stable-image/generate/ultra")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: StabilityError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.name,
            });
        }

        let result: StabilityImageResponse = response.json().await?;
        Ok(ImageResponse {
            created: 0,
            data: vec![ImageData {
                url: None,
                b64_json: Some(result.image),
                revised_prompt: None,
            }],
        })
    }

    pub async fn upscale(&self, image: &[u8], prompt: Option<&str>) -> Result<ImageResponse> {
        use reqwest::multipart::{Form, Part};

        let part = Part::bytes(image.to_vec())
            .file_name("image.png")
            .mime_str("image/png")?;

        let mut form = Form::new()
            .part("image", part)
            .text("output_format", "png");

        if let Some(p) = prompt {
            form = form.text("prompt", p.to_string());
        }

        let response = self.request(reqwest::Method::POST, "/stable-image/upscale/conservative")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: StabilityError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.name,
            });
        }

        let result: StabilityImageResponse = response.json().await?;
        Ok(ImageResponse {
            created: 0,
            data: vec![ImageData {
                url: None,
                b64_json: Some(result.image),
                revised_prompt: None,
            }],
        })
    }

    pub async fn image_to_image(&self, image: &[u8], prompt: &str, strength: Option<f32>) -> Result<ImageResponse> {
        use reqwest::multipart::{Form, Part};

        let part = Part::bytes(image.to_vec())
            .file_name("image.png")
            .mime_str("image/png")?;

        let mut form = Form::new()
            .part("image", part)
            .text("prompt", prompt.to_string())
            .text("output_format", "png");

        if let Some(s) = strength {
            form = form.text("strength", s.to_string());
        }

        let response = self.request(reqwest::Method::POST, "/stable-image/generate/sd3")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: StabilityError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.name,
            });
        }

        let result: StabilityImageResponse = response.json().await?;
        Ok(ImageResponse {
            created: 0,
            data: vec![ImageData {
                url: None,
                b64_json: Some(result.image),
                revised_prompt: None,
            }],
        })
    }

    pub async fn remove_background(&self, image: &[u8]) -> Result<ImageResponse> {
        use reqwest::multipart::{Form, Part};

        let part = Part::bytes(image.to_vec())
            .file_name("image.png")
            .mime_str("image/png")?;

        let form = Form::new()
            .part("image", part)
            .text("output_format", "png");

        let response = self.request(reqwest::Method::POST, "/stable-image/edit/remove-background")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: StabilityError = response.json().await?;
            return Err(Error::Api {
                message: error.message,
                code: error.name,
            });
        }

        let result: StabilityImageResponse = response.json().await?;
        Ok(ImageResponse {
            created: 0,
            data: vec![ImageData {
                url: None,
                b64_json: Some(result.image),
                revised_prompt: None,
            }],
        })
    }
}

#[async_trait]
impl ImageProvider for StabilityClient {
    async fn generate_image(&self, request: &ImageRequest) -> Result<ImageResponse> {
        let sd3_request = SD3Request {
            prompt: request.prompt.clone(),
            model: request.model.clone(),
            negative_prompt: None,
            aspect_ratio: request.size.clone(),
            seed: None,
            output_format: request.response_format.clone(),
        };
        self.generate_sd3(&sd3_request).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct SD3Request {
    pub prompt: String,
    pub model: Option<String>,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: Option<String>,
    pub seed: Option<u64>,
    pub output_format: Option<String>,
}

impl SD3Request {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_aspect_ratio(mut self, ar: impl Into<String>) -> Self {
        self.aspect_ratio = Some(ar.into());
        self
    }

    pub fn with_negative_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.negative_prompt = Some(prompt.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct UltraRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: Option<String>,
    pub seed: Option<u64>,
    pub output_format: Option<String>,
}

impl UltraRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    pub fn with_aspect_ratio(mut self, ar: impl Into<String>) -> Self {
        self.aspect_ratio = Some(ar.into());
        self
    }
}

#[derive(Debug, Deserialize)]
struct StabilityImageResponse {
    image: String,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct StabilityError {
    name: Option<String>,
    message: String,
}

pub mod models {
    pub const SD3_LARGE: &str = "sd3-large";
    pub const SD3_LARGE_TURBO: &str = "sd3-large-turbo";
    pub const SD3_MEDIUM: &str = "sd3-medium";
}

pub mod aspect_ratios {
    pub const SQUARE: &str = "1:1";
    pub const LANDSCAPE_16_9: &str = "16:9";
    pub const LANDSCAPE_21_9: &str = "21:9";
    pub const PORTRAIT_9_16: &str = "9:16";
    pub const PORTRAIT_2_3: &str = "2:3";
    pub const LANDSCAPE_3_2: &str = "3:2";
    pub const LANDSCAPE_4_5: &str = "4:5";
    pub const PORTRAIT_5_4: &str = "5:4";
}
