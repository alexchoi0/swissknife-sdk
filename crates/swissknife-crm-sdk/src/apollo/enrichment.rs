use crate::{Error, Result};
use crate::apollo::ApolloClient;
use crate::apollo::people::ApolloPerson;
use crate::apollo::organizations::ApolloOrganization;
use serde::Deserialize;

impl ApolloClient {
    pub async fn bulk_enrich_people(&self, emails: &[&str]) -> Result<BulkEnrichResponse> {
        let details: Vec<serde_json::Value> = emails.iter()
            .map(|e| serde_json::json!({"email": e}))
            .collect();

        let response = self.client()
            .post(format!("{}/people/bulk_match", self.base_url()))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "api_key": self.api_key(),
                "details": details
            }))
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

        let result: BulkEnrichResponse = response.json().await?;
        Ok(result)
    }

    pub async fn enrich_person(&self, request: EnrichPersonRequest) -> Result<ApolloPerson> {
        let mut body = serde_json::json!({
            "api_key": self.api_key()
        });

        if let Some(email) = request.email {
            body["email"] = serde_json::Value::String(email);
        }
        if let Some(name) = request.name {
            body["name"] = serde_json::Value::String(name);
        }
        if let Some(first_name) = request.first_name {
            body["first_name"] = serde_json::Value::String(first_name);
        }
        if let Some(last_name) = request.last_name {
            body["last_name"] = serde_json::Value::String(last_name);
        }
        if let Some(domain) = request.domain {
            body["domain"] = serde_json::Value::String(domain);
        }
        if let Some(linkedin_url) = request.linkedin_url {
            body["linkedin_url"] = serde_json::Value::String(linkedin_url);
        }

        let response = self.client()
            .post(format!("{}/people/match", self.base_url()))
            .header("Content-Type", "application/json")
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

        let result: super::people::PersonResponse = response.json().await?;
        Ok(result.person)
    }

    pub async fn get_email_status(&self, email: &str) -> Result<EmailStatusResponse> {
        let response = self.client()
            .post(format!("{}/emails/verify", self.base_url()))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "api_key": self.api_key(),
                "email": email
            }))
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

        let result: EmailStatusResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EnrichPersonRequest {
    pub email: Option<String>,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub domain: Option<String>,
    pub linkedin_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkEnrichResponse {
    pub matches: Vec<BulkEnrichMatch>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkEnrichMatch {
    pub email: Option<String>,
    pub person: Option<ApolloPerson>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailStatusResponse {
    pub email: String,
    pub is_valid: Option<bool>,
    pub is_verified: Option<bool>,
    pub is_free_email: Option<bool>,
    pub is_disposable_email: Option<bool>,
    pub is_role_email: Option<bool>,
    pub status: Option<String>,
}
