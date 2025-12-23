use crate::{Error, Result};
use crate::hunter::HunterClient;
use serde::Deserialize;

impl HunterClient {
    pub async fn verify_email(&self, email: &str) -> Result<EmailVerificationResponse> {
        let response = self.client()
            .get(format!("{}/email-verifier", self.base_url()))
            .query(&[
                ("email", email),
                ("api_key", self.api_key()),
            ])
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

        let result: super::search::HunterResponse<EmailVerificationData> = response.json().await?;
        Ok(EmailVerificationResponse { data: result.data })
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        let response = self.client()
            .get(format!("{}/account", self.base_url()))
            .query(&[("api_key", self.api_key())])
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

        let result: super::search::HunterResponse<AccountInfo> = response.json().await?;
        Ok(result.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailVerificationResponse {
    pub data: EmailVerificationData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailVerificationData {
    pub email: String,
    pub result: Option<String>,
    pub score: Option<i32>,
    pub regexp: Option<bool>,
    pub gibberish: Option<bool>,
    pub disposable: Option<bool>,
    pub webmail: Option<bool>,
    pub mx_records: Option<bool>,
    pub smtp_server: Option<bool>,
    pub smtp_check: Option<bool>,
    pub accept_all: Option<bool>,
    pub block: Option<bool>,
    pub sources: Option<Vec<super::search::HunterSource>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub plan_name: Option<String>,
    pub plan_level: Option<i32>,
    pub reset_date: Option<String>,
    pub team_id: Option<i64>,
    pub calls: Option<AccountCalls>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountCalls {
    pub used: Option<i32>,
    pub available: Option<i32>,
}

impl EmailVerificationData {
    pub fn is_valid(&self) -> bool {
        self.result.as_ref().map(|r| r == "deliverable").unwrap_or(false)
    }

    pub fn is_risky(&self) -> bool {
        self.result.as_ref().map(|r| r == "risky").unwrap_or(false)
    }

    pub fn is_invalid(&self) -> bool {
        self.result.as_ref().map(|r| r == "undeliverable").unwrap_or(false)
    }
}
