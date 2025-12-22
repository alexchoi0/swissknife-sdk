use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::push::{PushNotification, PushResponse, PushSender};
use crate::{Error, Result};

use super::FcmClient;

#[derive(Serialize)]
struct FcmRequest {
    message: FcmMessage,
}

#[derive(Serialize)]
struct FcmMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    notification: FcmNotification,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
struct FcmNotification {
    title: String,
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

#[derive(Deserialize)]
struct FcmResponse {
    name: Option<String>,
}

#[derive(Deserialize)]
struct FcmError {
    error: FcmErrorDetail,
}

#[derive(Deserialize)]
struct FcmErrorDetail {
    code: i32,
    message: String,
}

impl FcmClient {
    async fn send_message(&self, message: FcmMessage) -> Result<PushResponse> {
        let request = FcmRequest { message };

        let response = self
            .http
            .post(&self.send_url())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status_code = response.status();

        if !status_code.is_success() {
            let error: FcmError = response.json().await.map_err(|_| Error::Api {
                code: status_code.as_u16() as i32,
                message: "Failed to parse error response".into(),
            })?;
            return Err(Error::Api {
                code: error.error.code,
                message: error.error.message,
            });
        }

        let result: FcmResponse = response.json().await?;

        Ok(PushResponse {
            message_id: result.name,
            success_count: 1,
            failure_count: 0,
        })
    }
}

#[async_trait]
impl PushSender for FcmClient {
    async fn send_to_token(
        &self,
        token: &str,
        notification: &PushNotification,
    ) -> Result<PushResponse> {
        let message = FcmMessage {
            token: Some(token.to_string()),
            topic: None,
            notification: FcmNotification {
                title: notification.title.clone(),
                body: notification.body.clone(),
                image: notification.image.clone(),
            },
            data: notification.data.clone(),
        };

        self.send_message(message).await
    }

    async fn send_to_topic(
        &self,
        topic: &str,
        notification: &PushNotification,
    ) -> Result<PushResponse> {
        let message = FcmMessage {
            token: None,
            topic: Some(topic.to_string()),
            notification: FcmNotification {
                title: notification.title.clone(),
                body: notification.body.clone(),
                image: notification.image.clone(),
            },
            data: notification.data.clone(),
        };

        self.send_message(message).await
    }
}
