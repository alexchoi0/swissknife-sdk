use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;

use crate::push::{PushNotification, PushResponse, PushSender};
use crate::{Error, Result};

use super::ApnsClient;

#[derive(Serialize)]
struct ApnsPayload {
    aps: ApnsAps,
    #[serde(flatten)]
    data: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
struct ApnsAps {
    alert: ApnsAlert,
    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<String>,
}

#[derive(Serialize)]
struct ApnsAlert {
    title: String,
    body: String,
}

#[async_trait]
impl PushSender for ApnsClient {
    async fn send_to_token(
        &self,
        token: &str,
        notification: &PushNotification,
    ) -> Result<PushResponse> {
        let payload = ApnsPayload {
            aps: ApnsAps {
                alert: ApnsAlert {
                    title: notification.title.clone(),
                    body: notification.body.clone(),
                },
                badge: None,
                sound: Some("default".to_string()),
            },
            data: notification.data.clone(),
        };

        let response = self
            .http
            .post(&self.device_url(token))
            .header("authorization", self.auth_header())
            .header("apns-topic", &self.bundle_id)
            .header("apns-push-type", "alert")
            .json(&payload)
            .send()
            .await?;

        let status_code = response.status();
        let apns_id = response
            .headers()
            .get("apns-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if !status_code.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                code: status_code.as_u16() as i32,
                message: error_text,
            });
        }

        Ok(PushResponse {
            message_id: apns_id,
            success_count: 1,
            failure_count: 0,
        })
    }

    async fn send_to_topic(
        &self,
        _topic: &str,
        _notification: &PushNotification,
    ) -> Result<PushResponse> {
        Err(Error::Config(
            "APNs does not support topic-based messaging directly. Use FCM for topic support."
                .into(),
        ))
    }
}
