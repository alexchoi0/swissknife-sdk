use async_trait::async_trait;
use serde::Deserialize;

use crate::voice::{CallResponse, VoiceCaller};
use crate::{Error, Result};

use super::TwilioClient;

#[derive(Deserialize)]
struct TwilioCallResponse {
    sid: String,
    status: String,
    #[serde(default)]
    error_code: Option<i32>,
    #[serde(default)]
    error_message: Option<String>,
}

#[async_trait]
impl VoiceCaller for TwilioClient {
    async fn make_call(&self, from: &str, to: &str, twiml_url: &str) -> Result<CallResponse> {
        let params = [("From", from), ("To", to), ("Url", twiml_url)];

        let response = self
            .http
            .post(&self.calls_url())
            .header("Authorization", &self.auth_header)
            .form(&params)
            .send()
            .await?;

        let status_code = response.status();
        let response_body: TwilioCallResponse = response.json().await?;

        if let Some(error_code) = response_body.error_code {
            return Err(Error::Api {
                code: error_code,
                message: response_body
                    .error_message
                    .unwrap_or_else(|| format!("HTTP {}", status_code)),
            });
        }

        Ok(CallResponse {
            call_id: response_body.sid,
            status: response_body.status,
        })
    }
}
